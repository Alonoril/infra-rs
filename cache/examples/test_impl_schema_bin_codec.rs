use bincode::{Decode, Encode};
use cache_infra::memory::{AsyncMemCache, SecondsMemCache};
use cache_infra::{define_pub_schema, impl_schema_bin_codec};
use serde::{Deserialize, Serialize};
// #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
// pub struct McKey(i32, i32);
//
// #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
// pub struct McValue(i32, String, bool);
//
// define_pub_schema!(MinuteCacheSchema, McKey, McValue, CacheTtl::OneMinute);
//
// impl_schema_bin_codec!(MinuteCacheSchema, McKey, McValue);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct McKeySec(i32, i32);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct McValueSec(i32, String, bool);

define_pub_schema!(SecondsCacheSchema, McKeySec, McValueSec, SecondsMemCache);

impl_schema_bin_codec!(SecondsCacheSchema, McKeySec, McValueSec);

#[tokio::main]
async fn main() {
	let key = McKeySec(1, 2);
	let value = McValueSec(1, "hello".to_string(), true);

	SecondsMemCache.init_cache();
	SecondsMemCache
		.async_store::<SecondsCacheSchema>(&key, &value)
		.await
		.unwrap();

	let v = SecondsMemCache
		.async_load::<SecondsCacheSchema>(&key)
		.await
		.unwrap();

	println!("{:?}", v);
	assert_eq!(v, Some(value));
}

// mod t1_mod {
// 	use crate::{McKeySec, SecondsCacheSchema};
//
// 	impl cache_infra::schema::KeyCodec<SecondsCacheSchema> for McKeySec {
// 		fn encode_key(&self) -> base_infra::result::AppResult<Vec<u8>> {
// 			use base_infra::codec::bincode::BinEncodeExt;
// 			self.bin_encode()
// 		}
//
// 		fn decode_key(data: &[u8]) -> base_infra::result::AppResult<Self> {
// 			use base_infra::codec::bincode::BinDecodeExt;
// 			data.bin_decode::<McKeySec>()
// 		}
// 	}
// }
