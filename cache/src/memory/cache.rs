use crate::memory::{AsyncBytesCache, AsyncMemCache, TtlBytesCache};
use crate::schema::CacheTtl;
use moka::future::Cache;
use std::time::Duration;

pub struct SecondsMemCache;
impl SecondsMemCache {
	pub fn init_cache(&self) {
		let one_secs_cache: AsyncBytesCache = Cache::builder()
			.time_to_live(Duration::from_secs(1))
			.max_capacity(200)
			.build();

		TtlBytesCache::new(CacheTtl::OneSecond).insert(one_secs_cache);
	}
}

#[async_trait::async_trait]
impl AsyncMemCache for SecondsMemCache {
	fn ttl(&self) -> CacheTtl {
		CacheTtl::OneSecond
	}
}

pub struct Sec30MemCache;
impl Sec30MemCache {
	pub fn init_cache(&self) {
		let sec30_cache: AsyncBytesCache = Cache::builder()
			.time_to_live(Duration::from_secs(30))
			.max_capacity(1024)
			.build();

		TtlBytesCache::new(CacheTtl::Seconds(30)).insert(sec30_cache);
	}
}

#[async_trait::async_trait]
impl AsyncMemCache for Sec30MemCache {
	fn ttl(&self) -> CacheTtl {
		CacheTtl::Seconds(30)
	}
}

pub struct MinuteMemCache;
impl MinuteMemCache {
	pub fn init_cache(&self) {
		let one_minute_cache: AsyncBytesCache = Cache::builder()
			.time_to_live(Duration::from_secs(60))
			.max_capacity(200)
			.build();

		TtlBytesCache::new(CacheTtl::OneMinute).insert(one_minute_cache);
	}
}

#[async_trait::async_trait]
impl AsyncMemCache for MinuteMemCache {
	fn ttl(&self) -> CacheTtl {
		CacheTtl::OneMinute
	}
}

pub struct HourMemCache;
impl HourMemCache {
	pub fn init_cache(&self) {
		let one_hours_cache: AsyncBytesCache = Cache::builder()
			.time_to_live(Duration::from_secs(3600))
			.max_capacity(200)
			.build();

		TtlBytesCache::new(CacheTtl::OneHour).insert(one_hours_cache);
	}
}
#[async_trait::async_trait]
impl AsyncMemCache for HourMemCache {
	fn ttl(&self) -> CacheTtl {
		CacheTtl::OneHour
	}
}

pub struct NeverMemCache;
impl NeverMemCache {
	pub fn init_cache(&self) {
		let one_hours_cache: AsyncBytesCache = Cache::builder().max_capacity(1).build();
		TtlBytesCache::new(CacheTtl::Never).insert(one_hours_cache);
	}
}

#[async_trait::async_trait]
impl AsyncMemCache for NeverMemCache {
	fn ttl(&self) -> CacheTtl {
		CacheTtl::Never
	}
}
