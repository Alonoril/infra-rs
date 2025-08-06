
/// A macro to generate the `KeyCodec` and `ValueCodec` implementations for a given schema type.
#[macro_export]
macro_rules! impl_schema_bin_codec {
    ($schema_type:ty, $key_type:ty, $value_type:ty) => {
        impl $crate::schema::KeyCodec<$schema_type> for $key_type {
            fn encode_key(&self) -> base_infra::result::AppResult<Vec<u8>> {
                use base_infra::codec::bincode::BinEncodeExt;
                self.bin_encode()
            }

            fn decode_key(data: &[u8]) -> base_infra::result::AppResult<Self> {
                use base_infra::codec::bincode::BinDecodeExt;
                data.bin_decode::<$key_type>()
            }
        }

        impl $crate::schema::ValueCodec<$schema_type> for $value_type {
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


