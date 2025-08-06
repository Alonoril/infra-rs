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

// 定义默认超时时间
pub const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_millis(500);

#[macro_export]
macro_rules! cacheable_with_lock {
    (($key:expr, $biz_name:expr), ($cache:expr, $schema:ty), $fetch:expr) => {{
        use base::cache::lock::CacheError;
        use std::time::Duration;
        use tokio::sync::Mutex;
        use tokio::time::timeout;

        // 尝试从缓存中读取数据
        match $cache.async_load::<$schema>($key).await {
            Ok(Some(cached_value)) => {
                tracing::info!("{}: cache hit", $biz_name);
                return Ok(cached_value);
            }
            Ok(None) => {}
            Err(e) => return Err(CacheError::CacheOperation(e).into()),
        }

        // 获取基于键的细粒度锁（带超时）
        let mutex_map = $crate::cache::lock::CACHE_MUTEX_MAP.clone();
        let key_str = format!("{:?}", $key);
        let mutex = {
            let mut map = mutex_map.lock().await;
            map.entry(key_str.clone())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        // 尝试获取锁（带超时机制）
        let timeout_ms = $crate::cache::lock::DEFAULT_LOCK_TIMEOUT;
        let _guard = match timeout(timeout_ms, mutex.lock()).await {
            Ok(guard) => guard,
            Err(_) => return Err(CacheError::LockTimeout(key_str.clone()).into()),
        };

        // 二次缓存检查
        match $cache.async_load::<$schema>($key).await {
            Ok(Some(cached_value)) => {
                tracing::warn!("{}: cache hit (after lock)", $biz_name);
                return Ok(cached_value);
            }
            Ok(None) => {}
            Err(e) => return Err(CacheError::CacheOperation(e).into()),
        }

        // 执行数据获取
        let result = $fetch.await?; //.map_err(|e| CacheError::DataFetch(e))?;

        $cache
            .async_store::<$schema>($key, &result)
            .await
            .map_err(|e| CacheError::CacheOperation(e))?;

        // 清理未使用的锁
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

// 自定义错误类型（示例）
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache operation failed: {0}")]
    CacheOperation(#[source] BaseError),

    #[error("Failed to acquire lock for key: {0}")]
    LockTimeout(String),

    #[error("Data fetch failed: {0}")]
    DataFetch(#[source] BaseError),
}

// #[macro_export]
// macro_rules! cacheable_with_lock {
//     // ($cache:expr, $key:expr, $param:expr, $fetch:expr) => {{
//     (($key:expr, $biz_name:expr),($cache:expr, $schema:ty), $fetch:expr) => {{
//         // 尝试从缓存中读取数据
//         if let Some(cached_value) = $cache.async_load::<$schema>($key).await? {
//             tracing::info!("{}: cache hit", $biz_name);
//             return Ok(cached_value);
//         }
//
//         // 获取基于缓存键的细粒度锁
//         let mutex_map = crate::CACHE_MUTEX_MAP.clone();
//         let key_str = format!("{:?}", $key); // 将缓存键转换为字符串
//         let mutex = {
//             let mut map = mutex_map.lock().await;
//             map.entry(key_str.clone())
//                 .or_insert_with(|| Arc::new(Mutex::new(())))
//                 .clone()
//         };
//         let _guard = mutex.lock().await;
//
//         // 再次检查缓存
//         if let Some(cached_value) = $cache.async_load::<$schema>($key).await? {
//             tracing::warn!("{}: cache hit (after lock)", $biz_name);
//             return Ok(cached_value);
//         }
//
//         // 缓存未命中，调用数据获取函数
//         let result = $fetch.await?;
//
//         // 将数据存入缓存
//         $cache.async_store::<$schema>($key, &result).await?;
//
//         // 清理锁（可选）
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

// pub async fn with_cache<F, G, K, V, E>(
//     cache_key: K,                                       // 缓存键
//     cache_load: F,                                      // 缓存加载逻辑
//     cache_store: G,                                     // 缓存存储逻辑
//     business_logic: impl Future<Output = Result<V, E>>, // 业务逻辑
// ) -> Result<V, E>
// where
//     F: Fn(&K) -> Pin<Box<dyn Future<Output = Result<Option<V>, E>> + Send>>, // 缓存加载函数
//     G: Fn(&K, &V) -> Pin<Box<dyn Future<Output = Result<(), E>> + Send>>,    // 缓存存储函数
//     K: Clone + Debug + Send + Sync + 'static, // 缓存键需要实现这些 trait
//     V: Clone + Debug + Send + Sync + 'static, // 缓存值需要实现这些 trait
//     E: From<BaseError> + Send + Sync + 'static, // 错误类型需要支持从缓存错误转换
// {
//     // 尝试从缓存加载数据
//     if let Some(cached_value) = cache_load(&cache_key).await? {
//         info!("Cache hit for key: {:?}", cache_key);
//         return Ok(cached_value);
//     }
//
//     // 缓存未命中，执行业务逻辑
//     info!(
//         "Cache miss for key: {:?}, executing business logic",
//         cache_key
//     );
//     let result = business_logic.await?;
//
//     // 将结果存储到缓存中
//     cache_store(&cache_key, &result).await?;
//     info!("Cache stored for key: {:?}", cache_key);
//
//     Ok(result)
// }
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use std::collections::HashMap;
//
// // 全局缓存加载锁
// type CacheMutex<K, V> = Arc<Mutex<HashMap<K, Arc<tokio::sync::Notify>>>>;
//
// pub async fn with_cache_mutex<F, G, K, V, E>(
//     cache_key: K, // 缓存键
//     cache_load: F, // 缓存加载逻辑
//     cache_store: G, // 缓存存储逻辑
//     business_logic: impl Future<Output = Result<V, E>>, // 业务逻辑
//     cache_mutex: CacheMutex<K, V>, // 全局缓存加载锁
// ) -> Result<V, E>
// where
//     F: Fn(&K) -> Future<Output = Result<Option<V>, E>>, // 缓存加载函数
//     G: Fn(&K, &V) -> Future<Output = Result<(), E>>, // 缓存存储函数
//     K: Clone + Debug + Eq + Hash + Send + Sync + 'static, // 缓存键需要实现这些 trait
//     V: Clone + Debug + Send + Sync + 'static, // 缓存值需要实现这些 trait
//     E: From<CacheError> + Send + Sync + 'static, // 错误类型需要支持从缓存错误转换
// {
//     // 尝试从缓存加载数据
//     if let Some(cached_value) = cache_load(&cache_key).await? {
//         info!("Cache hit for key: {:?}", cache_key);
//         return Ok(cached_value);
//     }
//
//     // 缓存未命中，获取锁
//     let notify = {
//         let mut map = cache_mutex.lock().await;
//         map.entry(cache_key.clone())
//             .or_insert_with(|| Arc::new(tokio::sync::Notify::new()))
//             .clone()
//     };
//
//     // 检查是否已经有其他请求在加载数据
//     {
//         let map = cache_mutex.lock().await;
//         if let Some(existing_notify) = map.get(&cache_key) {
//             info!("Waiting for another request to load data for key: {:?}", cache_key);
//             existing_notify.notified().await; // 等待其他请求完成
//             drop(map); // 释放锁
//
//             // 再次尝试从缓存加载数据
//             if let Some(cached_value) = cache_load(&cache_key).await? {
//                 info!("Cache hit after waiting for key: {:?}", cache_key);
//                 return Ok(cached_value);
//             }
//         }
//     }
//
//     // 当前请求负责加载数据
//     info!("Cache miss for key: {:?}, executing business logic", cache_key);
//     let result = business_logic.await?;
//
//     // 将结果存储到缓存中
//     cache_store(&cache_key, &result).await?;
//     info!("Cache stored for key: {:?}", cache_key);
//
//     // 通知其他等待的请求
//     notify.notify_waiters();
//
//     // 清理锁
//     {
//         let mut map = cache_mutex.lock().await;
//         map.remove(&cache_key);
//     }
//
//     Ok(result)
// }
