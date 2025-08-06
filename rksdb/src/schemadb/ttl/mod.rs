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

/// TTL 过期索引的 Key，使用 (expire_timestamp, schema_name, original_key) 作为复合Key
/// 这样可以按时间排序扫描过期数据，同时能够根据原始key删除对应的过期索引
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TtlExpirationKey {
    /// 过期时间戳（Unix时间戳，秒）
    pub expire_timestamp: u64,
    /// Schema名称，用于区分不同类型的数据
    pub schema_name: String,
    /// 原始的key（序列化后的字节）
    pub original_key: Vec<u8>,
}

/// TTL 过期索引的 Value，存储必要的元数据
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TtlExpirationValue {
    /// 原始数据存储的列族名称
    pub cf_name: String,
}

/// TTL 单个Key的过期时间索引，使用 (schema_name, original_key) 作为Key
/// 用于快速查询单个key的过期时间，避免全表扫描
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TtlSingleKey {
    /// Schema名称
    pub schema_name: String,
    /// 原始的key（序列化后的字节）
    pub original_key: Vec<u8>,
}

/// TTL 单个Key的过期时间值
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct TtlSingleValue {
    /// 过期时间戳（Unix时间戳，秒）
    pub expire_timestamp: u64,
    /// 原始数据存储的列族名称
    pub cf_name: String,
}

// 定义过期索引的Schema
crate::define_pub_schema!(
    TtlExpirationSchema,
    TtlExpirationKey,
    TtlExpirationValue,
    "ttl_expiration"
);

// 定义单个Key过期时间索引的Schema
crate::define_pub_schema!(
    TtlSingleSchema,
    TtlSingleKey,
    TtlSingleValue,
    "ttl_single"
);

// 为过期索引Schema实现编码
crate::impl_schema_bin_codec!(TtlExpirationSchema, TtlExpirationKey, TtlExpirationValue);

// 为单个Key索引Schema实现编码
crate::impl_schema_bin_codec!(TtlSingleSchema, TtlSingleKey, TtlSingleValue);

impl RksDB {
    /// 写入带TTL的数据
    ///
    /// # 参数
    /// - `key`: 要存储的键
    /// - `value`: 要存储的值
    /// - `expire_at`: 过期时间戳（Unix时间戳，秒）
    ///
    /// # 错误
    /// - 如果 `expire_at` 小于当前时间，返回错误
    /// - 如果序列化失败，返回错误
    /// - 如果数据库写入失败，返回错误
    pub fn put_with_ttl<S: Schema>(
        &self,
        key: &S::Key,
        value: &S::Value,
        expire_at: u64,
    ) -> AppResult<()> {
        // 验证过期时间
        let current_time = current_timestamp();
        if expire_at <= current_time {
            return Err(crate::errors::RksDbError::Other(
                format!("TTL expire_at ({}) must be greater than current time ({})",
                       expire_at, current_time)
            ).into());
        }

        let batch = SchemaBatch::new();

        // 1. 写入原始数据
        batch.put::<S>(key, value)?;

        // 2. 序列化key用于TTL索引
        let original_key_bytes = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let schema_name = std::any::type_name::<S>().to_string();

        // 3. 写入过期时间索引（用于按时间排序扫描）
        let ttl_expiration_key = TtlExpirationKey {
            expire_timestamp: expire_at,
            schema_name: schema_name.clone(),
            original_key: original_key_bytes.clone(),
        };
        let ttl_expiration_value = TtlExpirationValue {
            cf_name: S::COLUMN_FAMILY_NAME.to_string(),
        };
        batch.put::<TtlExpirationSchema>(&ttl_expiration_key, &ttl_expiration_value)?;

        // 4. 写入单个Key的过期时间索引（用于快速查询）
        let ttl_single_key = TtlSingleKey {
            schema_name,
            original_key: original_key_bytes,
        };
        let ttl_single_value = TtlSingleValue {
            expire_timestamp: expire_at,
            cf_name: S::COLUMN_FAMILY_NAME.to_string(),
        };
        batch.put::<TtlSingleSchema>(&ttl_single_key, &ttl_single_value)?;

        // 原子性写入所有数据
        self.write_schemas(batch)
    }

    /// 读取数据并检查TTL，如果过期则自动删除
    ///
    /// # 返回值
    /// - `Some(value)`: 数据存在且未过期
    /// - `None`: 数据不存在或已过期
    pub fn get_check_ttl<S: Schema>(&self, schema_key: &S::Key) -> AppResult<Option<S::Value>> {
        // 1. 检查TTL
        let original_key_bytes = <S::Key as KeyCodec<S>>::encode_key(schema_key)?;
        let ttl_single_key = TtlSingleKey {
            schema_name: std::any::type_name::<S>().to_string(),
            original_key: original_key_bytes,
        };

        if let Some(ttl_single_value) = self.get::<TtlSingleSchema>(&ttl_single_key)? {
            let current_time = current_timestamp();
            if ttl_single_value.expire_timestamp <= current_time {
                // 数据已过期，删除它
                self.delete_expired::<S>(schema_key)?;
                return Ok(None);
            }
        }

        // 2. 数据未过期或没有TTL，正常读取
        self.get::<S>(schema_key)
    }

    /// 手动删除已过期的数据
    ///
    /// # 参数
    /// - `key`: 要检查和删除的键
    pub fn delete_expired<S: Schema>(&self, key: &S::Key) -> AppResult<()> {
        let original_key_bytes = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let schema_name = std::any::type_name::<S>().to_string();

        // 检查是否有TTL记录
        let ttl_single_key = TtlSingleKey {
            schema_name: schema_name.clone(),
            original_key: original_key_bytes.clone(),
        };

        if let Some(ttl_single_value) = self.get::<TtlSingleSchema>(&ttl_single_key)? {
            let batch = SchemaBatch::new();

            // 1. 删除原始数据
            batch.delete::<S>(key)?;

            // 2. 删除过期时间索引
            let ttl_expiration_key = TtlExpirationKey {
                expire_timestamp: ttl_single_value.expire_timestamp,
                schema_name,
                original_key: original_key_bytes,
            };
            batch.delete::<TtlExpirationSchema>(&ttl_expiration_key)?;

            // 3. 删除单个Key的过期时间索引
            batch.delete::<TtlSingleSchema>(&ttl_single_key)?;

            // 原子性删除
            self.write_schemas(batch)?;
        }

        Ok(())
    }

    /// 后台定时任务调用，清理过期数据
    ///
    /// # 参数
    /// - `current_time`: 当前时间戳，用于判断数据是否过期
    /// - `max_batch_size`: 单次处理的最大数据量，默认1000
    pub fn cleanup_expired(&self, current_time: u64) -> AppResult<()> {
        self.cleanup_expired_with_batch_size(current_time, 1000)
    }

    /// 后台定时任务调用，清理过期数据（可指定批量大小）
    ///
    /// # 参数
    /// - `current_time`: 当前时间戳
    /// - `max_batch_size`: 单次处理的最大数据量
    pub fn cleanup_expired_with_batch_size(&self, current_time: u64, max_batch_size: usize) -> AppResult<()> {
        if max_batch_size == 0 {
            return Err(crate::errors::RksDbError::Other(
                "max_batch_size must be greater than 0".to_string()
            ).into());
        }

        // 扫描所有过期的数据，使用前向迭代器按时间顺序扫描
        let mut iter = self.iter::<TtlExpirationSchema>()?;
        iter.seek_to_first();

        let mut expired_keys = Vec::with_capacity(max_batch_size);
        let mut total_cleaned = 0;

        // 收集所有过期的key
        while let Some((expiration_key, expiration_value)) = iter.next().transpose()? {
            // 如果时间戳大于当前时间，后续的记录都不会过期，直接跳出
            if expiration_key.expire_timestamp > current_time {
                break;
            }

            expired_keys.push((expiration_key, expiration_value));

            // 批量处理，避免内存占用过高
            if expired_keys.len() >= max_batch_size {
                self.batch_delete_expired(&expired_keys)?;
                total_cleaned += expired_keys.len();
                expired_keys.clear();
            }
        }

        // 处理剩余的过期数据
        if !expired_keys.is_empty() {
            self.batch_delete_expired(&expired_keys)?;
            total_cleaned += expired_keys.len();
        }

        if total_cleaned > 0 {
            tracing::debug!("Cleaned {} expired TTL entries", total_cleaned);
        }

        Ok(())
    }

    /// 批量删除过期数据
    fn batch_delete_expired(
        &self,
        expired_keys: &[(TtlExpirationKey, TtlExpirationValue)],
    ) -> AppResult<()> {
        if expired_keys.is_empty() {
            return Ok(());
        }

        let batch = SchemaBatch::new();

        for (expiration_key, expiration_value) in expired_keys {
            // 1. 删除过期索引记录
            batch.delete::<TtlExpirationSchema>(expiration_key)?;

            // 2. 删除单个Key的过期时间索引
            let ttl_single_key = TtlSingleKey {
                schema_name: expiration_key.schema_name.clone(),
                original_key: expiration_key.original_key.clone(),
            };
            batch.delete::<TtlSingleSchema>(&ttl_single_key)?;

            // 3. 删除原始数据（使用底层的rocksdb接口）
            self.delete_raw_key(&expiration_value.cf_name, &expiration_key.original_key)?;
        }

        self.write_schemas(batch)
    }

    /// 删除原始数据的底层方法
    fn delete_raw_key(&self, cf_name: &str, key_bytes: &[u8]) -> AppResult<()> {
        let cf_handle = self.get_cf_handle(cf_name)?;
        self.inner.delete_cf(cf_handle, key_bytes)
            .map_err(crate::errors::RksDbError::from)?;
        Ok(())
    }

    /// 获取所有列族名称，包含TTL相关的列族
    pub fn get_ttl_column_families() -> Vec<ColumnFamilyName> {
        vec![
            TtlExpirationSchema::COLUMN_FAMILY_NAME,
            TtlSingleSchema::COLUMN_FAMILY_NAME,
        ]
    }

    /// 获取TTL统计信息
    ///
    /// # 返回值
    /// 返回 (总TTL记录数, 过期记录数)
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

/// 获取当前Unix时间戳（秒）
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 计算N秒后的时间戳
pub fn timestamp_after_seconds(seconds: u64) -> u64 {
    current_timestamp() + seconds
}

/// 计算N分钟后的时间戳
pub fn timestamp_after_minutes(minutes: u64) -> u64 {
    current_timestamp() + (minutes * 60)
}

/// 计算N小时后的时间戳
pub fn timestamp_after_hours(hours: u64) -> u64 {
    current_timestamp() + (hours * 3600)
}

/// 计算N天后的时间戳
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

        // 写入带TTL的数据
        db.put_with_ttl::<TestSchema>(&key, &value, expire_at).unwrap();

        // 读取数据，应该能正常获取
        let result = db.get_check_ttl::<TestSchema>(&key).unwrap();
        assert_eq!(result, Some(value));
    }

    #[test]
    fn test_expired_data_removal() {
        let db = create_test_db();
        let key = TestKey(1, 2);
        let value = TestValue(1, "hello".to_string(), true);
        let past_time = current_timestamp() - 10; // 10秒前的时间

        // 写入已过期的数据
        db.put_with_ttl::<TestSchema>(&key, &value, past_time).unwrap();

        // 读取数据，应该返回None（因为已过期）
        let result = db.get_check_ttl::<TestSchema>(&key).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_cleanup_expired() {
        let db = create_test_db();
        let key = TestKey(1, 2);
        let value = TestValue(1, "hello".to_string(), true);
        let past_time = current_timestamp() - 10;

        // 写入已过期的数据
        db.put_with_ttl::<TestSchema>(&key, &value, past_time).unwrap();

        // 调用清理函数
        db.cleanup_expired(current_timestamp()).unwrap();

        // 验证过期索引是否被清理
        let ttl_single_key = TtlSingleKey {
            schema_name: std::any::type_name::<TestSchema>().to_string(),
            original_key: key.bin_encode().unwrap(),
        };
        let result = db.get::<TtlSingleSchema>(&ttl_single_key).unwrap();
        assert_eq!(result, None);
    }
}