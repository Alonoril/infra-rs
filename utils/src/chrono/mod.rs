use crate::error::UtlErr;
use base_infra::nar_err;
use base_infra::result::AppResult;
use chrono::{DateTime, NaiveDateTime};

pub mod date_util;
pub mod serde_datetime;

const MILLIS_THRESHOLD: i64 = 1_000_000_000_000;

pub fn ts_to_naive_datetime(timestamp: i64) -> AppResult<NaiveDateTime> {
	let datetime = if timestamp.abs() >= MILLIS_THRESHOLD {
		DateTime::from_timestamp_millis(timestamp)
	} else {
		DateTime::from_timestamp(timestamp, 0)
	}
	.map(|dt| dt.naive_utc());

	datetime.ok_or_else(nar_err!(&UtlErr::InvalidTimestamp, timestamp))
}

