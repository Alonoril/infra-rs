#![forbid(unsafe_code)]

#[macro_use]
pub mod schema;
pub mod batch;
pub mod db_impl;
pub mod iterator;
pub mod ttl;
pub mod utils;

// Re-export public types and traits
pub use batch::{ColumnFamilyName, SchemaBatch};
pub use db_impl::RksDB;
pub use schema::Schema;
pub use utils::IntoDbResult;

/// Type alias to `rocksdb::ReadOptions`. See [`rocksdb doc`](https://github.com/pingcap/rust-rocksdb/blob/master/src/rocksdb_options.rs)
pub use rocksdb::{
	BlockBasedOptions, Cache, ColumnFamilyDescriptor, DBCompressionType, DEFAULT_COLUMN_FAMILY_NAME,
	Options, ReadOptions, SliceTransform,
};
