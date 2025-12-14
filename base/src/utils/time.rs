use crate::map_err;
use crate::result::{AppResult, SysErr};

/// Get current Unix timestamp in seconds
///
/// Returns SystemTimeError when clock issues occur (e.g., rollback).
/// This is critical; callers should handle appropriately.
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
		// Validate timestamp is reasonable (> 2020-01-01)
		assert!(timestamp > 1577836800); // 2020-01-01 00:00:00 UTC
		// Validate timestamp is less than a future bound (e.g., 2030)
		assert!(timestamp < 1893456000); // 2030-01-01 00:00:00 UTC
	}

	#[test]
	fn test_unix_timestamp_consistency() {
		let ts1 = unix_timestamp().expect("Failed to get first timestamp");
		std::thread::sleep(std::time::Duration::from_millis(10));
		let ts2 = unix_timestamp().expect("Failed to get second timestamp");

		// Ensure time is monotonically increasing
		assert!(ts2 >= ts1);
		// Ensure delta is within a reasonable bound (< 1s)
		assert!(ts2 - ts1 < 1);
	}
}
