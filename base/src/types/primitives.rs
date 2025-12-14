//! Unified base wrapper types for big integers and address types
//!
//! These serve as base types that can be extended by implementing traits:
//! - In sql-infra for DB-related traits (TryGetable, ValueType, etc.)
//! - In alloy-ext for binary serialization traits (bincode::Encode/Decode)

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;

// =============================================================================
// Macro: generate base wrapper types and base impls
// =============================================================================

/// Macro: generate base wrapper types and common impls
/// Refer to uint_types.rs design, auto-infer error types
macro_rules! define_primitive_wrapper {
	// alloy_primitives numeric types (U256, U128)
	($wrapper_name:ident, alloy_primitives::U256, alloy_uint) => {
		#[derive(
			Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
		)]
		#[repr(transparent)]
		pub struct $wrapper_name(pub alloy_primitives::U256);

		impl Display for $wrapper_name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "{}", self.0)
			}
		}

		impl From<alloy_primitives::U256> for $wrapper_name {
			fn from(v: alloy_primitives::U256) -> Self {
				$wrapper_name(v)
			}
		}

		impl From<$wrapper_name> for alloy_primitives::U256 {
			fn from(v: $wrapper_name) -> Self {
				v.0
			}
		}

		impl From<u64> for $wrapper_name {
			fn from(v: u64) -> Self {
				$wrapper_name(alloy_primitives::U256::from(v))
			}
		}

		impl FromStr for $wrapper_name {
			type Err = alloy_primitives::ruint::ParseError;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				alloy_primitives::U256::from_str(s).map($wrapper_name)
			}
		}
	};

	($wrapper_name:ident, alloy_primitives::U128, alloy_uint) => {
		#[derive(
			Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
		)]
		#[repr(transparent)]
		pub struct $wrapper_name(pub alloy_primitives::U128);

		impl Display for $wrapper_name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "{}", self.0)
			}
		}

		impl From<alloy_primitives::U128> for $wrapper_name {
			fn from(v: alloy_primitives::U128) -> Self {
				$wrapper_name(v)
			}
		}

		impl From<$wrapper_name> for alloy_primitives::U128 {
			fn from(v: $wrapper_name) -> Self {
				v.0
			}
		}

		impl From<u64> for $wrapper_name {
			fn from(v: u64) -> Self {
				$wrapper_name(alloy_primitives::U128::from(v))
			}
		}

		impl FromStr for $wrapper_name {
			type Err = alloy_primitives::ruint::ParseError;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				alloy_primitives::U128::from_str(s).map($wrapper_name)
			}
		}
	};

	// alloy_primitives address type
	($wrapper_name:ident, alloy_primitives::Address, alloy_primitive) => {
		#[derive(
			Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
		)]
		#[repr(transparent)]
		pub struct $wrapper_name(pub alloy_primitives::Address);

		impl Display for $wrapper_name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "{}", self.0)
			}
		}

		impl From<alloy_primitives::Address> for $wrapper_name {
			fn from(v: alloy_primitives::Address) -> Self {
				$wrapper_name(v)
			}
		}

		impl From<$wrapper_name> for alloy_primitives::Address {
			fn from(v: $wrapper_name) -> Self {
				v.0
			}
		}

		impl FromStr for $wrapper_name {
			type Err = alloy_primitives::hex::FromHexError;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				alloy_primitives::Address::from_str(s).map($wrapper_name)
			}
		}
	};

	// Simple type (u64)
	($wrapper_name:ident, u64, simple) => {
		#[derive(
			Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
		)]
		#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
		#[repr(transparent)]
		pub struct $wrapper_name(pub u64);

		impl Display for $wrapper_name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "{}", self.0)
			}
		}

		impl From<u64> for $wrapper_name {
			fn from(v: u64) -> Self {
				$wrapper_name(v)
			}
		}

		impl From<$wrapper_name> for u64 {
			fn from(v: $wrapper_name) -> Self {
				v.0
			}
		}

		impl FromStr for $wrapper_name {
			type Err = std::num::ParseIntError;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				u64::from_str(s).map($wrapper_name)
			}
		}
	};
}

/// Macro: add convenience methods for wrapper types
macro_rules! impl_wrapper_utils {
	// U256 specific methods
	($wrapper_name:ident, U256) => {
		impl $wrapper_name {
			/// Create a zero value
			pub const ZERO: Self = Self(alloy_primitives::U256::ZERO);

			/// Create a max value
			pub const MAX: Self = Self(alloy_primitives::U256::MAX);

			/// Convert to little-endian bytes
			pub fn to_le_bytes(&self) -> [u8; 32] {
				self.0.to_le_bytes()
			}

			/// Create from little-endian bytes
			pub fn from_le_slice(slice: &[u8]) -> Self {
				Self(alloy_primitives::U256::from_le_slice(slice))
			}
		}
	};

	// U128 specific methods
	($wrapper_name:ident, U128) => {
		impl $wrapper_name {
			/// Create a zero value
			pub const ZERO: Self = Self(alloy_primitives::U128::ZERO);

			/// Create a max value
			pub const MAX: Self = Self(alloy_primitives::U128::MAX);

			/// Convert to little-endian bytes
			pub fn to_le_bytes(&self) -> [u8; 16] {
				self.0.to_le_bytes()
			}

			/// Create from little-endian bytes
			pub fn from_le_slice(slice: &[u8]) -> Self {
				Self(alloy_primitives::U128::from_le_slice(slice))
			}
		}
	};

	// u64 specific methods
	($wrapper_name:ident, u64) => {
		impl $wrapper_name {
			/// Create a zero value
			pub const ZERO: Self = Self(0);

			/// Create a max value
			pub const MAX: Self = Self(u64::MAX);
		}
	};

	// Address specific methods
	($wrapper_name:ident, Address) => {
		impl $wrapper_name {
			/// Create a zero address
			pub const ZERO: Self = Self(alloy_primitives::Address::ZERO);

			/// Get raw bytes
			pub fn as_bytes(&self) -> &[u8; 20] {
				self.0.as_ref()
			}

			/// Create from byte array
			pub fn from_bytes(bytes: [u8; 20]) -> Self {
				Self(alloy_primitives::Address::from(bytes))
			}
		}
	};
}

// =============================================================================
// Generate wrapper types - compact pattern like uint_types.rs
// =============================================================================

// Generate U256Wrapper via macro
define_primitive_wrapper!(U256Wrapper, alloy_primitives::U256, alloy_uint);
impl_wrapper_utils!(U256Wrapper, U256);

// Generate U128Wrapper via macro
define_primitive_wrapper!(U128Wrapper, alloy_primitives::U128, alloy_uint);
impl_wrapper_utils!(U128Wrapper, U128);

// Generate U64Wrapper via macro
define_primitive_wrapper!(U64Wrapper, u64, simple);
impl_wrapper_utils!(U64Wrapper, u64);

// Generate AddressWrapper via macro
define_primitive_wrapper!(AddressWrapper, alloy_primitives::Address, alloy_primitive);
impl_wrapper_utils!(AddressWrapper, Address);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_u256_wrapper() {
		// Test basic conversions
		let val = alloy_primitives::U256::from(12345u64);
		let wrapper = U256Wrapper::from(val);
		assert_eq!(wrapper.0, val);

		let back: alloy_primitives::U256 = wrapper.into();
		assert_eq!(back, val);

		// Test Display
		assert_eq!(wrapper.to_string(), "12345");

		// Test create from u64
		let wrapper2 = U256Wrapper::from(12345u64);
		assert_eq!(wrapper, wrapper2);

		// Test constants
		assert_eq!(U256Wrapper::ZERO.0, alloy_primitives::U256::ZERO);
		assert_eq!(U256Wrapper::MAX.0, alloy_primitives::U256::MAX);
	}

	#[test]
	fn test_u128_wrapper() {
		let val = alloy_primitives::U128::from(67890u64);
		let wrapper = U128Wrapper::from(val);
		assert_eq!(wrapper.0, val);

		let back: alloy_primitives::U128 = wrapper.into();
		assert_eq!(back, val);

		assert_eq!(wrapper.to_string(), "67890");
	}

	#[test]
	fn test_u64_wrapper() {
		let val = 42u64;
		let wrapper = U64Wrapper::from(val);
		assert_eq!(wrapper.0, val);

		let back: u64 = wrapper.into();
		assert_eq!(back, val);

		assert_eq!(wrapper.to_string(), "42");
	}

	#[test]
	fn test_address_wrapper() {
		let val = alloy_primitives::Address::from([1u8; 20]);
		let wrapper = AddressWrapper::from(val);
		assert_eq!(wrapper.0, val);

		let back: alloy_primitives::Address = wrapper.into();
		assert_eq!(back, val);
	}

	#[test]
	fn test_from_str() {
		// U256
		let wrapper = U256Wrapper::from_str("12345").unwrap();
		assert_eq!(wrapper.0, alloy_primitives::U256::from(12345u64));

		// U128
		let wrapper = U128Wrapper::from_str("67890").unwrap();
		assert_eq!(wrapper.0, alloy_primitives::U128::from(67890u64));

		// U64
		let wrapper = U64Wrapper::from_str("42").unwrap();
		assert_eq!(wrapper.0, 42u64);
	}

	#[test]
	fn test_serialization() {
		// Test serde serialization
		let wrapper = U256Wrapper::from(12345u64);
		let json = serde_json::to_string(&wrapper).unwrap();
		let deserialized: U256Wrapper = serde_json::from_str(&json).unwrap();
		assert_eq!(wrapper, deserialized);
	}
}
