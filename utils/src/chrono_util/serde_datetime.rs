use chrono::{DateTime, NaiveDateTime};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

const MILLIS_THRESHOLD: i64 = 1_000_000_000_000;

/// Includes implementations of serialization and deserialization from timestamps (e.g. 1_734_947_195).
pub mod serde_naive_datetime {
    use super::*;

    pub fn serialize<S>(value: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(value.and_utc().timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = TimestampInput::deserialize(deserializer)?;
        input.into_naive_datetime()
    }
}

/// Includes implementations of serialization and deserialization from optional timestamps such as 1734947195
pub mod serde_option_naive_datetime {
    use super::*;

    pub fn serialize<S>(value: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(dt) => serializer.serialize_some(&dt.and_utc().timestamp()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<TimestampInput>::deserialize(deserializer)?;
        match opt {
            Some(TimestampInput::String(s)) if s.trim().is_empty() => Ok(None),
            Some(input) => input.into_naive_datetime().map(Some),
            None => Ok(None),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TimestampInput {
    Int(i64),
    String(String),
}

impl TimestampInput {
    fn into_naive_datetime<E>(self) -> Result<NaiveDateTime, E>
    where
        E: DeError,
    {
        match self {
            TimestampInput::Int(ts) => timestamp_to_naive_datetime(ts),
            TimestampInput::String(value) => parse_datetime_string(&value),
        }
    }
}

fn parse_datetime_string<E>(value: &str) -> Result<NaiveDateTime, E>
where
    E: DeError,
{
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DeError::custom("empty datetime string"));
    }

    if let Ok(ts) = trimmed.parse::<i64>() {
        return timestamp_to_naive_datetime(ts);
    }

    NaiveDateTime::from_str(trimmed)
        .map_err(|_| DeError::custom(format!("invalid datetime format: {trimmed}")))
}

fn timestamp_to_naive_datetime<E>(timestamp: i64) -> Result<NaiveDateTime, E>
where
    E: DeError,
{
    let datetime = if timestamp.abs() >= MILLIS_THRESHOLD {
        DateTime::from_timestamp_millis(timestamp)
    } else {
        DateTime::from_timestamp(timestamp, 0)
    }
    .map(|dt| dt.naive_utc());

    datetime.ok_or_else(|| DeError::custom(format!("invalid unix timestamp: {timestamp}")))
}
