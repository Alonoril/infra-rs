pub mod bincode;

/// A macro to generate the `ValueCodec` implementation for a given schema type.
#[macro_export]
macro_rules! impl_schema_value_serde_codec {
	($schema_type:ty, $value_type:ty) => {
		impl $crate::schema::ValueCodec<$schema_type> for $value_type {
			fn encode_value(&self) -> base_infra::result::AppResult<Vec<u8>> {
				serde_json::to_vec(self).map_err(Into::into)
			}

			fn decode_value(data: &[u8]) -> base_infra::result::AppResult<Self> {
				serde_json::from_slice(data).map_err(Into::into)
			}
		}
	};
}
