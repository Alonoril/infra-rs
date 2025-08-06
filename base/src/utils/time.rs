use crate::result::{AppResult, SysErr};
use crate::map_err;

/// 获取当前 Unix 时间戳（秒）
/// 
/// 如果系统时间出现问题（如时钟回滚），会返回 SystemTimeError
/// 这是一个关键错误，调用方应该适当处理
pub fn unix_timestamp() -> AppResult<i64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(map_err!(&SysErr::SystemTimeError))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_timestamp() {
        let timestamp = unix_timestamp().expect("Failed to get timestamp");
        // 验证时间戳是一个合理的值（大于 2020年1月1日的时间戳）
        assert!(timestamp > 1577836800); // 2020-01-01 00:00:00 UTC
        // 验证时间戳小于某个未来时间（比如 2030年）
        assert!(timestamp < 1893456000); // 2030-01-01 00:00:00 UTC
    }

    #[test]
    fn test_unix_timestamp_consistency() {
        let ts1 = unix_timestamp().expect("Failed to get first timestamp");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = unix_timestamp().expect("Failed to get second timestamp");
        
        // 确保时间是递增的
        assert!(ts2 >= ts1);
        // 确保时间差在合理范围内（小于1秒）
        assert!(ts2 - ts1 < 1);
    }
}