use crate::{
	errors::RksDbError,
	schemadb::{
		batch::{SchemaBatch, WriteOp},
		iterator::{ScanDirection, SchemaIterator},
		schema::{KeyCodec, Schema, ValueCodec},
		utils::{DeUnc, IntoDbResult, OpenMode, default_write_options},
	},
};
use anyhow::format_err;
use base_infra::result::AppResult;
use rocksdb::{ColumnFamilyDescriptor, DBCompressionType, Options, ReadOptions};
use std::{collections::HashSet, path::Path};
use tracing::{info, warn};

/// This DB is a schematized RocksDB wrapper where all data passed in and out are typed according to
/// [`Schema`]s.
#[derive(Debug)]
pub struct RksDB {
	name: String, // for logging
	pub(crate) inner: rocksdb::DB,
}

impl RksDB {
	pub fn open(
		path: impl AsRef<Path>,
		name: &str,
		column_families: Vec<&'static str>,
		db_opts: &Options,
	) -> AppResult<Self> {
		let db = RksDB::open_cf(
			db_opts,
			path,
			name,
			column_families
				.iter()
				.map(|cf_name| {
					let mut cf_opts = Options::default();
					cf_opts.set_compression_type(DBCompressionType::Lz4);
					ColumnFamilyDescriptor::new((*cf_name).to_string(), cf_opts)
				})
				.collect(),
		)?;
		Ok(db)
	}

	pub fn open_cf(
		db_opts: &Options,
		path: impl AsRef<Path>,
		name: &str,
		cfds: Vec<ColumnFamilyDescriptor>,
	) -> AppResult<RksDB> {
		Self::open_cf_impl(db_opts, path, name, cfds, OpenMode::ReadWrite)
	}

	/// Open db in readonly mode
	/// Note that this still assumes there's only one process that opens the same DB.
	/// See `open_as_secondary`
	pub fn open_cf_readonly(
		opts: &Options,
		path: impl AsRef<Path>,
		name: &str,
		cfds: Vec<ColumnFamilyDescriptor>,
	) -> AppResult<RksDB> {
		Self::open_cf_impl(opts, path, name, cfds, OpenMode::ReadOnly)
	}

	pub fn open_cf_as_secondary<P: AsRef<Path>>(
		opts: &Options,
		primary_path: P,
		secondary_path: P,
		name: &str,
		cfds: Vec<ColumnFamilyDescriptor>,
	) -> AppResult<RksDB> {
		Self::open_cf_impl(
			opts,
			primary_path,
			name,
			cfds,
			OpenMode::Secondary(secondary_path.as_ref()),
		)
	}

	fn open_cf_impl(
		db_opts: &Options,
		path: impl AsRef<Path>,
		name: &str,
		cfds: Vec<ColumnFamilyDescriptor>,
		open_mode: OpenMode,
	) -> AppResult<RksDB> {
		// ignore error, since it'll fail to list cfs on the first open
		let existing_cfs = rocksdb::DB::list_cf(db_opts, path.de_unc()).unwrap_or_default();

		let unrecognized_cfds = existing_cfs
			.iter()
			.map(AsRef::as_ref)
			.collect::<HashSet<&str>>()
			.difference(&cfds.iter().map(|cfd| cfd.name()).collect())
			.map(|cf| {
				warn!("Unrecognized CF: {}", cf);

				let mut cf_opts = Options::default();
				cf_opts.set_compression_type(DBCompressionType::Lz4);
				ColumnFamilyDescriptor::new(cf.to_string(), cf_opts)
			})
			.collect::<Vec<_>>();
		let all_cfds = cfds.into_iter().chain(unrecognized_cfds);

		let inner = {
			use OpenMode::*;
			use rocksdb::DB;

			match open_mode {
				ReadWrite => DB::open_cf_descriptors(db_opts, path.de_unc(), all_cfds),
				ReadOnly => {
					DB::open_cf_descriptors_read_only(
						db_opts,
						path.de_unc(),
						all_cfds,
						false, /* error_if_log_file_exist */
					)
				}
				Secondary(secondary_path) => DB::open_cf_descriptors_as_secondary(
					db_opts,
					path.de_unc(),
					secondary_path,
					all_cfds,
				),
			}
		}
		.into_db_res()?;

		Ok(Self::log_construct(name, open_mode, inner))
	}

	fn log_construct(name: &str, open_mode: OpenMode, inner: rocksdb::DB) -> RksDB {
		info!(
			rocksdb_name = name,
			open_mode = ?open_mode,
			"Opened RocksDB."
		);
		RksDB {
			name: name.to_string(),
			inner,
		}
	}

	/// Reads single record by key.
	pub fn get<S: Schema>(&self, schema_key: &S::Key) -> AppResult<Option<S::Value>> {
		let k = <S::Key as KeyCodec<S>>::encode_key(schema_key)?;
		let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY_NAME)?;

		let result = self.inner.get_cf(cf_handle, k).into_db_res()?;
		result
			.map(|raw_value| <S::Value as ValueCodec<S>>::decode_value(&raw_value))
			.transpose()
			.map_err(Into::into)
	}

	pub fn get_all<S: Schema>(&self) -> AppResult<Vec<(S::Key, S::Value)>> {
		let mut iter = self.iter::<S>()?;
		iter.seek_to_first();
		Ok(iter.collect::<AppResult<Vec<(S::Key, S::Value)>>>()?)
	}

	pub fn multi_get<S: Schema>(&self, keys: &[S::Key]) -> AppResult<Vec<Option<S::Value>>> {
		let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY_NAME)?;
		let mut encoded_keys = vec![];
		for key in keys {
			encoded_keys.push((cf_handle, <S::Key as KeyCodec<S>>::encode_key(key)?));
		}

		let results: Vec<Result<Option<Vec<u8>>, rocksdb::Error>> =
			self.inner.multi_get_cf(encoded_keys);
		let mut res_vec = vec![];
		for result in results {
			if result.is_err() {
				res_vec.push(None);
				continue;
			}

			if let Some(raw_value) = result.into_db_res()? {
				res_vec.push(Some(<S::Value as ValueCodec<S>>::decode_value(&raw_value)?));
			}
		}

		Ok(res_vec)
	}

	pub fn multi_get_with_key<'a, S: Schema>(
		&self,
		keys: &'a [S::Key],
	) -> AppResult<Vec<(&'a S::Key, Option<S::Value>)>> {
		let values: Vec<Option<S::Value>> = self.multi_get::<S>(keys)?;
		let res_vec = keys.into_iter().zip(values.into_iter()).collect();
		Ok(res_vec)
	}

	pub fn multi_exists<S: Schema>(&self, keys: &[S::Key]) -> AppResult<Vec<bool>> {
		let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY_NAME)?;
		let mut encoded_keys = vec![];
		for key in keys {
			encoded_keys.push((cf_handle, <S::Key as KeyCodec<S>>::encode_key(key)?));
		}

		let results: Vec<Result<Option<Vec<u8>>, rocksdb::Error>> =
			self.inner.multi_get_cf(encoded_keys);
		let res_vec: Vec<bool> = results
			.iter()
			.map(|result| match result {
				Ok(Some(_)) => true,
				_ => false,
			})
			.collect();

		Ok(res_vec)
	}

	/// Writes single record.
	pub fn put<S: Schema>(&self, key: &S::Key, value: &S::Value) -> AppResult<()> {
		// Not necessary to use a batch, but we'd like a central place to bump counters.
		let batch = SchemaBatch::new();
		batch.put::<S>(key, value)?;
		self.write_schemas(batch)
	}

	/// Deletes a single record.
	pub fn delete<S: Schema>(&self, key: &S::Key) -> AppResult<()> {
		// Not necessary to use a batch, but we'd like a central place to bump counters.
		let batch = SchemaBatch::new();
		batch.delete::<S>(key)?;
		self.write_schemas(batch)
	}

	fn iter_with_direction<S: Schema>(
		&self,
		opts: ReadOptions,
		direction: ScanDirection,
	) -> AppResult<SchemaIterator<S>> {
		let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY_NAME)?;
		Ok(SchemaIterator::new(
			self.inner.raw_iterator_cf_opt(cf_handle, opts),
			direction,
		))
	}

	/// Returns a forward [`SchemaIterator`] on a certain schemadb.
	pub fn iter<S: Schema>(&self) -> AppResult<SchemaIterator<S>> {
		self.iter_with_opts(ReadOptions::default())
	}

	/// Returns a forward [`SchemaIterator`] on a certain schemadb, with non-default ReadOptions
	pub fn iter_with_opts<S: Schema>(&self, opts: ReadOptions) -> AppResult<SchemaIterator<S>> {
		self.iter_with_direction::<S>(opts, ScanDirection::Forward)
	}

	/// Returns a backward [`SchemaIterator`] on a certain schemadb.
	pub fn rev_iter<S: Schema>(&self) -> AppResult<SchemaIterator<S>> {
		self.rev_iter_with_opts(ReadOptions::default())
	}

	/// Returns a backward [`SchemaIterator`] on a certain schemadb, with non-default ReadOptions
	pub fn rev_iter_with_opts<S: Schema>(&self, opts: ReadOptions) -> AppResult<SchemaIterator<S>> {
		self.iter_with_direction::<S>(opts, ScanDirection::Backward)
	}

	/// Writes a group of records wrapped in a [`SchemaBatch`].
	pub fn write_schemas(&self, batch: SchemaBatch) -> AppResult<()> {
		let rows_locked = batch
			.rows
			.lock()
			.expect("Cannot currently handle a poisoned lock");

		let mut db_batch = rocksdb::WriteBatch::default();
		for (cf_name, rows) in rows_locked.iter() {
			let cf_handle = self.get_cf_handle(cf_name)?;
			for write_op in rows {
				match write_op {
					WriteOp::Value { key, value } => db_batch.put_cf(cf_handle, key, value),
					WriteOp::Deletion { key } => db_batch.delete_cf(cf_handle, key),
				}
			}
		}

		self.inner
			.write_opt(db_batch, &default_write_options())
			.into_db_res()?;

		Ok(())
	}

	pub(crate) fn get_cf_handle(&self, cf_name: &str) -> AppResult<&rocksdb::ColumnFamily> {
		self.inner
			.cf_handle(cf_name)
			.ok_or_else(|| {
				format_err!(
					"DB::cf_handle not found for column family name: {}",
					cf_name
				)
			})
			.map_err(Into::into)
	}

	/// Flushes memtable data. This is only used for testing `get_approximate_sizes_cf` in unit
	/// tests.
	pub fn flush_cf(&self, cf_name: &str) -> AppResult<()> {
		Ok(self
			.inner
			.flush_cf(self.get_cf_handle(cf_name)?)
			.into_db_res()?)
	}

	pub fn get_property(&self, cf_name: &str, property_name: &str) -> AppResult<u64> {
		Ok(self
			.inner
			.property_int_value_cf(self.get_cf_handle(cf_name)?, property_name)
			.into_db_res()?
			.ok_or_else(|| {
				RksDbError::Other(
					format!(
						"Unable to get property \"{}\" of  column family \"{}\".",
						property_name, cf_name,
					)
					.to_string(),
				)
			})?)
	}

	/// Creates new physical DB checkpoint in directory specified by `path`.
	pub fn create_checkpoint<P: AsRef<Path>>(&self, path: P) -> AppResult<()> {
		rocksdb::checkpoint::Checkpoint::new(&self.inner)
			.into_db_res()?
			.create_checkpoint(path)
			.into_db_res()?;
		Ok(())
	}
}

impl Drop for RksDB {
	fn drop(&mut self) {
		info!(rocksdb_name = self.name, "Dropped RocksDB.");
	}
}
