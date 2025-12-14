pub mod error;
pub mod lock;
pub mod memory;
pub mod schema;
pub mod schema_codec;

use crate::memory::NeverMemCache;
use std::fmt;
use std::hash::Hash;
use tracing::info;

pub type BsResult<T> = Result<T, error::BaseError>;

#[async_trait::async_trait]
pub trait Cacheable<K, V>
where
	// K: PartialEq + Eq + Hash + PartialOrd + Ord + Clone + Send + Sync,
	// V: Clone + Send + Sync,
	K: fmt::Debug + Eq + Hash + Send + Sync + 'static,
	V: fmt::Debug + Clone + Send + Sync + 'static,
{
	async fn store(&self, key: K, value: V);

	async fn load(&self, key: &K) -> Option<V>;

	async fn remove(&self, key: &K);
	// async fn clear(&self);
}

pub fn init_cache() {
	info!("Start init memory cache");
	// SecondsMemCache.init_cache();
	// Secs30MemCache.init_cache();
	// MinuteMemCache.init_cache();
	// HourMemCache.init_cache();
	NeverMemCache.init_cache();
	info!("Init memory cache done");
}
