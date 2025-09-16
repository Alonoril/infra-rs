use crate::schemadb::{
    schema::{KeyCodec, Schema},
    ColumnFamilyName, RksDB, SchemaBatch,
};
use base_infra::result::AppResult;
use base_infra::codec::bincode::{BinDecodeExt, BinEncodeExt};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod schedule;

/// TTL expiration index Key uses (expire_timestamp, schema_name, original_key) as composite key
/// Enables scanning by time and deleting expiration index by original key
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TtlExpirationKey {
    /// Expiration timestamp (Unix, seconds)
    pub expire_timestamp: u64,
    /// Schema name, to distinguish data types
    pub schema_name: String,
    /// Original key (serialized bytes)
    pub original_key: Vec<u8>,
}

/// TTL expiration index Value, stores required metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TtlExpirationValue {
    /// Column family name where original data is stored
    pub cf_name: String,
}

/// TTL per-key expiration index uses (schema_name, original_key) as the key
/// Used to quickly query expiration time for a single key to avoid full scans
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TtlSingleKey {
    /// Schema name
    pub schema_name: String,
    /// Original key (serialized bytes)
    pub original_key: Vec<u8>,
}

/// TTL expiration value for a single key
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TtlSingleValue {
    /// Expiration timestamp (Unix timestamp, seconds)
    pub expire_timestamp: u64,
    /// Column family name where original data is stored
    pub cf_name: String,
}

// Define schema for expiration index
crate::define_pub_schema!(
    TtlExpirationSchema,
    TtlExpirationKey,
    TtlExpirationValue,
    "ttl_expiration"
);

// Define schema for single key expiration time index
crate::define_pub_schema!(
    TtlSingleSchema,
    TtlSingleKey,
    TtlSingleValue,
    "ttl_single"
);

// Implement encoding for expiration index schema
crate::impl_schema_bin_codec!(TtlExpirationSchema, TtlExpirationKey, TtlExpirationValue);

// Implement encoding for single key index schema
crate::impl_schema_bin_codec!(TtlSingleSchema, TtlSingleKey, TtlSingleValue);

impl RksDB {
    /// Write data with TTL
    ///
    /// # Parameters
    /// - `key`: Key to store
    /// - `value`: Value to store
    /// - `expire_at`: Expiration timestamp (Unix seconds)
    ///
    /// # Errors
    /// - Returns error if `expire_at` < now
    /// - Returns error on serialization failure
    /// - Returns error on DB write failure
    pub fn put_with_ttl<S: Schema>(
        &self,
        key: &S::Key,
        value: &S::Value,
        expire_at: u64,
    ) -> AppResult<()> {
        // Validate expiration time
        let current_time = current_timestamp();
        if expire_at <= current_time {
            return Err(crate::errors::RksDbError::Other(
                format!("TTL expire_at ({}) must be greater than current time ({})",
                       expire_at, current_time)
            ).into());
        }

        let batch = SchemaBatch::new();

        // 1. Write original data
        batch.put::<S>(key, value)?;

        // 2. Serialize key for TTL index
        let original_key_bytes = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let schema_name = std::any::type_name::<S>().to_string();

        // 3. Write expiration time index (for time-ordered scan)
        let ttl_expiration_key = TtlExpirationKey {
            expire_timestamp: expire_at,
            schema_name: schema_name.clone(),
            original_key: original_key_bytes.clone(),
        };
        let ttl_expiration_value = TtlExpirationValue {
            cf_name: S::COLUMN_FAMILY_NAME.to_string(),
        };
        batch.put::<TtlExpirationSchema>(&ttl_expiration_key, &ttl_expiration_value)?;

        // 4. Write per-key expiration index (for quick lookup)
        let ttl_single_key = TtlSingleKey {
            schema_name,
            original_key: original_key_bytes,
        };
        let ttl_single_value = TtlSingleValue {
            expire_timestamp: expire_at,
            cf_name: S::COLUMN_FAMILY_NAME.to_string(),
        };
        batch.put::<TtlSingleSchema>(&ttl_single_key, &ttl_single_value)?;

        // Atomically write all data
        self.write_schemas(batch)
    }

    /// Read data and check TTL; auto-delete if expired
    ///
    /// # Returns
    /// - `Some(value)`: Data exists and is not expired
    /// - `None`: Data does not exist or is expired
    pub fn get_check_ttl<S: Schema>(&self, schema_key: &S::Key) -> AppResult<Option<S::Value>> {
        // 1. Check TTL
        let original_key_bytes = <S::Key as KeyCodec<S>>::encode_key(schema_key)?;
        let ttl_single_key = TtlSingleKey {
            schema_name: std::any::type_name::<S>().to_string(),
            original_key: original_key_bytes,
        };

        if let Some(ttl_single_value) = self.get::<TtlSingleSchema>(&ttl_single_key)? {
            let current_time = current_timestamp();
            if ttl_single_value.expire_timestamp <= current_time {
                // Data expired; delete it
                self.delete_expired::<S>(schema_key)?;
                return Ok(None);
            }
        }

        // 2. Not expired or no TTL; return value
        self.get::<S>(schema_key)
    }

    /// Manually delete expired data
    ///
    /// # Parameters
    /// - `key`: The key to check and delete
    pub fn delete_expired<S: Schema>(&self, key: &S::Key) -> AppResult<()> {
        let original_key_bytes = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let schema_name = std::any::type_name::<S>().to_string();

        // Check if there is a TTL record
        let ttl_single_key = TtlSingleKey {
            schema_name: schema_name.clone(),
            original_key: original_key_bytes.clone(),
        };

        if let Some(ttl_single_value) = self.get::<TtlSingleSchema>(&ttl_single_key)? {
            let batch = SchemaBatch::new();

            // 1. Delete original data
            batch.delete::<S>(key)?;

            // 2. Delete expiration time index
            let ttl_expiration_key = TtlExpirationKey {
                expire_timestamp: ttl_single_value.expire_timestamp,
                schema_name,
                original_key: original_key_bytes,
            };
            batch.delete::<TtlExpirationSchema>(&ttl_expiration_key)?;

            // 3. Delete per-key expiration index
            batch.delete::<TtlSingleSchema>(&ttl_single_key)?;

            // Atomic delete
            self.write_schemas(batch)?;
        }

        Ok(())
    }

    /// Called by background scheduler to clean up expired data
    ///
    /// # Parameters
    /// - `current_time`: Current timestamp to determine expiration
    /// - `max_batch_size`: Max batch size per run, default 1000
    pub fn cleanup_expired(&self, current_time: u64) -> AppResult<()> {
        self.cleanup_expired_with_batch_size(current_time, 1000)
    }

    /// Background cleanup with configurable batch size
    ///
    /// # Parameters
    /// - `current_time`: Current timestamp
    /// - `max_batch_size`: Max items processed per run
    pub fn cleanup_expired_with_batch_size(&self, current_time: u64, max_batch_size: usize) -> AppResult<()> {
        if max_batch_size == 0 {
            return Err(crate::errors::RksDbError::Other(
                "max_batch_size must be greater than 0".to_string()
            ).into());
        }

        // Scan all expired data in order using a forward iterator
        let mut iter = self.iter::<TtlExpirationSchema>()?;
        iter.seek_to_first();

        let mut expired_keys = Vec::with_capacity(max_batch_size);
        let mut total_cleaned = 0;

        // Collect all expired keys
        while let Some((expiration_key, expiration_value)) = iter.next().transpose()? {
            // If timestamp > now, later records won't be expired; break
            if expiration_key.expire_timestamp > current_time {
                break;
            }

            expired_keys.push((expiration_key, expiration_value));

            // Process in batches to avoid high memory usage
            if expired_keys.len() >= max_batch_size {
                self.batch_delete_expired(&expired_keys)?;
                total_cleaned += expired_keys.len();
                expired_keys.clear();
            }
        }

        // Process any remaining expired data
        if !expired_keys.is_empty() {
            self.batch_delete_expired(&expired_keys)?;
            total_cleaned += expired_keys.len();
        }

        if total_cleaned > 0 {
            tracing::debug!("Cleaned {} expired TTL entries", total_cleaned);
        }

        Ok(())
    }

    /// Delete expired data in batch
    fn batch_delete_expired(
        &self,
        expired_keys: &[(TtlExpirationKey, TtlExpirationValue)],
    ) -> AppResult<()> {
        if expired_keys.is_empty() {
            return Ok(());
        }

        let batch = SchemaBatch::new();

        for (expiration_key, expiration_value) in expired_keys {
            // 1. Delete expiration index records
            batch.delete::<TtlExpirationSchema>(expiration_key)?;

            // 2. Delete per-key expiration index
            let ttl_single_key = TtlSingleKey {
                schema_name: expiration_key.schema_name.clone(),
                original_key: expiration_key.original_key.clone(),
            };
            batch.delete::<TtlSingleSchema>(&ttl_single_key)?;

            // 3. Delete original data (via low-level RocksDB API)
            self.delete_raw_key(&expiration_value.cf_name, &expiration_key.original_key)?;
        }

        self.write_schemas(batch)
    }

    /// Low-level method to delete original data
    fn delete_raw_key(&self, cf_name: &str, key_bytes: &[u8]) -> AppResult<()> {
        let cf_handle = self.get_cf_handle(cf_name)?;
        self.inner.delete_cf(cf_handle, key_bytes)
            .map_err(crate::errors::RksDbError::from)?;
        Ok(())
    }

    /// Get all column family names including TTL-related ones
    pub fn get_ttl_column_families() -> Vec<ColumnFamilyName> {
        vec![
            TtlExpirationSchema::COLUMN_FAMILY_NAME,
            TtlSingleSchema::COLUMN_FAMILY_NAME,
        ]
    }

    /// Get TTL statistics
    ///
    /// # Returns
    /// (total TTL records, expired records)
    pub fn get_ttl_stats(&self) -> AppResult<(usize, usize)> {
        let current_time = current_timestamp();
        let mut total_count = 0;
        let mut expired_count = 0;

        let mut iter = self.iter::<TtlExpirationSchema>()?;
        iter.seek_to_first();

        while let Some((expiration_key, _)) = iter.next().transpose()? {
            total_count += 1;
            if expiration_key.expire_timestamp <= current_time {
                expired_count += 1;
            }
        }

        Ok((total_count, expired_count))
    }
}

/// Get current Unix timestamp (seconds)
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Compute timestamp after N seconds
pub fn timestamp_after_seconds(seconds: u64) -> u64 {
    current_timestamp() + seconds
}

/// Compute timestamp after N minutes
pub fn timestamp_after_minutes(minutes: u64) -> u64 {
    current_timestamp() + (minutes * 60)
}

/// Compute timestamp after N hours
pub fn timestamp_after_hours(hours: u64) -> u64 {
    current_timestamp() + (hours * 3600)
}

/// Compute timestamp after N days
pub fn timestamp_after_days(days: u64) -> u64 {
    current_timestamp() + (days * 86400)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemadb::schema::Schema;
    use base_infra::codec::bincode::{BinDecodeExt, BinEncodeExt};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
    pub struct TestKey(i32, i32);

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
    pub struct TestValue(i32, String, bool);

    crate::define_schema!(TestSchema, TestKey, TestValue, "test_schema");
    crate::impl_schema_bin_codec!(TestSchema, TestKey, TestValue);

    fn create_test_db() -> RksDB {
        use rocksdb::Options;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let mut column_families = vec![TestSchema::COLUMN_FAMILY_NAME];
        column_families.extend(RksDB::get_ttl_column_families());

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        RksDB::open(path, "test_db", column_families, &opts).unwrap()
    }

    #[test]
    fn test_put_and_get_with_ttl() {
        let db = create_test_db();
        let key = TestKey(1, 2);
        let value = TestValue(1, "hello".to_string(), true);
        let expire_at = timestamp_after_seconds(10);

        // Write data with TTL
        db.put_with_ttl::<TestSchema>(&key, &value, expire_at).unwrap();

        // Read data should succeed
        let result = db.get_check_ttl::<TestSchema>(&key).unwrap();
        assert_eq!(result, Some(value));
    }

    #[test]
    fn test_expired_data_removal() {
        let db = create_test_db();
        let key = TestKey(1, 2);
        let value = TestValue(1, "hello".to_string(), true);
        let past_time = current_timestamp() - 10; // 10 seconds ago

        // Write already-expired data
        db.put_with_ttl::<TestSchema>(&key, &value, past_time).unwrap();

        // Read should return None (expired)
        let result = db.get_check_ttl::<TestSchema>(&key).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_cleanup_expired() {
        let db = create_test_db();
        let key = TestKey(1, 2);
        let value = TestValue(1, "hello".to_string(), true);
        let past_time = current_timestamp() - 10;

        // Write already-expired data
        db.put_with_ttl::<TestSchema>(&key, &value, past_time).unwrap();

        // Call cleanup
        db.cleanup_expired(current_timestamp()).unwrap();

        // Verify expiration index cleaned
        let ttl_single_key = TtlSingleKey {
            schema_name: std::any::type_name::<TestSchema>().to_string(),
            original_key: key.bin_encode().unwrap(),
        };
        let result = db.get::<TtlSingleSchema>(&ttl_single_key).unwrap();
        assert_eq!(result, None);
    }
}
