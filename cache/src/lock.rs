use crate::error::BaseError;
use std::fmt::Debug;
use std::future::Future;
use tracing::info;

use std::boxed::Box;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::Mutex;

pub static CACHE_MUTEX_MAP: LazyLock<Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

// Define default timeout
pub const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_millis(500);

#[macro_export]
macro_rules! cacheable_with_lock {
    (($key:expr, $biz_name:expr), ($cache:expr, $schema:ty), $fetch:expr) => {{
        use base::cache::lock::CacheError;
        use std::time::Duration;
        use tokio::sync::Mutex;
        use tokio::time::timeout;

        // Try reading from cache
        match $cache.async_load::<$schema>($key).await {
            Ok(Some(cached_value)) => {
                tracing::info!("{}: cache hit", $biz_name);
                return Ok(cached_value);
            }
            Ok(None) => {}
            Err(e) => return Err(CacheError::CacheOperation(e).into()),
        }

        // Acquire fine-grained lock by key (with timeout)
        let mutex_map = $crate::cache::lock::CACHE_MUTEX_MAP.clone();
        let key_str = format!("{:?}", $key);
        let mutex = {
            let mut map = mutex_map.lock().await;
            map.entry(key_str.clone())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        // Try acquiring the lock (with timeout)
        let timeout_ms = $crate::cache::lock::DEFAULT_LOCK_TIMEOUT;
        let _guard = match timeout(timeout_ms, mutex.lock()).await {
            Ok(guard) => guard,
            Err(_) => return Err(CacheError::LockTimeout(key_str.clone()).into()),
        };

        // Second cache check
        match $cache.async_load::<$schema>($key).await {
            Ok(Some(cached_value)) => {
                tracing::warn!("{}: cache hit (after lock)", $biz_name);
                return Ok(cached_value);
            }
            Ok(None) => {}
            Err(e) => return Err(CacheError::CacheOperation(e).into()),
        }

        // Execute data fetch
        let result = $fetch.await?; //.map_err(|e| CacheError::DataFetch(e))?;

        $cache
            .async_store::<$schema>($key, &result)
            .await
            .map_err(|e| CacheError::CacheOperation(e))?;

        // Cleanup unused locks
        let mut map = mutex_map.lock().await;
        if map
            .get(&key_str)
            .map(|m| std::sync::Arc::strong_count(m) == 1)
            .unwrap_or(false)
        {
            map.remove(&key_str);
        }

        Ok(result)
    }};
}

// Custom error type (example)
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache operation failed: {0}")]
    CacheOperation(#[source] BaseError),

    #[error("Failed to acquire lock for key: {0}")]
    LockTimeout(String),

    #[error("Data fetch failed: {0}")]
    DataFetch(#[source] BaseError),
}

#[macro_export]
macro_rules! cacheable {
    (($key:expr, $biz_name:expr),($cache:expr, $schema:ty), $fetch:expr) => {{
        if let Some(cached_value) = $cache.async_load::<$schema>($key).await? {
            tracing::info!("{}: cache hit", $biz_name);
            return Ok(cached_value);
        }

        let result = $fetch.await?;
        $cache.async_store::<$schema>($key, &result).await?;
        Ok(result)
    }};
}

pub async fn with_cache<CL, CS, K, V, E>(
    cache_key: K,
    cache_load: CL,
    cache_store: CS,
    business_logic: impl Future<Output = Result<V, E>>,
) -> Result<V, E>
where
    CL: Fn(K) -> Pin<Box<dyn Future<Output = Result<Option<V>, E>> + Send>>,
    CS: Fn(K, V) -> Pin<Box<dyn Future<Output = Result<(), E>> + Send>>,
    K: Clone + Debug + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
    E: From<BaseError> + Send + Sync + 'static,
{
    if let Some(cached_value) = cache_load(cache_key.clone()).await? {
        info!("Cache hit for key: {:?}", cache_key);
        return Ok(cached_value);
    }

    info!(
        "Cache miss for key: {:?}, executing business logic",
        cache_key
    );
    let result = business_logic.await?;

    cache_store(cache_key.clone(), result.clone()).await?;
    info!("Cache stored for key: {:?}", cache_key);

    Ok(result)
}

// #[macro_export]
// macro_rules! cacheable_with_lock {
//     // ($cache:expr, $key:expr, $param:expr, $fetch:expr) => {{
//     (($key:expr, $biz_name:expr),($cache:expr, $schema:ty), $fetch:expr) => {{
//         // Try to read data from cache
//         if let Some(cached_value) = $cache.async_load::<$schema>($key).await? {
//             tracing::info!("{}: cache hit", $biz_name);
//             return Ok(cached_value);
//         }
//
//         // Acquire fine-grained lock based on cache key
//         let mutex_map = crate::CACHE_MUTEX_MAP.clone();
//         let key_str = format!("{:?}", $key); // Convert cache key to string
//         let mutex = {
//             let mut map = mutex_map.lock().await;
//             map.entry(key_str.clone())
//                 .or_insert_with(|| Arc::new(Mutex::new(())))
//                 .clone()
//         };
//         let _guard = mutex.lock().await;
//
//         // Check cache again
//         if let Some(cached_value) = $cache.async_load::<$schema>($key).await? {
//             tracing::warn!("{}: cache hit (after lock)", $biz_name);
//             return Ok(cached_value);
//         }
//
//         // Cache miss, call data fetch function
//         let result = $fetch.await?;
//
//         // Store data in cache
//         $cache.async_store::<$schema>($key, &result).await?;
//
//         // Clean up lock (optional)
//         let mut map = mutex_map.lock().await;
//         if map
//             .get(&key_str)
//             .map(|m| Arc::strong_count(m) == 1)
//             .unwrap_or(false)
//         {
//             map.remove(&key_str);
//         }
//
//         Ok(result)
//     }};
// }


// pub async fn with_cache<F, G, K, V, E>(
//     cache_key: K,                                       // Cache key
//     cache_load: F,                                      // Cache loading logic
//     cache_store: G,                                     // Cache storing logic
//     business_logic: impl Future<Output = Result<V, E>>, // Business logic
// ) -> Result<V, E>
// where
//     F: Fn(&K) -> Pin<Box<dyn Future<Output = Result<Option<V>, E>> + Send>>, // Cache loading function
//     G: Fn(&K, &V) -> Pin<Box<dyn Future<Output = Result<(), E>> + Send>>,    // Cache storing function
//     K: Clone + Debug + Send + Sync + 'static, // Cache key needs to implement these traits
//     V: Clone + Debug + Send + Sync + 'static, // Cache value needs to implement these traits
//     E: From<BaseError> + Send + Sync + 'static, // Error type needs to support conversion from cache error
// {
//     // Try to load data from cache
//     if let Some(cached_value) = cache_load(&cache_key).await? {
//         info!("Cache hit for key: {:?}", cache_key);
//         return Ok(cached_value);
//     }
//
//     // Cache miss, execute business logic
//     info!(
//         "Cache miss for key: {:?}, executing business logic",
//         cache_key
//     );
//     let result = business_logic.await?;
//
//     // Store the result in cache
//     cache_store(&cache_key, &result).await?;
//     info!("Cache stored for key: {:?}", cache_key);
//
//     Ok(result)
// }
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use std::collections::HashMap;
//
// // Global cache loading lock
// type CacheMutex<K, V> = Arc<Mutex<HashMap<K, Arc<tokio::sync::Notify>>>>;
//
// pub async fn with_cache_mutex<F, G, K, V, E>(
//     cache_key: K, // Cache key
//     cache_load: F, // Cache loading logic
//     cache_store: G, // Cache storing logic
//     business_logic: impl Future<Output = Result<V, E>>, // Business logic
//     cache_mutex: CacheMutex<K, V>, // Global cache loading lock
// ) -> Result<V, E>
// where
//     F: Fn(&K) -> Future<Output = Result<Option<V>, E>>, // Cache loading function
//     G: Fn(&K, &V) -> Future<Output = Result<(), E>>, // Cache storing function
//     K: Clone + Debug + Eq + Hash + Send + Sync + 'static, // Cache key needs to implement these traits
//     V: Clone + Debug + Send + Sync + 'static, // Cache value needs to implement these traits
//     E: From<CacheError> + Send + Sync + 'static, // Error type needs to support conversion from cache error
// {
//     // Try to load data from cache
//     if let Some(cached_value) = cache_load(&cache_key).await? {
//         info!("Cache hit for key: {:?}", cache_key);
//         return Ok(cached_value);
//     }
//
//     // Cache miss, acquire lock
//     let notify = {
//         let mut map = cache_mutex.lock().await;
//         map.entry(cache_key.clone())
//             .or_insert_with(|| Arc::new(tokio::sync::Notify::new()))
//             .clone()
//     };
//
//     // Check if another request is already loading data
//     {
//         let map = cache_mutex.lock().await;
//         if let Some(existing_notify) = map.get(&cache_key) {
//             info!("Waiting for another request to load data for key: {:?}", cache_key);
//             existing_notify.notified().await; // Wait for other request to complete
//             drop(map); // Release lock
//
//             // Try to load data from cache again
//             if let Some(cached_value) = cache_load(&cache_key).await? {
//                 info!("Cache hit after waiting for key: {:?}", cache_key);
//                 return Ok(cached_value);
//             }
//         }
//     }
//
//     // Current request is responsible for loading data
//     info!("Cache miss for key: {:?}, executing business logic", cache_key);
//     let result = business_logic.await?;
//
//     // Store the result in cache
//     cache_store(&cache_key, &result).await?;
//     info!("Cache stored for key: {:?}", cache_key);
//
//     // Notify other waiting requests
//     notify.notify_waiters();
//
//     // Clean up lock
//     {
//         let mut map = cache_mutex.lock().await;
//         map.remove(&cache_key);
//     }
//
//     Ok(result)
// }
