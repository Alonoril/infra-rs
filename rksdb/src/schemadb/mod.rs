#![forbid(unsafe_code)]

#[macro_use]
pub mod schema;
pub mod iterator;
pub mod ttl;
pub mod batch;
pub mod utils;
pub mod db_impl;

// Re-export public types and traits
pub use batch::{SchemaBatch, ColumnFamilyName};
pub use db_impl::RksDB;
pub use utils::IntoDbResult;
pub use schema::Schema;

/// Type alias to `rocksdb::ReadOptions`. See [`rocksdb doc`](https://github.com/pingcap/rust-rocksdb/blob/master/src/rocksdb_options.rs)
pub use rocksdb::{
    BlockBasedOptions, Cache, ColumnFamilyDescriptor, DBCompressionType, DEFAULT_COLUMN_FAMILY_NAME,
    Options, ReadOptions, SliceTransform,
};
