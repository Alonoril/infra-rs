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

// 定义业务数据结构
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

// 定义Schema
rksdb_infra::define_schema!(
    UserSessionSchema,
    UserSessionKey,
    UserSessionValue,
    "user_sessions"
);

// 实现编码
rksdb_infra::impl_schema_bin_codec!(UserSessionSchema, UserSessionKey, UserSessionValue);

fn create_db_with_ttl() -> Result<Arc<RksDB>, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path().to_path_buf();
    
    // 获取所有需要的列族，包括TTL相关的列族
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
    // 初始化日志（简化版本）
    // tracing_subscriber::fmt::init();
    
    println!("=== RocksDB TTL 功能示例 ===\n");
    
    // 创建数据库
    let db = create_db_with_ttl()?;
    println!("1. 数据库创建成功");
    
    // 创建用户会话数据
    let session_key = UserSessionKey {
        user_id: 12345,
        session_id: "sess_abc123".to_string(),
    };
    
    let session_value = UserSessionValue {
        access_token: "token_xyz789".to_string(),
        last_activity: current_timestamp(),
        device_info: "iPhone 15 Pro".to_string(),
    };
    
    // 设置5秒后过期的会话数据
    let expire_time = timestamp_after_seconds(5);
    db.put_with_ttl::<UserSessionSchema>(&session_key, &session_value, expire_time)?;
    println!("2. 写入会话数据，将在5秒后过期");
    
    // 立即读取数据（应该能正常获取）
    let result = db.get_check_ttl::<UserSessionSchema>(&session_key)?;
    match result {
        Some(value) => println!("3. 立即读取成功: {:?}", value),
        None => println!("3. 立即读取失败"),
    }
    
    // 等待3秒（数据尚未过期）
    println!("4. 等待3秒...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let result = db.get_check_ttl::<UserSessionSchema>(&session_key)?;
    match result {
        Some(value) => println!("5. 3秒后读取成功: {:?}", value),
        None => println!("5. 3秒后读取失败（数据已过期）"),
    }
    
    // 等待另外3秒（总计6秒，数据应该过期）
    println!("6. 再等待3秒...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let result = db.get_check_ttl::<UserSessionSchema>(&session_key)?;
    match result {
        Some(value) => println!("7. 6秒后读取成功: {:?}", value),
        None => println!("7. 6秒后读取失败（数据已过期）"),
    }
    
    // 演示定时清理功能
    println!("\n=== 定时清理功能演示 ===");
    
    // 写入多个过期数据
    for i in 0..5 {
        let key = UserSessionKey {
            user_id: 1000 + i,
            session_id: format!("expired_sess_{}", i),
        };
        let value = UserSessionValue {
            access_token: format!("expired_token_{}", i),
            last_activity: current_timestamp() - 10, // 10秒前过期
            device_info: "Test Device".to_string(),
        };
        let past_time = current_timestamp() - 5; // 5秒前过期
        db.put_with_ttl::<UserSessionSchema>(&key, &value, past_time)?;
    }
    println!("8. 写入5个已过期的会话数据");
    
    // 配置并启动TTL清理调度器
    let config = TtlScheduleConfig {
        cleanup_interval_seconds: 2, // 2秒清理一次
        enable_cleanup: true,
        max_cleanup_batch_size: 100,
    };
    
    let mut scheduler = RksdbTtlScheduler::new(Arc::clone(&db), config);
    scheduler.start()?;
    println!("9. TTL清理调度器已启动，每2秒清理一次");
    
    // 等待清理完成
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // 立即触发一次清理
    let cleanup_time = scheduler.trigger_cleanup()?;
    println!("10. 手动触发清理完成，清理时间戳: {}", cleanup_time);
    
    // 写入一些有效数据（10分钟后过期）
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
    println!("11. 写入3个有效的会话数据（10分钟后过期）");
    
    // 验证有效数据是否能正常读取
    let valid_key = UserSessionKey {
        user_id: 2000,
        session_id: "valid_sess_0".to_string(),
    };
    let result = db.get_check_ttl::<UserSessionSchema>(&valid_key)?;
    match result {
        Some(value) => println!("12. 有效数据读取成功: {:?}", value),
        None => println!("12. 有效数据读取失败"),
    }
    
    // 停止调度器
    scheduler.stop().await?;
    println!("13. TTL清理调度器已停止");
    
    println!("\n=== TTL 功能演示完成 ===");
    
    Ok(())
} 