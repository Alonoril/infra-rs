#[cfg(feature = "rayon-pool")]
mod rayon;
#[cfg(feature = "tokio-pool")]
mod task_util;
#[cfg(feature = "tokio-pool")]
mod tokio;

#[cfg(feature = "rayon-pool")]
pub use rayon::*;
#[cfg(feature = "tokio-pool")]
pub use task_util::*;
#[cfg(feature = "tokio-pool")]
pub use tokio::*;

#[cfg(any(feature = "tokio-pool", feature = "rayon-pool"))]
const MAX_THREAD_NAME_LENGTH: usize = 12;
