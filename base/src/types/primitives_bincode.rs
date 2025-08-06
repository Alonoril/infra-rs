//! Bincode 实现的兼容性层
//! 
//! 由于 alloy_primitives 的类型没有实现 bincode traits，
//! 我们在这里为 wrapper 类型提供手动实现。

use super::primitives::{U256Wrapper, U128Wrapper, U64Wrapper, AddressWrapper};
use crate::codec::bincode::{BinEncodeExt, BinDecodeExt};
use crate::result::AppResult;

/// 为 wrapper 类型提供便捷的 bincode 编码解码方法
pub trait WrapperBinCodec: Sized {
    fn wrapper_encode(&self) -> AppResult<Vec<u8>>;
    fn wrapper_decode(data: &[u8]) -> AppResult<Self>;
}

// 定义宏来为不同的 wrapper 类型实现 bincode 编码解码
macro_rules! impl_wrapper_bincode {
    // 对于有 to_le_bytes/from_le_slice 方法的类型（U256, U128）
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
    
    // 对于简单包装的类型（U64）
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
    
    // 对于地址类型
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

// 应用宏为各种 wrapper 类型实现 bincode
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