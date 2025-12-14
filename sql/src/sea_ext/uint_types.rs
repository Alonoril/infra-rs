use bigdecimal::BigDecimal;
use ruint::aliases::{U128, U256};
use sea_orm::{
	ColIdx, DbErr, QueryResult, TryGetError, TryGetable,
	sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;

// Macro: generate basic DbUxxx wrapper types
macro_rules! define_db_uint_wrapper {
	// Simple types (e.g., u64), direct wrapper
	($wrapper_name:ident, $inner_type:ty) => {
		#[derive(
			Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
		)]
		pub struct $wrapper_name(pub $inner_type);

		impl Display for $wrapper_name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "{}", self.0)
			}
		}

		impl From<$inner_type> for $wrapper_name {
			fn from(v: $inner_type) -> Self {
				$wrapper_name(v)
			}
		}

		impl From<$wrapper_name> for $inner_type {
			fn from(v: $wrapper_name) -> Self {
				v.0
			}
		}
	};
	// Complex types (e.g., U128, U256), require custom serialization
	($wrapper_name:ident, $inner_type:ty, with_custom_serde) => {
		#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct $wrapper_name(pub $inner_type);

		impl Display for $wrapper_name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "{}", self.0)
			}
		}

		impl From<$inner_type> for $wrapper_name {
			fn from(v: $inner_type) -> Self {
				$wrapper_name(v)
			}
		}

		impl From<$wrapper_name> for $inner_type {
			fn from(v: $wrapper_name) -> Self {
				v.0
			}
		}
	};
}

// Macro: implement Serialize/Deserialize for types requiring custom serialization
macro_rules! impl_db_uint_serde {
	($wrapper_name:ident, $inner_type:ty) => {
		impl Serialize for $wrapper_name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				self.0.to_string().serialize(serializer)
			}
		}

		impl<'de> Deserialize<'de> for $wrapper_name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				let s = String::deserialize(deserializer)?;
				<$inner_type>::from_str(&s)
					.map($wrapper_name)
					.map_err(serde::de::Error::custom)
			}
		}
	};
}

// Macro: implement TryGetable trait
macro_rules! impl_db_uint_try_getable {
	// Special handling for u64: convert from i64
	($wrapper_name:ident, u64) => {
		impl TryGetable for $wrapper_name {
			fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
				// PostgreSQL BIGINT can store up to 9,223,372,036,854,775,807 (i64::MAX)
				// For u64, we need to handle values > i64::MAX
				let val = i64::try_get_by(res, idx)?;
				if val < 0 {
					Err(TryGetError::Null(format!("{:?}", idx)))
				} else {
					Ok($wrapper_name(val as u64))
				}
			}
		}
	};
	// U128/U256 etc.: convert from BigDecimal
	($wrapper_name:ident, $inner_type:ty) => {
		impl TryGetable for $wrapper_name {
			fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
				// Get as BigDecimal from PostgreSQL
				let big_decimal = BigDecimal::try_get_by(res, idx)?;
				// Use our TryFrom implementation for better error handling
				big_decimal
					.try_into()
					.map_err(|e: &'static str| TryGetError::DbErr(DbErr::Type(e.to_string())))
			}
		}
	};
}

// Macro: implement ValueType-related traits
macro_rules! impl_db_uint_value_type {
	// u64 version
	($wrapper_name:ident, u64) => {
		impl ValueType for $wrapper_name {
			fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
				match v {
					Value::BigInt(Some(x)) => {
						if x < 0 {
							Err(ValueTypeErr)
						} else {
							Ok($wrapper_name(x as u64))
						}
					}
					Value::BigUnsigned(Some(x)) => Ok($wrapper_name(x)),
					_ => Err(ValueTypeErr),
				}
			}

			fn type_name() -> String {
				stringify!($wrapper_name).to_owned()
			}

			fn array_type() -> ArrayType {
				ArrayType::BigInt
			}

			fn column_type() -> ColumnType {
				ColumnType::BigInteger
			}
		}

		impl From<$wrapper_name> for Value {
			fn from(v: $wrapper_name) -> Self {
				Value::BigUnsigned(Some(v.0))
			}
		}

		impl Nullable for $wrapper_name {
			fn null() -> Value {
				Value::BigUnsigned(None)
			}
		}
	};
	// BigDecimal version (U128/U256)
	($wrapper_name:ident, $inner_type:ty, $precision:expr) => {
		impl ValueType for $wrapper_name {
			fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
				match v {
					Value::BigDecimal(Some(x)) => {
						let str_val = x.to_string();
						<$inner_type>::from_str_radix(&str_val, 10)
							.map($wrapper_name)
							.map_err(|_| ValueTypeErr)
					}
					_ => Err(ValueTypeErr),
				}
			}

			fn type_name() -> String {
				stringify!($wrapper_name).to_owned()
			}

			fn array_type() -> ArrayType {
				ArrayType::BigDecimal
			}

			fn column_type() -> ColumnType {
				ColumnType::Decimal(Some(($precision, 0)))
			}
		}

		impl From<$wrapper_name> for Value {
			fn from(v: $wrapper_name) -> Self {
				// BigDecimal can handle arbitrary precision
				let str_val = v.0.to_string();
				match BigDecimal::from_str(&str_val) {
					Ok(big_decimal) => Value::BigDecimal(Some(Box::new(big_decimal))),
					Err(_) => {
						// This should never happen for valid values
						panic!(
							"Failed to convert {} to BigDecimal: {}",
							stringify!($inner_type),
							str_val
						)
					}
				}
			}
		}

		impl Nullable for $wrapper_name {
			fn null() -> Value {
				Value::BigDecimal(None)
			}
		}
	};
}

// Generate DbU64 via macro
define_db_uint_wrapper!(DbU64, u64);
impl_db_uint_try_getable!(DbU64, u64);

// Generate DbU128 via macro
define_db_uint_wrapper!(DbU128, U128, with_custom_serde);
impl_db_uint_serde!(DbU128, U128);
impl_db_uint_try_getable!(DbU128, U128);

// Generate DbU256 via macro
define_db_uint_wrapper!(DbU256, U256, with_custom_serde);
impl_db_uint_serde!(DbU256, U256);
impl_db_uint_try_getable!(DbU256, U256);

// Implement ValueType-related traits via macros
impl_db_uint_value_type!(DbU64, u64);

// Implement ValueType-related traits via macros
// U128 max is ~340 undecillion (38 digits), so NUMERIC(39,0) is sufficient
impl_db_uint_value_type!(DbU128, U128, 39);

// Implement ValueType-related traits via macros
// U256 max is ~115 quattuorvigintillion (78 digits), so NUMERIC(78,0) is sufficient
impl_db_uint_value_type!(DbU256, U256, 78);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_u64_value_conversion() {
		// Test u64 to Value
		let val = DbU64(1234567890);
		let value = Value::from(val);
		assert!(matches!(value, Value::BigUnsigned(Some(1234567890))));

		// Test Value to DbU64
		let result = <DbU64 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, val);
	}

	#[test]
	fn test_u64_max_value() {
		let val = DbU64(u64::MAX);
		let value = Value::from(val);
		let result = <DbU64 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, val);
	}

	#[test]
	fn test_u128_value_conversion() {
		// Test U128 to Value
		let val = DbU128(U128::from(1234567890u64));
		let value = Value::from(val);
		assert!(matches!(value, Value::BigDecimal(Some(_))));

		// Test Value to DbU128
		let result = <DbU128 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, val);
	}

	#[test]
	fn test_u128_large_value() {
		// Test with a value larger than u64::MAX
		let val = DbU128(U128::from_str_radix("123456789012345678901234567890", 10).unwrap());
		let value = Value::from(val);
		// BigDecimal can handle this value
		assert!(matches!(value, Value::BigDecimal(Some(_))));
		let result = <DbU128 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, val);
	}

	#[test]
	fn test_u256_value_conversion() {
		// Test U256 to Value
		let val = DbU256(U256::from(1234567890u64));
		let value = Value::from(val);
		assert!(matches!(value, Value::BigDecimal(Some(_))));

		// Test Value to DbU256
		let result = <DbU256 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, val);
	}

	#[test]
	fn test_u256_large_value() {
		// Test with a value larger than u128::MAX
		let val = DbU256(
			U256::from_str_radix(
				"123456789012345678901234567890123456789012345678901234567890",
				10,
			)
			.unwrap(),
		);
		let value = Value::from(val);
		// BigDecimal can handle this value
		assert!(matches!(value, Value::BigDecimal(Some(_))));
		let result = <DbU256 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, val);
	}

	#[test]
	fn test_column_types() {
		assert_eq!(DbU64::column_type(), ColumnType::BigInteger);
		assert_eq!(DbU128::column_type(), ColumnType::Decimal(Some((39, 0))));
		assert_eq!(DbU256::column_type(), ColumnType::Decimal(Some((78, 0))));
	}

	#[test]
	fn test_type_names() {
		assert_eq!(DbU64::type_name(), "DbU64");
		assert_eq!(DbU128::type_name(), "DbU128");
		assert_eq!(DbU256::type_name(), "DbU256");
	}

	#[test]
	fn test_u128_max_value() {
		let val = DbU128(U128::MAX);
		let value = Value::from(val);
		// BigDecimal can handle U128::MAX
		assert!(matches!(value, Value::BigDecimal(Some(_))));
		let result = <DbU128 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, val);
	}

	#[test]
	fn test_u256_max_value() {
		let val = DbU256(U256::MAX);
		let value = Value::from(val);
		// BigDecimal can handle U256::MAX
		assert!(matches!(value, Value::BigDecimal(Some(_))));
		let result = <DbU256 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, val);
	}

	#[test]
	fn test_invalid_conversions() {
		// Test negative value to DbU64
		let neg_value = Value::BigInt(Some(-1));
		assert!(<DbU64 as ValueType>::try_from(neg_value).is_err());

		// Test invalid type conversions
		let int_value = Value::Int(Some(42));
		assert!(<DbU128 as ValueType>::try_from(int_value.clone()).is_err());
		assert!(<DbU256 as ValueType>::try_from(int_value).is_err());
	}

	#[test]
	fn test_dbu64_additional_cases() {
		// Test DbU64 with BigInt positive value
		let bigint_value = Value::BigInt(Some(42));
		let result = <DbU64 as ValueType>::try_from(bigint_value).unwrap();
		assert_eq!(result, DbU64(42));

		// Test DbU64 with zero
		let zero_value = Value::BigInt(Some(0));
		let result = <DbU64 as ValueType>::try_from(zero_value).unwrap();
		assert_eq!(result, DbU64(0));

		// Test DbU64 with BigUnsigned
		let bigunsigned_value = Value::BigUnsigned(Some(12345));
		let result = <DbU64 as ValueType>::try_from(bigunsigned_value).unwrap();
		assert_eq!(result, DbU64(12345));

		// Test invalid types for DbU64
		let string_value = Value::String(Some(Box::new("123".to_string())));
		assert!(<DbU64 as ValueType>::try_from(string_value).is_err());

		let float_value = Value::Float(Some(123.45));
		assert!(<DbU64 as ValueType>::try_from(float_value).is_err());

		let null_value = Value::BigInt(None);
		assert!(<DbU64 as ValueType>::try_from(null_value).is_err());
	}

	#[test]
	fn test_nullable_implementations() {
		// Test null values for each type
		let null_u64 = <DbU64 as Nullable>::null();
		assert!(matches!(null_u64, Value::BigUnsigned(None)));

		let null_u128 = <DbU128 as Nullable>::null();
		assert!(matches!(null_u128, Value::BigDecimal(None)));

		let null_u256 = <DbU256 as Nullable>::null();
		assert!(matches!(null_u256, Value::BigDecimal(None)));
	}

	#[test]
	fn test_array_types() {
		// Test array types for each wrapper
		assert_eq!(<DbU64 as ValueType>::array_type(), ArrayType::BigInt);
		assert_eq!(<DbU128 as ValueType>::array_type(), ArrayType::BigDecimal);
		assert_eq!(<DbU256 as ValueType>::array_type(), ArrayType::BigDecimal);
	}

	#[test]
	fn test_display_implementations() {
		// Test Display implementations
		let dbu64 = DbU64(12345);
		assert_eq!(format!("{}", dbu64), "12345");

		let dbu128 = DbU128(U128::from(67890u64));
		assert_eq!(format!("{}", dbu128), "67890");

		let dbu256 = DbU256(U256::from(11111u64));
		assert_eq!(format!("{}", dbu256), "11111");
	}

	#[test]
	fn test_wrapper_conversions() {
		// Test u64 <-> DbU64
		let u64_val: u64 = 42;
		let db_u64 = DbU64::from(u64_val);
		assert_eq!(db_u64.0, u64_val);
		let back_u64: u64 = db_u64.into();
		assert_eq!(back_u64, u64_val);

		// Test U128 <-> DbU128
		let u128_val = U128::from(42u64);
		let db_u128 = DbU128::from(u128_val);
		assert_eq!(db_u128.0, u128_val);
		let back_u128: U128 = db_u128.into();
		assert_eq!(back_u128, u128_val);

		// Test U256 <-> DbU256
		let u256_val = U256::from(42u64);
		let db_u256 = DbU256::from(u256_val);
		assert_eq!(db_u256.0, u256_val);
		let back_u256: U256 = db_u256.into();
		assert_eq!(back_u256, u256_val);
	}

	#[test]
	fn test_dbu64_edge_cases() {
		// Test i64::MAX value (should work)
		let max_i64_value = Value::BigInt(Some(i64::MAX));
		let result = <DbU64 as ValueType>::try_from(max_i64_value).unwrap();
		assert_eq!(result, DbU64(i64::MAX as u64));

		// Test edge case at i64 boundary
		let boundary_value = Value::BigUnsigned(Some((i64::MAX as u64) + 1));
		let result = <DbU64 as ValueType>::try_from(boundary_value).unwrap();
		assert_eq!(result, DbU64((i64::MAX as u64) + 1));
	}

	#[test]
	fn test_bigdecimal_edge_cases() {
		// Test U128 with value larger than u64::MAX
		let large_u128 = U128::from_str_radix("18446744073709551616", 10).unwrap(); // u64::MAX + 1
		let db_u128 = DbU128(large_u128);
		let value = Value::from(db_u128);
		let result = <DbU128 as ValueType>::try_from(value).unwrap();
		assert_eq!(result, db_u128);

		// Test invalid BigDecimal value for U128 (non-numeric string would fail at parse time)
		// Instead test with a string that parses but contains invalid characters for base 10
		let invalid_bd = Value::String(Some(Box::new("not_a_number".to_string())));
		assert!(<DbU128 as ValueType>::try_from(invalid_bd).is_err());
	}

	#[test]
	fn test_derive_traits() {
		// Test Clone
		let dbu64 = DbU64(123);
		let cloned = dbu64.clone();
		assert_eq!(dbu64, cloned);

		// Test Debug
		let debug_str = format!("{:?}", dbu64);
		assert!(debug_str.contains("DbU64"));
		assert!(debug_str.contains("123"));

		// Test PartialOrd and Ord
		let dbu64_small = DbU64(100);
		let dbu64_large = DbU64(200);
		assert!(dbu64_small < dbu64_large);
		assert!(dbu64_large > dbu64_small);

		// Test Hash
		use std::collections::hash_map::DefaultHasher;
		use std::hash::{Hash, Hasher};
		let mut hasher = DefaultHasher::new();
		dbu64.hash(&mut hasher);
		let hash1 = hasher.finish();

		let mut hasher2 = DefaultHasher::new();
		dbu64.hash(&mut hasher2);
		let hash2 = hasher2.finish();
		assert_eq!(hash1, hash2);
	}
}

// =============================================================================
// Extension example: how to add a new DbUxxx type
// =============================================================================
//
// With these macros, you can easily add new DB wrapper types. For example, DbU512:
//
// 1. Import necessary types at the top:
//    use ruint::aliases::U512;
//
// 2. Use macros to generate type definitions and impls:
//
//    // Generate DbU512 wrapper type (needs custom serde)
//    define_db_uint_wrapper!(DbU512, U512, with_custom_serde);
//
//    // Implement custom serde
//    impl_db_uint_serde!(DbU512, U512);
//
//    // Implement TryGetable trait
//    impl_db_uint_try_getable!(DbU512, U512);
//
//    // Implement ValueType-related traits
//    // U512 max has 155 digits, so use NUMERIC(155,0)
//    impl_db_uint_value_type!(DbU512, U512, 155);
//
// 3. Add TryFrom in bigdecimal.rs:
//    impl TryFrom<BigDecimal> for DbU512 {
//        type Error = &'static str;
//        fn try_from(value: BigDecimal) -> Result<Self, Self::Error> {
//            // Implement conversion logic
//        }
//    }
//
// =============================================================================
