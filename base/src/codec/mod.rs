#[cfg(feature = "bincode")]
pub mod bincode;
pub mod error;
#[cfg(feature = "rkyv-codec")]
pub mod rkyv;
