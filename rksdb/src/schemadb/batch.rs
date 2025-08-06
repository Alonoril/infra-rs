use crate::schemadb::schema::{KeyCodec, Schema, ValueCodec};
use base_infra::result::AppResult;
use std::collections::HashMap;
use std::sync::Mutex;

pub type ColumnFamilyName = &'static str;

#[derive(Debug)]
pub enum WriteOp {
    Value { key: Vec<u8>, value: Vec<u8> },
    Deletion { key: Vec<u8> },
}

/// `SchemaBatch` holds a consolidate of updates that can be applied to a DB atomically. The updates
/// will be applied in the order in which they are added to the `SchemaBatch`.
#[derive(Debug)]
pub struct SchemaBatch {
    pub(crate) rows: Mutex<HashMap<ColumnFamilyName, Vec<WriteOp>>>,
}

impl Default for SchemaBatch {
    fn default() -> Self {
        Self {
            rows: Mutex::new(HashMap::new()),
        }
    }
}

impl SchemaBatch {
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an insert/update operation to the batch.
    pub fn put<S: Schema>(&self, key: &S::Key, value: &S::Value) -> AppResult<()> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let value = <S::Value as ValueCodec<S>>::encode_value(value)?;
        self.rows
            .lock()
            .unwrap()
            .entry(S::COLUMN_FAMILY_NAME)
            .or_default()
            .push(WriteOp::Value { key, value });

        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete<S: Schema>(&self, key: &S::Key) -> AppResult<()> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        self.rows
            .lock()
            .unwrap()
            .entry(S::COLUMN_FAMILY_NAME)
            .or_default()
            .push(WriteOp::Deletion { key });

        Ok(())
    }
}