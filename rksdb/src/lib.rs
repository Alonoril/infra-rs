pub mod codec;
pub mod errors;
pub mod rksdb_config;
mod rocksdb_opts;
pub mod schemadb;

use anyhow::ensure;
pub use rocksdb_opts::*;
use std::path::PathBuf;

use crate::{
	errors::RksDbError,
	schemadb::{ColumnFamilyName, RksDB},
};
use tracing::info;

use crate::rksdb_config::{RocksdbConfig, StorageDirPaths};
use base_infra::result::AppResult;
use rocksdb::{BlockBasedOptions, Cache, ColumnFamilyDescriptor, DBCompressionType, Options};

pub use rocksdb::DEFAULT_COLUMN_FAMILY_NAME;
pub type DbResult<T, E = RksDbError> = Result<T, E>;

fn ensure_slice_len_eq(data: &[u8], len: usize) -> anyhow::Result<()> {
	ensure!(
		data.len() == len,
		"Unexpected data len {}, expected {}.",
		data.len(),
		len,
	);
	Ok(())
}

fn ensure_slice_len_gt(data: &[u8], len: usize) -> anyhow::Result<()> {
	ensure!(
		data.len() > len,
		"Unexpected data len {}, expected to be greater than {}.",
		data.len(),
		len,
	);
	Ok(())
}

pub trait OpenRocksDB {
	fn new(
		path: PathBuf,
		name: &str,
		db_config: &RocksdbConfig,
		readonly: bool,
		with_ttl: bool,
	) -> AppResult<Self>
	where
		Self: Sized,
	{
		let db = Self::open_rocksdb(path, name, db_config, readonly, with_ttl)?;
		// Ok(Self { db: Arc::new(db) })
		Self::new_inner(db)
	}

	fn new_inner(db: RksDB) -> AppResult<Self>
	where
		Self: Sized;

	fn get_db_column_families() -> Vec<ColumnFamilyName>;

	fn get_db_column_families_with_ttl() -> Vec<ColumnFamilyName> {
		let mut cfs = Self::get_db_column_families();
		cfs.extend(RksDB::get_ttl_column_families());
		cfs
	}

	fn get_db_path(db_paths: StorageDirPaths) -> PathBuf;

	fn open_rocksdb(
		path: PathBuf,
		name: &str,
		db_config: &RocksdbConfig,
		readonly: bool,
		with_ttl: bool,
	) -> AppResult<RksDB> {
		let db = if readonly {
			RksDB::open_cf_readonly(
				&gen_rocksdb_options(db_config, true),
				path.clone(),
				name,
				Self::gen_db_cfds(with_ttl, db_config, |_, _| {}),
			)?
		} else {
			RksDB::open_cf(
				&gen_rocksdb_options(db_config, false),
				path.clone(),
				name,
				Self::gen_db_cfds(with_ttl, db_config, |_, _| {}),
			)?
		};

		info!("Opened {name} at {path:?}!");
		Ok(db)
	}

	// fn gen_db_cfds(&self, rocksdb_config: &RocksdbConfig) -> Vec<ColumnFamilyDescriptor>;

	fn gen_db_cfds<F>(
		with_ttl: bool,
		rocksdb_config: &RocksdbConfig,
		cf_opts_post_processor: F,
	) -> Vec<ColumnFamilyDescriptor>
	where
		F: Fn(ColumnFamilyName, &mut Options),
	{
		let cfs = if with_ttl {
			Self::get_db_column_families_with_ttl()
		} else {
			Self::get_db_column_families()
		};
		Self::gen_cfds(rocksdb_config, cfs, cf_opts_post_processor)
	}

	fn gen_cfds<F>(
		rocksdb_config: &RocksdbConfig,
		cfs: Vec<ColumnFamilyName>,
		cf_opts_post_processor: F,
	) -> Vec<ColumnFamilyDescriptor>
	where
		F: Fn(ColumnFamilyName, &mut Options),
	{
		let mut table_options = BlockBasedOptions::default();
		table_options.set_cache_index_and_filter_blocks(rocksdb_config.cache_index_and_filter_blocks);
		table_options.set_block_size(rocksdb_config.block_size as usize);
		let cache = Cache::new_lru_cache(rocksdb_config.block_cache_size as usize);
		table_options.set_block_cache(&cache);
		let mut cfds = Vec::with_capacity(cfs.len());
		for cf_name in cfs {
			let mut cf_opts = Options::default();
			cf_opts.set_compression_type(DBCompressionType::Lz4);
			cf_opts.set_block_based_table_factory(&table_options);
			cf_opts_post_processor(cf_name, &mut cf_opts);
			cfds.push(ColumnFamilyDescriptor::new((*cf_name).to_string(), cf_opts));
		}
		cfds
	}
}
