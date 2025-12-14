use crate::schemadb::RksDB;
use base_infra::{result::AppResult, runtimes::Tokio};
use std::{
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};
use tokio::{
	sync::mpsc,
	time::{Instant, sleep},
};
use tracing::{error, info, warn};

/// TTL cleanup scheduler config
#[derive(Debug, Clone)]
pub struct TtlScheduleConfig {
	/// Cleanup interval in seconds
	pub cleanup_interval_seconds: u64,
	/// Whether to enable periodic cleanup
	pub enable_cleanup: bool,
	/// Max items processed per cleanup
	pub max_cleanup_batch_size: usize,
}

impl Default for TtlScheduleConfig {
	fn default() -> Self {
		Self {
			cleanup_interval_seconds: 300, // Default: clean every 5 minutes
			enable_cleanup: true,
			max_cleanup_batch_size: 1000,
		}
	}
}

/// TTL periodic cleanup scheduler
pub struct RksdbTtlScheduler {
	db: Arc<RksDB>,
	config: TtlScheduleConfig,
	shutdown_tx: Option<mpsc::Sender<()>>,
	is_running: Arc<AtomicBool>,
}

impl RksdbTtlScheduler {
	/// Create a new TTL scheduler
	pub fn new(db: Arc<RksDB>, config: TtlScheduleConfig) -> Self {
		Self {
			db,
			config,
			shutdown_tx: None,
			is_running: Arc::new(AtomicBool::new(false)),
		}
	}

	/// Start TTL cleanup background job
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

		// Start background cleanup task
		Tokio.spawn(async move {
			Self::cleanup_task(db, config, shutdown_rx, is_running).await;
		});

		info!(
			"TTL scheduler started with interval: {} seconds",
			self.config.cleanup_interval_seconds
		);

		Ok(())
	}

	/// Stop TTL cleanup background job
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

		// Wait for task to stop
		let start_time = Instant::now();
		let timeout = Duration::from_secs(10); // 10s timeout

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

	/// Check whether scheduler is running
	pub fn is_running(&self) -> bool {
		self.is_running.load(Ordering::SeqCst)
	}

	/// Trigger an immediate cleanup run
	pub fn trigger_cleanup(&self) -> AppResult<u64> {
		let current_time = super::current_timestamp();
		self.db.cleanup_expired(current_time)?;
		Ok(current_time)
	}

	/// Main loop for background cleanup task
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
			// Check for stop signal
			tokio::select! {
				_ = shutdown_rx.recv() => {
					info!("Received shutdown signal, stopping TTL cleanup task");
					break;
				}
				_ = sleep(Duration::from_millis(100)) => {
					// Continue to check if next cleanup time is reached
				}
			}

			// Check whether it's time to clean
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

				// Set next cleanup time
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
			// Note: cannot use async methods here; only send stop signal
			if let Some(shutdown_tx) = &self.shutdown_tx {
				let _ = shutdown_tx.try_send(());
			}
		}
	}
}

/// TTL scheduler manager to manage multiple DB's cleanup
pub struct RksdbTtlSchedulerManager {
	schedulers: Vec<RksdbTtlScheduler>,
}

impl RksdbTtlSchedulerManager {
	/// Create a new scheduler manager
	pub fn new() -> Self {
		Self {
			schedulers: Vec::new(),
		}
	}

	/// Add a DB's TTL scheduler
	pub fn add_scheduler(&mut self, db: Arc<RksDB>, config: TtlScheduleConfig) {
		let scheduler = RksdbTtlScheduler::new(db, config);
		self.schedulers.push(scheduler);
	}

	/// Start all schedulers
	pub fn start_all(&mut self) -> AppResult<()> {
		for scheduler in &mut self.schedulers {
			scheduler.start()?;
		}
		info!("Started {} TTL schedulers", self.schedulers.len());
		Ok(())
	}

	/// Stop all schedulers
	pub async fn stop_all(&mut self) -> AppResult<()> {
		for scheduler in &mut self.schedulers {
			scheduler.stop().await?;
		}
		info!("Stopped {} TTL schedulers", self.schedulers.len());
		Ok(())
	}

	/// Trigger immediate cleanup on all schedulers
	pub fn trigger_all_cleanup(&self) -> AppResult<Vec<u64>> {
		let mut results = Vec::new();
		for scheduler in &self.schedulers {
			let timestamp = scheduler.trigger_cleanup()?;
			results.push(timestamp);
		}
		Ok(results)
	}

	/// Get count of running schedulers
	pub fn running_count(&self) -> usize {
		self.schedulers
			.iter()
			.filter(|scheduler| scheduler.is_running())
			.count()
	}

	/// Check if all schedulers are running
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
	use tokio::time::{Duration, sleep};

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

		// Start scheduler
		scheduler.start().unwrap();
		assert!(scheduler.is_running());

		// Wait for a while
		sleep(Duration::from_millis(500)).await;
		assert!(scheduler.is_running());

		// Stop scheduler
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

		// Write some expired data
		let key = TestKey(1);
		let value = TestValue("test".to_string());
		let past_time = super::super::current_timestamp() - 10;

		db.put_with_ttl::<TestSchema>(&key, &value, past_time)
			.unwrap();

		let scheduler = RksdbTtlScheduler::new(Arc::clone(&db), config);

		// Trigger immediate cleanup
		let cleanup_time = scheduler.trigger_cleanup().unwrap();
		assert!(cleanup_time > 0);

		// Verify data is cleaned
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

		// Start all schedulers
		manager.start_all().unwrap();
		assert_eq!(manager.running_count(), 2);
		assert!(manager.all_running());

		// Wait for a while
		sleep(Duration::from_millis(500)).await;

		// Stop all schedulers
		manager.stop_all().await.unwrap();
		assert_eq!(manager.running_count(), 0);
		assert!(!manager.all_running());
	}

	#[tokio::test]
	async fn test_disabled_scheduler() {
		let db = create_test_db().await;
		let config = TtlScheduleConfig {
			cleanup_interval_seconds: 1,
			enable_cleanup: false, // Disable cleanup
			max_cleanup_batch_size: 100,
		};

		let mut scheduler = RksdbTtlScheduler::new(db, config);

		// Start scheduler (should not actually start)
		scheduler.start().unwrap();
		assert!(!scheduler.is_running());

		// Stop scheduler
		scheduler.stop().await.unwrap();
		assert!(!scheduler.is_running());
	}
}
