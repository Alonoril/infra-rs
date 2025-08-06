//! 统一的基础 wrapper 类型，用于包装大整数和地址类型
//! 
//! 这些类型作为基础类型，可以在不同的模块中通过实现特定的 trait 来扩展功能：
//! - 在 sql-infra 中实现数据库相关的 traits（TryGetable, ValueType 等）
//! - 在 alloy-ext 中实现二进制序列化 traits（bincode::Encode/Decode）

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;

// =============================================================================
// 宏定义：生成基础包装类型和基础实现
// =============================================================================

/// 宏：生成基础包装类型和通用实现
/// 参考 uint_types.rs 的设计，自动推断错误类型
macro_rules! define_primitive_wrapper {
    // alloy_primitives 数字类型（U256, U128）
    ($wrapper_name:ident, alloy_primitives::U256, alloy_uint) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    
    // alloy_primitives 地址类型
    ($wrapper_name:ident, alloy_primitives::Address, alloy_primitive) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    
    // 简单类型（u64）
    ($wrapper_name:ident, u64, simple) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[cfg_attr(feature = "bin-codec", derive(bincode::Encode, bincode::Decode))]
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

/// 宏：为包装类型添加便捷方法
macro_rules! impl_wrapper_utils {
    // U256 特定方法
    ($wrapper_name:ident, U256) => {
        impl $wrapper_name {
            /// 创建一个零值
            pub const ZERO: Self = Self(alloy_primitives::U256::ZERO);
            
            /// 创建一个最大值
            pub const MAX: Self = Self(alloy_primitives::U256::MAX);
            
            /// 转换为小端字节数组
            pub fn to_le_bytes(&self) -> [u8; 32] {
                self.0.to_le_bytes()
            }
            
            /// 从小端字节数组创建
            pub fn from_le_slice(slice: &[u8]) -> Self {
                Self(alloy_primitives::U256::from_le_slice(slice))
            }
        }
    };
    
    // U128 特定方法
    ($wrapper_name:ident, U128) => {
        impl $wrapper_name {
            /// 创建一个零值
            pub const ZERO: Self = Self(alloy_primitives::U128::ZERO);
            
            /// 创建一个最大值
            pub const MAX: Self = Self(alloy_primitives::U128::MAX);
            
            /// 转换为小端字节数组
            pub fn to_le_bytes(&self) -> [u8; 16] {
                self.0.to_le_bytes()
            }
            
            /// 从小端字节数组创建
            pub fn from_le_slice(slice: &[u8]) -> Self {
                Self(alloy_primitives::U128::from_le_slice(slice))
            }
        }
    };
    
    // u64 特定方法
    ($wrapper_name:ident, u64) => {
        impl $wrapper_name {
            /// 创建一个零值
            pub const ZERO: Self = Self(0);
            
            /// 创建一个最大值
            pub const MAX: Self = Self(u64::MAX);
        }
    };
    
    // Address 特定方法
    ($wrapper_name:ident, Address) => {
        impl $wrapper_name {
            /// 创建一个零地址
            pub const ZERO: Self = Self(alloy_primitives::Address::ZERO);
            
            /// 获取原始字节
            pub fn as_bytes(&self) -> &[u8; 20] {
                self.0.as_ref()
            }
            
            /// 从字节数组创建
            pub fn from_bytes(bytes: [u8; 20]) -> Self {
                Self(alloy_primitives::Address::from(bytes))
            }
        }
    };
}

// =============================================================================
// 生成包装类型 - 参考 uint_types.rs 的简洁模式
// =============================================================================

// 使用宏生成 U256Wrapper
define_primitive_wrapper!(U256Wrapper, alloy_primitives::U256, alloy_uint);
impl_wrapper_utils!(U256Wrapper, U256);

// 使用宏生成 U128Wrapper  
define_primitive_wrapper!(U128Wrapper, alloy_primitives::U128, alloy_uint);
impl_wrapper_utils!(U128Wrapper, U128);

// 使用宏生成 U64Wrapper
define_primitive_wrapper!(U64Wrapper, u64, simple);
impl_wrapper_utils!(U64Wrapper, u64);

// 使用宏生成 AddressWrapper
define_primitive_wrapper!(AddressWrapper, alloy_primitives::Address, alloy_primitive);
impl_wrapper_utils!(AddressWrapper, Address);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_wrapper() {
        // 测试基本转换
        let val = alloy_primitives::U256::from(12345u64);
        let wrapper = U256Wrapper::from(val);
        assert_eq!(wrapper.0, val);
        
        let back: alloy_primitives::U256 = wrapper.into();
        assert_eq!(back, val);
        
        // 测试 Display
        assert_eq!(wrapper.to_string(), "12345");
        
        // 测试从 u64 创建
        let wrapper2 = U256Wrapper::from(12345u64);
        assert_eq!(wrapper, wrapper2);
        
        // 测试常量
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
        // 测试 serde 序列化
        let wrapper = U256Wrapper::from(12345u64);
        let json = serde_json::to_string(&wrapper).unwrap();
        let deserialized: U256Wrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(wrapper, deserialized);
    }
}