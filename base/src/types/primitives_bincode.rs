//! Compatibility layer for bincode implementations
//!
//! Since alloy_primitives types do not implement bincode traits,
//! we provide manual implementations for wrapper types here.

use super::primitives::{AddressWrapper, U64Wrapper, U128Wrapper, U256Wrapper};
use crate::codec::bincode::{BinDecodeExt, BinEncodeExt};
use crate::result::AppResult;

/// Convenience bincode encode/decode for wrapper types
pub trait WrapperBinCodec: Sized {
	fn wrapper_encode(&self) -> AppResult<Vec<u8>>;
	fn wrapper_decode(data: &[u8]) -> AppResult<Self>;
}

// Define macro to implement bincode encode/decode for various wrapper types
macro_rules! impl_wrapper_bincode {
	// Types with to_le_bytes/from_le_slice (U256, U128)
	($wrapper:ty, $size:expr, le_bytes) => {
		impl WrapperBinCodec for $wrapper {
			fn wrapper_encode(&self) -> AppResult<Vec<u8>> {
				let bytes: [u8; $size] = self.to_le_bytes();
				bytes.bin_encode()
			}

			fn wrapper_decode(data: &[u8]) -> AppResult<Self> {
				let bytes: [u8; $size] = data.bin_decode()?;
				Ok(Self::from_le_slice(&bytes))
			}
		}
	};

	// Simple wrapper types (U64)
	($wrapper:ty, simple) => {
		impl WrapperBinCodec for $wrapper {
			fn wrapper_encode(&self) -> AppResult<Vec<u8>> {
				self.0.bin_encode()
			}

			fn wrapper_decode(data: &[u8]) -> AppResult<Self> {
				let inner = data.bin_decode()?;
				Ok(Self(inner))
			}
		}
	};

	// Address types
	($wrapper:ty, address) => {
		impl WrapperBinCodec for $wrapper {
			fn wrapper_encode(&self) -> AppResult<Vec<u8>> {
				let bytes: &[u8; 20] = self.as_bytes();
				bytes.bin_encode()
			}

			fn wrapper_decode(data: &[u8]) -> AppResult<Self> {
				let bytes: [u8; 20] = data.bin_decode()?;
				Ok(Self::from_bytes(bytes))
			}
		}
	};
}

// Apply macro to implement bincode for wrapper types
impl_wrapper_bincode!(U256Wrapper, 32, le_bytes);
impl_wrapper_bincode!(U128Wrapper, 16, le_bytes);
impl_wrapper_bincode!(U64Wrapper, simple);
impl_wrapper_bincode!(AddressWrapper, address);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_u256_wrapper_bincode() {
		let wrapper = U256Wrapper::from(12345u64);
		let encoded = wrapper.wrapper_encode().unwrap();
		let decoded = U256Wrapper::wrapper_decode(&encoded).unwrap();
		assert_eq!(wrapper, decoded);
	}

	#[test]
	fn test_u128_wrapper_bincode() {
		let wrapper = U128Wrapper::from(67890u64);
		let encoded = wrapper.wrapper_encode().unwrap();
		let decoded = U128Wrapper::wrapper_decode(&encoded).unwrap();
		assert_eq!(wrapper, decoded);
	}

	#[test]
	fn test_u64_wrapper_bincode() {
		let wrapper = U64Wrapper::from(42u64);
		let encoded = wrapper.wrapper_encode().unwrap();
		let decoded = U64Wrapper::wrapper_decode(&encoded).unwrap();
		assert_eq!(wrapper, decoded);
	}

	#[test]
	fn test_address_wrapper_bincode() {
		let wrapper = AddressWrapper::from_bytes([1u8; 20]);
		let encoded = wrapper.wrapper_encode().unwrap();
		let decoded = AddressWrapper::wrapper_decode(&encoded).unwrap();
		assert_eq!(wrapper, decoded);
	}
}
