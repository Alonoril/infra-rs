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


/// A macro to generate the `KeyCodec` and `ValueCodec` implementations for a given schema type with BCS.
#[macro_export]
macro_rules! impl_schema_bcs_codec {
    ($schema_type:ty, $key_type:ty, $value_type:ty) => {
        impl $crate::schemadb::schema::KeyCodec<$schema_type> for $key_type {
            fn encode_key(&self) -> base_infra::result::AppResult<Vec<u8>> {
                bcs::to_bytes(self).map_err(base_infra::map_err!(&rksdb_infra::errors::RksErr::BcsErr))
            }

            fn decode_key(data: &[u8]) -> base_infra::result::AppResult<Self> {
                bcs::from_bytes(data).map_err(base_infra::map_err!(&rksdb_infra::errors::RksErr::BcsErr))
            }
        }

        impl $crate::schemadb::schema::ValueCodec<$schema_type> for $value_type {
            fn encode_value(&self) -> base_infra::result::AppResult<Vec<u8>> {
                bcs::to_bytes(self).map_err(base_infra::map_err!(&rksdb_infra::errors::RksErr::BcsErr))
            }

            fn decode_value(data: &[u8]) -> base_infra::result::AppResult<Self> {
                bcs::from_bytes(data).map_err(base_infra::map_err!(&rksdb_infra::errors::RksErr::BcsErr))
            }
        }
    };
}

// /// A macro to generate the `KeyCodec` implementation for a given schema type.
// #[macro_export]
// macro_rules! impl_schema_key_bcs_codec {
//     ($schema_type:ty, $key_type:ty) => {
//         impl $crate::schemadb::schema::KeyCodec<$schema_type> for $key_type {
//             fn encode_key(&self) -> anyhow::Result<Vec<u8>> {
//                 bcs::to_bytes(self).map_err(Into::into)
//             }
//
//             fn decode_key(data: &[u8]) -> anyhow::Result<Self> {
//                 bcs::from_bytes(data).map_err(Into::into)
//             }
//         }
//     };
// }
//
// /// A macro to generate the `ValueCodec` implementation for a given schema type.
// #[macro_export]
// macro_rules! impl_schema_value_bcs_codec {
//     ($schema_type:ty, $value_type:ty) => {
//         impl $crate::schemadb::schema::ValueCodec<$schema_type> for $value_type {
//             fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
//                 bcs::to_bytes(self).map_err(Into::into)
//             }
//
//             fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
//                 bcs::from_bytes(data).map_err(Into::into)
//             }
//         }
//     };
// }