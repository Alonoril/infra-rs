use crate::memory::AsyncMemCache;
use base_infra::result::AppResult;
use std::fmt::Debug;

#[macro_export]
macro_rules! define_schema {
	($schema_type:ident, $key_type:ty, $value_type:ty, $cache:ty) => {
		#[derive(Debug)]
		pub(crate) struct $schema_type;

		impl $crate::schema::Schema for $schema_type {
			// const TTL: $crate::cache::schema::CacheTtl = $cache_ttl;
			type Cache = $cache;
			type Key = $key_type;
			type Value = $value_type;
		}
	};
}

#[macro_export]
macro_rules! define_pub_schema {
	($schema_type:ident, $key_type:ty, $value_type:ty, $cache:ty) => {
		#[derive(Debug)]
		pub struct $schema_type;

		impl $crate::schema::Schema for $schema_type {
			// const TTL: $crate::cache::schema::CacheTtl = $cache_ttl;
			type Cache = $cache;
			type Key = $key_type;
			type Value = $value_type;
		}
	};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheTtl {
	OneSecond,
	Seconds(i32),
	OneMinute,
	Minutes(i32),
	OneHour,
	Hours(i32),
	OneDay,
	Days(i32),
	Never,
}

pub trait BaseCache<S: Schema + ?Sized>: AsyncMemCache {}

impl<S, T> BaseCache<S> for T
where
	S: Schema + ?Sized,
	T: AsyncMemCache,
	// T: AsyncMemCache + MemCache,
{
}

pub trait KeyCodec<S: Schema + ?Sized>: Sized + Debug + PartialEq + Send + Sync {
	fn encode_key(&self) -> AppResult<Vec<u8>>;
	fn decode_key(data: &[u8]) -> AppResult<Self>;
}

pub trait ValueCodec<S: Schema + ?Sized>: Sized + Debug + Send + Sync {
	fn encode_value(&self) -> AppResult<Vec<u8>>;
	fn decode_value(data: &[u8]) -> AppResult<Self>;
}

pub trait Schema: Debug + Send + Sync + 'static {
	// const TTL: CacheTtl;
	type Cache: BaseCache<Self>;
	type Key: KeyCodec<Self>;
	type Value: ValueCodec<Self>;
}
