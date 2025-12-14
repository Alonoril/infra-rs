#[cfg(feature = "alloy-primitives")]
pub mod primitives;
mod simple_map;
pub mod task;

#[cfg(feature = "bincode")]
#[cfg(feature = "alloy-primitives")]
pub mod primitives_bincode;

pub use simple_map::*;
