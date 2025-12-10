use crate::error::UtlErr;
use base_infra::result::AppResult;
use base_infra::{else_err, err, map_err};
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use std::fmt::{Display, Formatter};

pub trait DiffDays<Rhs = Self> {
    fn diff_days(self, to: Rhs) -> i64;
}

impl DiffDays<NaiveDateTime> for NaiveDateTime {
    fn diff_days(self, to: NaiveDateTime) -> i64 {
        self.signed_duration_since(to).num_days().abs()
    }
}

impl DiffDays<NaiveDate> for NaiveDate {
    fn diff_days(self, to: NaiveDate) -> i64 {
        self.signed_duration_since(to).num_days().abs()
    }
}

impl DiffDays<DateTime<Utc>> for DateTime<Utc> {
    fn diff_days(self, to: DateTime<Utc>) -> i64 {
        self.signed_duration_since(to).num_days().abs()
    }
}

impl DiffDays<DateTime<Local>> for DateTime<Local> {
    fn diff_days(self, to: DateTime<Local>) -> i64 {
        self.signed_duration_since(to).num_days().abs()
    }
}

pub trait DateTimeUtcExt {
    fn utc_from_str(dt_str: &str) -> AppResult<DateTime<Utc>> {
        NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S")
            .map_err(map_err!(&UtlErr::StrToNaiveDt))
            .map(|dt| dt.and_utc())
    }

    fn utc_from_timestamp(secs: i64, nsecs: u32) -> AppResult<DateTime<Utc>> {
        DateTime::from_timestamp(secs, nsecs).ok_or_else(else_err!(&UtlErr::TimestampToDate))
    }

    fn utc_from_millis(millis: i64) -> AppResult<DateTime<Utc>> {
        DateTime::from_timestamp_millis(millis).ok_or_else(else_err!(&UtlErr::TimestampToDate))
    }

    fn utc_from_micros(micros: i64) -> AppResult<DateTime<Utc>> {
        DateTime::from_timestamp_micros(micros).ok_or_else(else_err!(&UtlErr::TimestampToDate))
    }
}

impl DateTimeUtcExt for DateTime<Utc> {}
impl DateTimeUtcExt for NaiveDateTime {}

pub trait LocalDateTimeExt {
    fn to_local_datetime(&self) -> AppResult<DateTime<Local>>;
}

impl LocalDateTimeExt for DateTime<Utc> {
    fn to_local_datetime(&self) -> AppResult<DateTime<Local>> {
        Ok(self.with_timezone(&Local))
    }
}

impl LocalDateTimeExt for &str {
    fn to_local_datetime(&self) -> AppResult<DateTime<Local>> {
        if let Ok(ldt) = self.parse::<DateTime<chrono::FixedOffset>>() {
            println!("ldt: {:?}", ldt);
            return Ok(ldt.with_timezone(&Local));
        }

        let naive = NaiveDateTime::parse_from_str(self, "%Y-%m-%d %H:%M:%S")
            .map_err(map_err!(&UtlErr::StrToNaiveDt))?;

        let dt_local = match Local.from_local_datetime(&naive) {
            chrono::LocalResult::Single(dt) => dt,
            // DST Callback: Select one
            chrono::LocalResult::Ambiguous(dt1, _dt2) => dt1,
            chrono::LocalResult::None => err!(&UtlErr::LocalDtNotExistDstGap)?,
        };
        Ok(dt_local)
    }
}

/// Truncate DateTime to unit (zeroed in hours/minutes/seconds/nanoseconds)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncUnit {
    Hour,
    Minute,
    Second,
    // Nanosecond,
}

impl Display for TruncUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TruncUnit::Hour => write!(f, "hour"),
            TruncUnit::Minute => write!(f, "minute"),
            TruncUnit::Second => write!(f, "second"),
            // TruncUnit::Nanosecond => write!(f, "truncate to nanosecond"),
        }
    }
}

pub fn truncate_timelike<T>(value: &T, unit: TruncUnit) -> AppResult<T>
where
    T: Timelike + Sized,
{
    let truncated = match unit {
        TruncUnit::Hour => value
            .with_minute(0)
            .and_then(|x| x.with_second(0))
            .and_then(|x| x.with_nanosecond(0)),
        TruncUnit::Minute => value.with_second(0).and_then(|x| x.with_nanosecond(0)),
        TruncUnit::Second => value.with_nanosecond(0),
    };

    truncated.ok_or_else(else_err!(&UtlErr::TruncateDateTime, unit))
}

pub trait TimelikeTruncate: Timelike + Sized {
    /// Truncate DateTime to unit (zeroed in hours/minutes/seconds/nanoseconds)
    fn truncate(&self, unit: TruncUnit) -> AppResult<Self> {
        truncate_timelike(self, unit)
    }
}

impl TimelikeTruncate for DateTime<Utc> {}
impl TimelikeTruncate for NaiveDateTime {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    #[test]
    fn test_to_local_datetime() {
        let s = "2025-12-07T10:30:00+08:00";
        println!("{:?}", s.to_local_datetime().unwrap());

        let s = "2025-12-07T02:30:00Z";
        println!("{:?}", s.to_local_datetime().unwrap());

        let s = "2025-12-07 10:30:00";
        println!("{:?}", s.to_local_datetime().unwrap());
    }

    #[test]
    fn test_from_str() {
        let dt_str = "2021-08-01 12:00:00";
        let dt = DateTime::utc_from_str(dt_str).unwrap();
        println!("{:?}", dt);

        let tz = FixedOffset::east_opt(8 * 3600).unwrap();
        println!("east8: {:?}", dt.with_timezone(&tz));

        assert_eq!(
            dt,
            NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .and_utc()
        );
    }

    #[test]
    fn test_from_timestamp() {
        // let secs = 1627840000;
        let secs = 1431648000;
        let nsecs = 0;
        let dt = DateTime::utc_from_timestamp(secs, nsecs).unwrap();
        println!("{:?}", dt);
    }

    #[test]
    fn test_from_timestamp_nanos() {
        let secs = 1627840000;
        let nsecs = 1001;
        let dt = NaiveDateTime::utc_from_timestamp(secs, nsecs).unwrap();
        println!("{:?}", dt);
    }

    #[test]
    fn test_from_timestamp_millis() {
        let millis = 1627840000_001;
        let dt = NaiveDateTime::utc_from_millis(millis).unwrap();
        println!("{:?}", dt);
    }

    #[test]
    fn test_from_timestamp_micros() {
        let micros = 1627840000_001_001;
        let dt = NaiveDateTime::utc_from_micros(micros).unwrap();
        println!("{:?}", dt);
    }
}
