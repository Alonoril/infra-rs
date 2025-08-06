use crate::sea_ext::uint_types::{DbU128, DbU256};
use bigdecimal::BigDecimal;
use bigdecimal::num_bigint::{BigInt, BigUint, Sign};
use ruint::aliases::{U128, U256};

/// 宏：实现 DbUxxx → BigDecimal 的转换
macro_rules! impl_from_dbuint_to_bigdecimal {
    ($db_type:ty, $uint_type:ty, $byte_size:expr) => {
        impl From<$db_type> for BigDecimal {
            fn from(value: $db_type) -> Self {
                let value: $uint_type = value.into();
                let buf: [u8; $byte_size] = value.to_be_bytes();
                let big_uint = BigUint::from_bytes_be(&buf);
                let big_int = BigInt::from_biguint(Sign::Plus, big_uint);
                BigDecimal::from(big_int)
            }
        }
    };
}

/// 宏：实现 BigDecimal → DbUxxx 的转换
macro_rules! impl_try_from_bigdecimal_to_dbuint {
    ($db_type:ty, $uint_type:ty, $byte_size:expr) => {
        impl TryFrom<BigDecimal> for $db_type {
            type Error = &'static str;

            fn try_from(value: BigDecimal) -> Result<Self, Self::Error> {
                let (big_int, scale) = value.into_bigint_and_exponent();
                if scale != 0 {
                    return Err("BigDecimal has fractional part");
                }
                if big_int.sign() == Sign::Minus {
                    return Err("negative value");
                }

                let big_uint = big_int.to_biguint().ok_or("negative value")?;
                let bytes = big_uint.to_bytes_be();
                if bytes.len() > $byte_size {
                    return Err(concat!("value too large for ", stringify!($uint_type)));
                }

                // 左侧补零到指定字节数
                let mut buf = [0u8; $byte_size];
                buf[$byte_size - bytes.len()..].copy_from_slice(&bytes);
                Ok(<$uint_type>::from_be_bytes(buf).into())
            }
        }
    };
}

// 使用宏生成实现
impl_from_dbuint_to_bigdecimal!(DbU256, U256, 32);
impl_from_dbuint_to_bigdecimal!(DbU128, U128, 16);

impl_try_from_bigdecimal_to_dbuint!(DbU256, U256, 32);
impl_try_from_bigdecimal_to_dbuint!(DbU128, U128, 16);
