use crate::schemadb::RksDB;
use base_infra::{result::AppResult, runtimes::Tokio};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::mpsc,
    time::{sleep, Instant},
};
use tracing::{error, info, warn};

/// TTL清理调度器配置
#[derive(Debug, Clone)]
pub struct TtlScheduleConfig {
    /// 清理任务执行间隔（秒）
    pub cleanup_interval_seconds: u64,
    /// 是否启用定时清理
    pub enable_cleanup: bool,
    /// 每次清理的最大处理数量
    pub max_cleanup_batch_size: usize,
}

impl Default for TtlScheduleConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_seconds: 300, // 默认5分钟清理一次
            enable_cleanup: true,
            max_cleanup_batch_size: 1000,
        }
    }
}

/// TTL定时清理调度器
pub struct RksdbTtlScheduler {
    db: Arc<RksDB>,
    config: TtlScheduleConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    is_running: Arc<AtomicBool>,
}

impl RksdbTtlScheduler {
    /// 创建新的TTL调度器
    pub fn new(db: Arc<RksDB>, config: TtlScheduleConfig) -> Self {
        Self {
            db,
            config,
            shutdown_tx: None,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启动TTL清理定时任务
    pub fn start(&mut self) -> AppResult<()> {
        if !self.config.enable_cleanup {
            info!("TTL cleanup is disabled, skipping scheduler start");
            return Ok(());
        }

        if self.is_running.load(Ordering::SeqCst) {
            warn!("TTL scheduler is already running");
            return Ok(());
        }

        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);
        self.is_running.store(true, Ordering::SeqCst);

        let db = Arc::clone(&self.db);
        let config = self.config.clone();
        let is_running = Arc::clone(&self.is_running);

        // 启动后台清理任务
        Tokio.spawn(async move {
            Self::cleanup_task(db, config, shutdown_rx, is_running).await;
        });

        info!(
            "TTL scheduler started with interval: {} seconds",
            self.config.cleanup_interval_seconds
        );

        Ok(())
    }

    /// 停止TTL清理定时任务
    pub async fn stop(&mut self) -> AppResult<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            info!("TTL scheduler is not running");
            return Ok(());
        }

        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if let Err(e) = shutdown_tx.send(()).await {
                warn!("Failed to send shutdown signal: {}", e);
            }
        }

        // 等待任务停止
        let start_time = Instant::now();
        let timeout = Duration::from_secs(10); // 10秒超时

        while self.is_running.load(Ordering::SeqCst) && start_time.elapsed() < timeout {
            sleep(Duration::from_millis(100)).await;
        }

        if self.is_running.load(Ordering::SeqCst) {
            warn!("TTL scheduler failed to stop within timeout");
        } else {
            info!("TTL scheduler stopped successfully");
        }

        Ok(())
    }

    /// 检查调度器是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// 立即执行一次清理任务
    pub fn trigger_cleanup(&self) -> AppResult<u64> {
        let current_time = super::current_timestamp();
        self.db.cleanup_expired(current_time)?;
        Ok(current_time)
    }

    /// 后台清理任务的主循环
    async fn cleanup_task(
        db: Arc<RksDB>,
        config: TtlScheduleConfig,
        mut shutdown_rx: mpsc::Receiver<()>,
        is_running: Arc<AtomicBool>,
    ) {
        let interval = Duration::from_secs(config.cleanup_interval_seconds);
        let mut next_cleanup = Instant::now() + interval;

        info!("TTL cleanup task started");

        loop {
            // 检查是否收到停止信号
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal, stopping TTL cleanup task");
                    break;
                }
                _ = sleep(Duration::from_millis(100)) => {
                    // 继续检查是否到达下次清理时间
                }
            }

            // 检查是否到达清理时间
            if Instant::now() >= next_cleanup {
                let cleanup_start = Instant::now();
                let current_time = super::current_timestamp();

                match db.cleanup_expired(current_time) {
                    Ok(()) => {
                        let cleanup_duration = cleanup_start.elapsed();
                        info!(
                            "TTL cleanup completed in {:?} for timestamp: {}",
                            cleanup_duration, current_time
                        );
                    }
                    Err(e) => {
                        error!("TTL cleanup failed: {}", e);
                    }
                }

                // 设置下次清理时间
                next_cleanup = Instant::now() + interval;
            }
        }

        is_running.store(false, Ordering::SeqCst);
        info!("TTL cleanup task stopped");
    }
}

impl Drop for RksdbTtlScheduler {
    fn drop(&mut self) {
        if self.is_running.load(Ordering::SeqCst) {
            warn!("TTL scheduler is being dropped while still running");
            // 注意：这里不能使用异步方法，只能发送停止信号
            if let Some(shutdown_tx) = &self.shutdown_tx {
                let _ = shutdown_tx.try_send(());
            }
        }
    }
}

/// TTL调度器管理器，用于管理多个数据库的TTL清理
pub struct RksdbTtlSchedulerManager {
    schedulers: Vec<RksdbTtlScheduler>,
}

impl RksdbTtlSchedulerManager {
    /// 创建新的调度器管理器
    pub fn new() -> Self {
        Self {
            schedulers: Vec::new(),
        }
    }

    /// 添加数据库的TTL调度器
    pub fn add_scheduler(&mut self, db: Arc<RksDB>, config: TtlScheduleConfig) {
        let scheduler = RksdbTtlScheduler::new(db, config);
        self.schedulers.push(scheduler);
    }

    /// 启动所有调度器
    pub fn start_all(&mut self) -> AppResult<()> {
        for scheduler in &mut self.schedulers {
            scheduler.start()?;
        }
        info!("Started {} TTL schedulers", self.schedulers.len());
        Ok(())
    }

    /// 停止所有调度器
    pub async fn stop_all(&mut self) -> AppResult<()> {
        for scheduler in &mut self.schedulers {
            scheduler.stop().await?;
        }
        info!("Stopped {} TTL schedulers", self.schedulers.len());
        Ok(())
    }

    /// 触发所有调度器立即执行清理
    pub fn trigger_all_cleanup(&self) -> AppResult<Vec<u64>> {
        let mut results = Vec::new();
        for scheduler in &self.schedulers {
            let timestamp = scheduler.trigger_cleanup()?;
            results.push(timestamp);
        }
        Ok(results)
    }

    /// 获取运行中的调度器数量
    pub fn running_count(&self) -> usize {
        self.schedulers
            .iter()
            .filter(|scheduler| scheduler.is_running())
            .count()
    }

    /// 检查所有调度器是否都在运行
    pub fn all_running(&self) -> bool {
        !self.schedulers.is_empty()
            && self
                .schedulers
                .iter()
                .all(|scheduler| scheduler.is_running())
    }
}

impl Default for RksdbTtlSchedulerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemadb::Schema;
    use base_infra::codec::bincode::{BinDecodeExt, BinEncodeExt};
    use bincode::{Decode, Encode};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
    pub struct TestKey(i32);

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
    pub struct TestValue(String);

    crate::define_schema!(TestSchema, TestKey, TestValue, "test_schema");
    crate::impl_schema_bin_codec!(TestSchema, TestKey, TestValue);

    async fn create_test_db() -> Arc<RksDB> {
        use rocksdb::Options;

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let mut column_families = vec![TestSchema::COLUMN_FAMILY_NAME];
        column_families.extend(RksDB::get_ttl_column_families());

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let db = RksDB::open(path, "test_db", column_families, &opts).unwrap();
        Arc::new(db)
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let db = create_test_db().await;
        let config = TtlScheduleConfig {
            cleanup_interval_seconds: 1,
            enable_cleanup: true,
            max_cleanup_batch_size: 100,
        };

        let mut scheduler = RksdbTtlScheduler::new(db, config);

        // 启动调度器
        scheduler.start().unwrap();
        assert!(scheduler.is_running());

        // 等待一段时间
        sleep(Duration::from_millis(500)).await;
        assert!(scheduler.is_running());

        // 停止调度器
        scheduler.stop().await.unwrap();
        assert!(!scheduler.is_running());
    }

    #[tokio::test]
    async fn test_scheduler_cleanup() {
        let db = create_test_db().await;
        let config = TtlScheduleConfig {
            cleanup_interval_seconds: 1,
            enable_cleanup: true,
            max_cleanup_batch_size: 100,
        };

        // 写入一些过期数据
        let key = TestKey(1);
        let value = TestValue("test".to_string());
        let past_time = super::super::current_timestamp() - 10;

        db.put_with_ttl::<TestSchema>(&key, &value, past_time)
            .unwrap();

        let scheduler = RksdbTtlScheduler::new(Arc::clone(&db), config);

        // 触发立即清理
        let cleanup_time = scheduler.trigger_cleanup().unwrap();
        assert!(cleanup_time > 0);

        // 验证数据是否被清理
        let result = db.get_check_ttl::<TestSchema>(&key).unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_scheduler_manager() {
        let db1 = create_test_db().await;
        let db2 = create_test_db().await;

        let config = TtlScheduleConfig {
            cleanup_interval_seconds: 2,
            enable_cleanup: true,
            max_cleanup_batch_size: 100,
        };

        let mut manager = RksdbTtlSchedulerManager::new();
        manager.add_scheduler(db1, config.clone());
        manager.add_scheduler(db2, config);

        // 启动所有调度器
        manager.start_all().unwrap();
        assert_eq!(manager.running_count(), 2);
        assert!(manager.all_running());

        // 等待一段时间
        sleep(Duration::from_millis(500)).await;

        // 停止所有调度器
        manager.stop_all().await.unwrap();
        assert_eq!(manager.running_count(), 0);
        assert!(!manager.all_running());
    }

    #[tokio::test]
    async fn test_disabled_scheduler() {
        let db = create_test_db().await;
        let config = TtlScheduleConfig {
            cleanup_interval_seconds: 1,
            enable_cleanup: false, // 禁用清理
            max_cleanup_batch_size: 100,
        };

        let mut scheduler = RksdbTtlScheduler::new(db, config);

        // 启动调度器（应该不会真正启动）
        scheduler.start().unwrap();
        assert!(!scheduler.is_running());

        // 停止调度器
        scheduler.stop().await.unwrap();
        assert!(!scheduler.is_running());
    }
}
