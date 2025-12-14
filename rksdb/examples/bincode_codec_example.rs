use bincode::{Decode, Encode};
use rksdb_infra::schemadb::schema::Schema;
use rksdb_infra::schemadb::{ColumnFamilyName, RksDB};
use rksdb_infra::{define_pub_schema, impl_schema_bin_codec};
use rocksdb::DEFAULT_COLUMN_FAMILY_NAME;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TestKey(i32, i32);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TestValue(i32, String, bool);

define_pub_schema!(TestSchema, TestKey, TestValue, "TestSchema");

impl_schema_bin_codec!(TestSchema, TestKey, TestValue);

fn get_column_families() -> Vec<ColumnFamilyName> {
	vec![DEFAULT_COLUMN_FAMILY_NAME, TestSchema::COLUMN_FAMILY_NAME]
}

fn open_db(dir: &aptos_temppath::TempPath) -> RksDB {
	let mut db_opts = rocksdb::Options::default();
	db_opts.create_if_missing(true);
	db_opts.create_missing_column_families(true);
	RksDB::open(dir.path(), "test", get_column_families(), &db_opts).expect("Failed to open DB.")
}

struct TestDB {
	_tmpdir: aptos_temppath::TempPath,
	db: RksDB,
}

impl TestDB {
	fn new() -> Self {
		let tmpdir = aptos_temppath::TempPath::new();
		let db = open_db(&tmpdir);

		TestDB {
			_tmpdir: tmpdir,
			db,
		}
	}
}

impl std::ops::Deref for TestDB {
	type Target = RksDB;

	fn deref(&self) -> &Self::Target {
		&self.db
	}
}

fn main() {
	let key = TestKey(1, 2);
	let value = TestValue(1, "hello".to_string(), true);

	let db = TestDB::new();
	db.put::<TestSchema>(&key, &value).unwrap();

	assert_eq!(db.get::<TestSchema>(&key).unwrap(), Some(value),);
}
