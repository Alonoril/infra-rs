/// ```rust
/// pub struct SchemaKey;
///
/// impl SchemaKey {
/// 	pub fn encode(&self) -> AppResult<Vec<u8>> {
/// 		Ok(vec![])
/// 	}
///
/// 	pub fn decode(bytes: &[u8]) -> AppResult<SchemaKey> {
/// 		Ok(SchemaKey)
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! impl_schema_key_codec {
	($schema_type:ty, $key_type:ty) => {
		impl $crate::schemadb::schema::KeyCodec<$schema_type> for $key_type {
			fn encode_key(&self) -> base_infra::result::AppResult<Vec<u8>> {
				self.encode()
			}

			fn decode_key(data: &[u8]) -> base_infra::result::AppResult<Self> {
				Self::decode(data)
			}
		}
	};
}

/// ```rust
/// pub struct SchemaValue;
///
/// impl SchemaValue {
/// 	pub fn encode(&self) -> AppResult<Vec<u8>> {
/// 		Ok(vec![])
/// 	}
///
/// 	pub fn decode(bytes: &[u8]) -> AppResult<SchemaValue> {
/// 		Ok(SchemaValue)
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! impl_schema_value_codec {
	($schema_type:ty, $value_type:ty) => {
		impl $crate::schemadb::schema::ValueCodec<$schema_type> for $value_type {
			fn encode_value(&self) -> base_infra::result::AppResult<Vec<u8>> {
				self.encode()
			}

			fn decode_value(data: &[u8]) -> base_infra::result::AppResult<Self> {
				Self::decode(data)
			}
		}
	};
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
/// // first use impl_rkyv_codec
/// impl_rkyv_codec!(Value, ArchivedValue);
///
/// // and use impl_schema_value_rkyv_codec
/// // impl_schema_value_rkyv_codec!(...);
/// ```
#[macro_export]
macro_rules! impl_schema_value_rkyv_codec {
	($schema_type:ty, $value_type:ty) => {
		impl $crate::schemadb::schema::ValueCodec<$schema_type> for $value_type {
			fn encode_value(&self) -> base_infra::result::AppResult<Vec<u8>> {
				use base_infra::codec::rkyv::RkyvCodecExt;
				self.rkyv_encode()
			}

			fn decode_value(data: &[u8]) -> base_infra::result::AppResult<Self> {
				use base_infra::codec::rkyv::RkyvCodecExt;
				Self::rkyv_decode(data)
			}
		}
	};
}

/// A macro to generate the `KeyCodec` and `ValueCodec` implementations for a given schema type  with bincode.
#[macro_export]
macro_rules! impl_schema_bin_codec {
	($schema_type:ty, $key_type:ty, $value_type:ty) => {
		impl $crate::schemadb::schema::KeyCodec<$schema_type> for $key_type {
			fn encode_key(&self) -> base_infra::result::AppResult<Vec<u8>> {
				use base_infra::codec::bincode::BinEncodeExt;
				self.bin_encode()
			}

			fn decode_key(data: &[u8]) -> base_infra::result::AppResult<Self> {
				use base_infra::codec::bincode::BinDecodeExt;
				data.bin_decode::<$key_type>()
			}
		}

		impl $crate::schemadb::schema::ValueCodec<$schema_type> for $value_type {
			fn encode_value(&self) -> base_infra::result::AppResult<Vec<u8>> {
				use base_infra::codec::bincode::BinEncodeExt;
				self.bin_encode()
			}

			fn decode_value(data: &[u8]) -> base_infra::result::AppResult<Self> {
				use base_infra::codec::bincode::BinDecodeExt;
				data.bin_decode::<$value_type>()
			}
		}
	};
}

/// A macro to generate the `ValueCodec` implementations for a given schema type  with bincode.
#[macro_export]
macro_rules! impl_schema_value_bin_codec {
	($schema_type:ty, $value_type:ty) => {
		impl $crate::schemadb::schema::ValueCodec<$schema_type> for $value_type {
			fn encode_value(&self) -> base_infra::result::AppResult<Vec<u8>> {
				use base_infra::codec::bincode::BinEncodeExt;
				self.bin_encode()
			}

			fn decode_value(data: &[u8]) -> base_infra::result::AppResult<Self> {
				use base_infra::codec::bincode::BinDecodeExt;
				data.bin_decode::<$value_type>()
			}
		}
	};
}

/// A macro to generate the `KeyCodec` and `ValueCodec` implementations for a given schema type with BCS.
#[macro_export]
macro_rules! impl_schema_bcs_codec {
	($schema_type:ty, $key_type:ty, $value_type:ty) => {
		impl $crate::schemadb::schema::KeyCodec<$schema_type> for $key_type {
			fn encode_key(&self) -> base_infra::result::AppResult<Vec<u8>> {
				bcs::to_bytes(self)
					.map_err(base_infra::map_err!(&rksdb_infra::errors::RksErr::BcsErr))
			}

			fn decode_key(data: &[u8]) -> base_infra::result::AppResult<Self> {
				bcs::from_bytes(data)
					.map_err(base_infra::map_err!(&rksdb_infra::errors::RksErr::BcsErr))
			}
		}

		impl $crate::schemadb::schema::ValueCodec<$schema_type> for $value_type {
			fn encode_value(&self) -> base_infra::result::AppResult<Vec<u8>> {
				bcs::to_bytes(self)
					.map_err(base_infra::map_err!(&rksdb_infra::errors::RksErr::BcsErr))
			}

			fn decode_value(data: &[u8]) -> base_infra::result::AppResult<Self> {
				bcs::from_bytes(data)
					.map_err(base_infra::map_err!(&rksdb_infra::errors::RksErr::BcsErr))
			}
		}
	};
}

/// A macro to generate the `KeyCodec` implementation for a given schema type.
#[macro_export]
macro_rules! impl_schema_key_bcs_codec {
	($schema_type:ty, $key_type:ty) => {
		impl $crate::schemadb::schema::KeyCodec<$schema_type> for $key_type {
			fn encode_key(&self) -> anyhow::Result<Vec<u8>> {
				bcs::to_bytes(self).map_err(Into::into)
			}

			fn decode_key(data: &[u8]) -> anyhow::Result<Self> {
				bcs::from_bytes(data).map_err(Into::into)
			}
		}
	};
}

/// A macro to generate the `ValueCodec` implementation for a given schema type.
#[macro_export]
macro_rules! impl_schema_value_bcs_codec {
	($schema_type:ty, $value_type:ty) => {
		impl $crate::schemadb::schema::ValueCodec<$schema_type> for $value_type {
			fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
				bcs::to_bytes(self).map_err(Into::into)
			}

			fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
				bcs::from_bytes(data).map_err(Into::into)
			}
		}
	};
}
