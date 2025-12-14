use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DbPathConfig {
	pub rks_db_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RocksdbConfig {
	/// Maximum number of files open by RocksDB at one time
	pub max_open_files: i32,
	/// Maximum size of the RocksDB write ahead log (WAL)
	pub max_total_wal_size: u64,
	/// Maximum number of background threads for Rocks DB
	pub max_background_jobs: i32,
	/// Block cache size for Rocks DB
	pub block_cache_size: u64,
	/// Block size for Rocks DB
	pub block_size: u64,
	/// Whether cache index and filter blocks into block cache.
	pub cache_index_and_filter_blocks: bool,
}

impl Default for RocksdbConfig {
	fn default() -> Self {
		Self {
			// Allow db to close old sst files, saving memory.
			max_open_files: 5000,
			// For now we set the max total WAL size to be 1G. This config can be useful when column
			// families are updated at non-uniform frequencies.
			max_total_wal_size: 1u64 << 30,
			// This includes threads for flashing and compaction. Rocksdb will decide the # of
			// threads to use internally.
			max_background_jobs: 16,
			// Default block cache size is 8MB,
			block_cache_size: 8 * (1u64 << 20),
			// Default block cache size is 4KB,
			block_size: 4 * (1u64 << 10),
			// Whether cache index and filter blocks into block cache.
			cache_index_and_filter_blocks: false,
		}
	}
}

#[derive(Default, Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RocksdbConfigs {
	pub rks_db_config: RocksdbConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RksdbConfig {
	/// Top level directory to store the RocksDB
	pub dir: PathBuf,
	/// Subdirectory for handler in tests only
	#[serde(skip)]
	data_dir: PathBuf,
	/// Rocksdb-specific configurations
	pub rocksdb_configs: RocksdbConfigs,
}

#[derive(Clone)]
pub struct StorageDirPaths {
	default_path: PathBuf,
	rks_db_path: Option<PathBuf>,
}

impl Default for RksdbConfig {
	fn default() -> RksdbConfig {
		RksdbConfig {
			dir: PathBuf::from("db"),
			data_dir: PathBuf::from("/opt/app/data"),
			rocksdb_configs: RocksdbConfigs::default(),
		}
	}
}

impl RksdbConfig {
	pub fn dir(&self) -> PathBuf {
		if self.dir.is_relative() {
			self.data_dir.join(&self.dir)
		} else {
			self.dir.clone()
		}
	}

	pub fn get_dir_paths(&self) -> StorageDirPaths {
		let default_dir = self.dir();
		StorageDirPaths::new(default_dir, None)
	}

	pub fn set_data_dir(&mut self, data_dir: PathBuf) {
		self.data_dir = data_dir;
	}
}

impl StorageDirPaths {
	pub fn default_root_path(&self) -> &PathBuf {
		&self.default_path
	}

	pub fn signet_db_root_path(&self) -> &PathBuf {
		if let Some(signet_db_path) = self.rks_db_path.as_ref() {
			signet_db_path
		} else {
			&self.default_path
		}
	}

	pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
		Self {
			default_path: path.as_ref().to_path_buf(),
			rks_db_path: None,
		}
	}

	fn new(default_path: PathBuf, signet_db_path: Option<PathBuf>) -> Self {
		Self {
			default_path,
			rks_db_path: signet_db_path,
		}
	}
}
