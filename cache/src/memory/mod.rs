mod cache;

use crate::error::CacheErr;
use crate::schema::{CacheTtl, KeyCodec, Schema, ValueCodec};
use base_infra::else_err;
use base_infra::result::AppResult;
pub use cache::*;
use moka::future::Cache;
use std::sync::LazyLock;

pub type BytesCache = moka::sync::Cache<Vec<u8>, Vec<u8>>;
pub type AsyncBytesCache = Cache<Vec<u8>, Vec<u8>>;

static ASYNC_TTL_CACHE: LazyLock<moka::sync::Cache<CacheTtl, AsyncBytesCache>> =
    LazyLock::new(|| moka::sync::Cache::builder().max_capacity(100).build());

pub(crate) struct TtlBytesCache(CacheTtl);

impl TtlBytesCache {
    pub fn new(ttl: CacheTtl) -> Self {
        Self(ttl)
    }

    pub fn insert(&self, cache: AsyncBytesCache) {
        (&ASYNC_TTL_CACHE).insert(self.0, cache)
    }

    pub fn get(&self) -> Option<AsyncBytesCache> {
        (&ASYNC_TTL_CACHE).get(&self.0)
    }
}

pub trait MemCache {
    fn cache<S: Schema>(&self) -> AppResult<BytesCache>;
}

#[async_trait::async_trait]
pub trait AsyncMemCache {
    fn ttl(&self) -> CacheTtl;

    fn async_cache<S: Schema>(&self) -> AppResult<AsyncBytesCache> {
        let ttl = self.ttl();
        TtlBytesCache::new(ttl)
            .get()
            .ok_or_else(else_err!(&CacheErr::CacheNotInit, format!("{:?}", ttl)))
    }

    async fn async_store<S: Schema>(&self, key: &S::Key, value: &S::Value) -> AppResult<()> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let value = <S::Value as ValueCodec<S>>::encode_value(value)?;

        self.async_cache::<S>()?.insert(key, value).await;
        Ok(())
    }

    async fn async_load<S: Schema>(&self, key: &S::Key) -> AppResult<Option<S::Value>> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let value = self.async_cache::<S>()?.get(&key).await;
        let res = value.map(|v| <S::Value as ValueCodec<S>>::decode_value(&v));
        Ok(res.transpose()?)
    }

    async fn async_remove<S: Schema>(&self, key: &S::Key) -> AppResult<()> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        self.async_cache::<S>()?.remove(&key).await;
        Ok(())
    }
}
