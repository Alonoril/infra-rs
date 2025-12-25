use crate::result::AppResult;

pub trait RkyvCodecExt {
	fn rkyv_encode(&self) -> AppResult<Vec<u8>>;

	fn rkyv_decode(bytes: &[u8]) -> AppResult<Self>
	where
		Self: Sized;
}

/// ```rust
/// use base_infra::codec::rkyv::RkyvCodecExt;
/// use base_infra::impl_rkyv_codec;
/// use rkyv_derive::{Archive, Deserialize, Serialize};
///
/// #[derive(Clone, Debug, Default, PartialEq, Archive, Deserialize, Serialize)]
/// pub struct Value {
/// 	pub val0: f64,
/// 	pub val1: Option<String>,
/// }
///
/// impl_rkyv_codec!(Value, ArchivedValue);
///
/// #[cfg(test)]
/// fn test_rkyv_encode_decode() {
/// 	let val = Value {
/// 		val0: 1.2,
/// 		val1: Some("rkyv".into()),
/// 	};
///
/// 	let bytes = val.rkyv_encode().unwrap();
/// 	let val2 = Value::rkyv_decode(&bytes[..]).unwrap();
/// 	assert_eq!(val, val2);
/// }
/// ```
#[macro_export]
macro_rules! impl_rkyv_codec {
	($value_type:ty, $archived_type:ty) => {
		impl $crate::codec::rkyv::RkyvCodecExt for $value_type {
			fn rkyv_encode(&self) -> $crate::result::AppResult<Vec<u8>> {
				use rkyv::ser::allocator::Arena;
				use $crate::codec::error::RkyvErr;
				let mut arena = Arena::new();
				let bytes =
					::rkyv::api::high::to_bytes_with_alloc::<_, rancor::Error>(self, arena.acquire())
						.map_err($crate::map_err!(&RkyvErr::EncodeWithArena))?;
				Ok(bytes.into_vec())
			}

			fn rkyv_decode(bytes: &[u8]) -> $crate::result::AppResult<Self>
			where
				Self: Sized,
			{
				use $crate::codec::error::RkyvErr;
				let archived = ::rkyv::access::<$archived_type, rancor::Error>(bytes)
					.map_err($crate::map_err!(&RkyvErr::DecodeToArchivedType))?;

				::rkyv::api::high::deserialize::<Self, rancor::Error>(archived)
					.map_err($crate::map_err!(&RkyvErr::DeserFromArchived))
			}
		}
	};
}

