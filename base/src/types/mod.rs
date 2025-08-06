mod simple_map;
pub mod task;
pub mod primitives;

#[cfg(feature = "bin-codec")]
pub mod primitives_bincode;

pub use simple_map::*;
