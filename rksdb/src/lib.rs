pub mod codec;
pub mod errors;
mod rdb_opts;
pub mod schemadb;

use crate::{
	errors::RksDbError,
	schemadb::{ColumnFamilyName, RksDB},
};
pub use rdb_opts::*;
use std::path::PathBuf;
use std::time::Instant;
use tracing::info;

use base_infra::result::AppResult;
use rocksdb::{BlockBasedOptions, Cache, ColumnFamilyDescriptor, DBCompressionType, Options};

use rksdb_cfg::{RksDbDirPaths, RocksdbConfig};
pub use rocksdb::DEFAULT_COLUMN_FAMILY_NAME;

pub type DbResult<T, E = RksDbError> = Result<T, E>;

pub type CfPost = fn(ColumnFamilyName, &mut Options);

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

	fn cf_opts_post_processor() -> CfPost {
		noop_cf_post
	}

	fn gen_db_cfds(with_ttl: bool, rocksdb_config: &RocksdbConfig) -> Vec<ColumnFamilyDescriptor> {
		let post = Self::cf_opts_post_processor();

		if with_ttl {
			let cfs = Self::get_db_column_families_with_ttl();
			build_cfds_with_post(rocksdb_config, &cfs, post)
		} else {
			build_cfds_with_post(rocksdb_config, &Self::get_db_column_families(), post)
		}
	}

	fn open_rocksdb(
		path: PathBuf,
		name: &str,
		db_config: &RocksdbConfig,
		readonly: bool,
		with_ttl: bool,
	) -> AppResult<RksDB> {
		let started_at = Instant::now();

		let cfds = Self::gen_db_cfds(with_ttl, db_config);

		let db = if readonly {
			RksDB::open_cf_readonly(
				&gen_rocksdb_options(db_config, true),
				path.clone(),
				name,
				cfds,
			)?
		} else {
			RksDB::open_cf(
				&gen_rocksdb_options(db_config, false),
				path.clone(),
				name,
				cfds,
			)?
		};

		info!(
			"Database {name} opened in {:?} at {path:?}!",
			started_at.elapsed()
		);
		Ok(db)
	}

	fn get_db_path(db_paths: RksDbDirPaths) -> PathBuf;
}

#[inline]
pub fn noop_cf_post(_: ColumnFamilyName, _: &mut Options) {}

pub fn build_table_opts(rocksdb_config: &RocksdbConfig) -> (BlockBasedOptions, Cache) {
	let mut table_opts = BlockBasedOptions::default();
	table_opts.set_cache_index_and_filter_blocks(rocksdb_config.cache_index_and_filter_blocks);
	table_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
	table_opts.set_block_size(rocksdb_config.block_size as usize);

	let cache = Cache::new_lru_cache(rocksdb_config.block_cache_size as usize);
	table_opts.set_block_cache(&cache);

	table_opts.set_hybrid_ribbon_filter(10.0, 1);
	table_opts.set_format_version(5);

	// 返回 cache 是为了延长其生命周期（避免被 drop 后 table_opts 内引用失效）
	(table_opts, cache)
}

//     // bottommost 字典大小（max_dict_bytes）：比如 16KB / 32KB 常见
//     // 参数含义与 set_compression_options 相同；对 zstd 来说你主要关心 max_dict_bytes。
//     cf_opts.set_bottommost_compression_options(
//         0,   // w_bits（更多是 zlib 场景）
//         0,   // level（更多是 zlib 场景）
//         0,   // strategy（更多是 zlib 场景）
//         32 * 1024, // max_dict_bytes：字典最大大小（示例 32KiB）
//         true,      // enabled：必须 true 才会启用 bottommost 配置
//     );
//
//     // zstd 训练数据上限（train_bytes）：建议从 0 或几十/几百 KB 起步逐渐调
//     // ⚠️ train_bytes 越大，压缩率可能更好，但训练/内存开销越高
//     cf_opts.set_bottommost_zstd_max_train_bytes(256 * 1024, true);
pub fn build_cfds_with_post(
	rocksdb_config: &RocksdbConfig,
	cfs: &[ColumnFamilyName],
	post: CfPost,
) -> Vec<ColumnFamilyDescriptor> {
	let (table_opts, _cache) = build_table_opts(rocksdb_config);

	let mut cfds = Vec::with_capacity(cfs.len());
	for &cf_name in cfs {
		let mut cf_opts = Options::default();

		// L1~Ln LZ4
		cf_opts.set_compression_type(DBCompressionType::Lz4);
		// bottommost ZSTD
		cf_opts.set_bottommost_compression_type(DBCompressionType::Zstd);
		cf_opts.set_bottommost_zstd_max_train_bytes(0, true);

		cf_opts.set_level_compaction_dynamic_level_bytes(true);
		cf_opts.set_block_based_table_factory(&table_opts);

		post(cf_name, &mut cf_opts);

		cfds.push(ColumnFamilyDescriptor::new((*cf_name).to_string(), cf_opts));
	}
	cfds
}

// pub trait OpenRocksDB {
// 	fn new(
// 		path: PathBuf,
// 		name: &str,
// 		db_config: &RocksdbConfig,
// 		readonly: bool,
// 		with_ttl: bool,
// 	) -> AppResult<Self>
// 	where
// 		Self: Sized,
// 	{
// 		let db = Self::open_rocksdb(path, name, db_config, readonly, with_ttl)?;
// 		// Ok(Self { db: Arc::new(db) })
// 		Self::new_inner(db)
// 	}
//
// 	fn new_inner(db: RksDB) -> AppResult<Self>
// 	where
// 		Self: Sized;
//
// 	fn get_db_column_families() -> Vec<ColumnFamilyName>;
//
// 	fn get_db_column_families_with_ttl() -> Vec<ColumnFamilyName> {
// 		let mut cfs = Self::get_db_column_families();
// 		cfs.extend(RksDB::get_ttl_column_families());
// 		cfs
// 	}
//
// 	fn get_db_path(db_paths: StorageDirPaths) -> PathBuf;
//
// 	fn open_rocksdb(
// 		path: PathBuf,
// 		name: &str,
// 		db_config: &RocksdbConfig,
// 		readonly: bool,
// 		with_ttl: bool,
// 	) -> AppResult<RksDB> {
// 		let started_at = Instant::now();
// 		let db = if readonly {
// 			RksDB::open_cf_readonly(
// 				&gen_rocksdb_options(db_config, true),
// 				path.clone(),
// 				name,
// 				Self::gen_db_cfds(with_ttl, db_config, |_, _| {}),
// 			)?
// 		} else {
// 			RksDB::open_cf(
// 				&gen_rocksdb_options(db_config, false),
// 				path.clone(),
// 				name,
// 				Self::gen_db_cfds(with_ttl, db_config, |_, _| {}),
// 			)?
// 		};
//
// 		let elapsed = started_at.elapsed();
// 		info!("Database opened {name} in {elapsed:.3?} at {path:?}!");
// 		Ok(db)
// 	}
//
// 	// fn gen_db_cfds(&self, rocksdb_config: &RocksdbConfig) -> Vec<ColumnFamilyDescriptor>;
//
// 	fn gen_db_cfds<F>(
// 		with_ttl: bool,
// 		rocksdb_config: &RocksdbConfig,
// 		cf_opts_post_processor: F,
// 	) -> Vec<ColumnFamilyDescriptor>
// 	where
// 		F: Fn(ColumnFamilyName, &mut Options),
// 	{
// 		let cfs = if with_ttl {
// 			Self::get_db_column_families_with_ttl()
// 		} else {
// 			Self::get_db_column_families()
// 		};
// 		Self::gen_cfds(rocksdb_config, cfs, cf_opts_post_processor)
// 	}
//
// 	//     // bottommost 字典大小（max_dict_bytes）：比如 16KB / 32KB 常见
// 	//     // 参数含义与 set_compression_options 相同；对 zstd 来说你主要关心 max_dict_bytes。
// 	//     cf_opts.set_bottommost_compression_options(
// 	//         0,   // w_bits（更多是 zlib 场景）
// 	//         0,   // level（更多是 zlib 场景）
// 	//         0,   // strategy（更多是 zlib 场景）
// 	//         32 * 1024, // max_dict_bytes：字典最大大小（示例 32KiB）
// 	//         true,      // enabled：必须 true 才会启用 bottommost 配置
// 	//     );
// 	//
// 	//     // zstd 训练数据上限（train_bytes）：建议从 0 或几十/几百 KB 起步逐渐调
// 	//     // ⚠️ train_bytes 越大，压缩率可能更好，但训练/内存开销越高
// 	//     cf_opts.set_bottommost_zstd_max_train_bytes(256 * 1024, true);
// 	fn gen_cfds<F>(
// 		rocksdb_config: &RocksdbConfig,
// 		cfs: Vec<ColumnFamilyName>,
// 		cf_opts_post_processor: F,
// 	) -> Vec<ColumnFamilyDescriptor>
// 	where
// 		F: Fn(ColumnFamilyName, &mut Options),
// 	{
// 		let mut table_opts = BlockBasedOptions::default();
// 		table_opts.set_cache_index_and_filter_blocks(rocksdb_config.cache_index_and_filter_blocks);
// 		table_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
// 		table_opts.set_block_size(rocksdb_config.block_size as usize);
// 		let cache = Cache::new_lru_cache(rocksdb_config.block_cache_size as usize);
// 		table_opts.set_block_cache(&cache);
// 		table_opts.set_hybrid_ribbon_filter(10.0, 1);
// 		table_opts.set_format_version(5);
//
// 		let mut cfds = Vec::with_capacity(cfs.len());
// 		for cf_name in cfs {
// 			let mut cf_opts = Options::default();
// 			// L1~Ln With LZ4 (Fast)
// 			cf_opts.set_compression_type(DBCompressionType::Lz4);
// 			// bottommost With ZSTD (Space Saving)
// 			cf_opts.set_bottommost_compression_type(DBCompressionType::Zstd);
// 			// 真正“启用” bottommost compression
// 			// 最简单、最稳 —— 不启用字典训练（train_bytes=0），但 enabled=true
// 			cf_opts.set_bottommost_zstd_max_train_bytes(0, true);
//
// 			cf_opts.set_level_compaction_dynamic_level_bytes(true);
// 			cf_opts.set_block_based_table_factory(&table_opts);
// 			cf_opts_post_processor(cf_name, &mut cf_opts);
// 			cfds.push(ColumnFamilyDescriptor::new((*cf_name).to_string(), cf_opts));
// 		}
// 		cfds
// 	}
// }
