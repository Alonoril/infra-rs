use bincode::{Decode, Encode};
use rksdb_infra::{
    schemadb::{
        schema::Schema,
        ttl::{
            schedule::{TtlScheduleConfig, RksdbTtlScheduler},
            timestamp_after_minutes, timestamp_after_seconds, current_timestamp
        },
        RksDB,
    },
};
use rocksdb::Options;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tempfile::TempDir;

// Define business data structure
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
struct UserSessionKey {
    user_id: u64,
    session_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
struct UserSessionValue {
    access_token: String,
    last_activity: u64,
    device_info: String,
}

// Define schema
rksdb_infra::define_schema!(
    UserSessionSchema,
    UserSessionKey,
    UserSessionValue,
    "user_sessions"
);

// Implement codec
rksdb_infra::impl_schema_bin_codec!(UserSessionSchema, UserSessionKey, UserSessionValue);

fn create_db_with_ttl() -> Result<Arc<RksDB>, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path().to_path_buf();
    
    // Get all required column families, including TTL ones
    let mut column_families = vec![UserSessionSchema::COLUMN_FAMILY_NAME];
    column_families.extend(RksDB::get_ttl_column_families());
    
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    
    let db = RksDB::open(path, "ttl_example_db", column_families, &opts)?;
    
    Ok(Arc::new(db))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (simplified)
    // tracing_subscriber::fmt::init();
    
    println!("=== RocksDB TTL Feature Example ===\n");
    
    // Create database
    let db = create_db_with_ttl()?;
    println!("1. Database created successfully");
    
    // Create user session data
    let session_key = UserSessionKey {
        user_id: 12345,
        session_id: "sess_abc123".to_string(),
    };
    
    let session_value = UserSessionValue {
        access_token: "token_xyz789".to_string(),
        last_activity: current_timestamp(),
        device_info: "iPhone 15 Pro".to_string(),
    };
    
    // Set session to expire in 5 seconds
    let expire_time = timestamp_after_seconds(5);
    db.put_with_ttl::<UserSessionSchema>(&session_key, &session_value, expire_time)?;
    println!("2. Write session data that expires in 5 seconds");
    
    // Read immediately (should succeed)
    let result = db.get_check_ttl::<UserSessionSchema>(&session_key)?;
    match result {
        Some(value) => println!("3. Immediate read success: {:?}", value),
        None => println!("3. Immediate read failed"),
    }
    
    // Wait 3 seconds (not expired yet)
    println!("4. Waiting 3 seconds...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let result = db.get_check_ttl::<UserSessionSchema>(&session_key)?;
    match result {
        Some(value) => println!("5. Read after 3s success: {:?}", value),
        None => println!("5. Read after 3s failed (expired)"),
    }
    
    // Wait another 3 seconds (6s total, should be expired)
    println!("6. Waiting another 3 seconds...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let result = db.get_check_ttl::<UserSessionSchema>(&session_key)?;
    match result {
        Some(value) => println!("7. Read after 6s success: {:?}", value),
        None => println!("7. Read after 6s failed (expired)"),
    }
    
    // Demonstrate scheduled cleanup
    println!("\n=== Scheduled cleanup demo ===");
    
    // Write multiple expired entries
    for i in 0..5 {
        let key = UserSessionKey {
            user_id: 1000 + i,
            session_id: format!("expired_sess_{}", i),
        };
        let value = UserSessionValue {
            access_token: format!("expired_token_{}", i),
            last_activity: current_timestamp() - 10, // expired 10 seconds ago
            device_info: "Test Device".to_string(),
        };
        let past_time = current_timestamp() - 5; // expired 5 seconds ago
        db.put_with_ttl::<UserSessionSchema>(&key, &value, past_time)?;
    }
    println!("8. Wrote 5 expired session entries");
    
    // Configure and start TTL cleanup scheduler
    let config = TtlScheduleConfig {
        cleanup_interval_seconds: 2, // clean every 2 seconds
        enable_cleanup: true,
        max_cleanup_batch_size: 100,
    };
    
    let mut scheduler = RksdbTtlScheduler::new(Arc::clone(&db), config);
    scheduler.start()?;
    println!("9. TTL scheduler started, cleaning every 2 seconds");
    
    // Wait for cleanup to complete
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // Trigger an immediate cleanup
    let cleanup_time = scheduler.trigger_cleanup()?;
    println!("10. Manual cleanup triggered, timestamp: {}", cleanup_time);
    
    // Write some valid data (expires in 10 minutes)
    for i in 0..3 {
        let key = UserSessionKey {
            user_id: 2000 + i,
            session_id: format!("valid_sess_{}", i),
        };
        let value = UserSessionValue {
            access_token: format!("valid_token_{}", i),
            last_activity: current_timestamp(),
            device_info: "Valid Device".to_string(),
        };
        let future_time = timestamp_after_minutes(10);
        db.put_with_ttl::<UserSessionSchema>(&key, &value, future_time)?;
    }
    println!("11. Wrote 3 valid session entries (expire in 10 minutes)");
    
    // Verify valid data can be read
    let valid_key = UserSessionKey {
        user_id: 2000,
        session_id: "valid_sess_0".to_string(),
    };
    let result = db.get_check_ttl::<UserSessionSchema>(&valid_key)?;
    match result {
        Some(value) => println!("12. Valid data read success: {:?}", value),
        None => println!("12. Valid data read failed"),
    }
    
    // Stop scheduler
    scheduler.stop().await?;
    println!("13. TTL scheduler stopped");
    
    println!("\n=== TTL feature demo complete ===");
    
    Ok(())
} 
