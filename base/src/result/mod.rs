mod code;
mod error;
mod resp;

pub use code::*;
pub use error::*;
pub use resp::*;
use std::fmt::Display;

pub type AppResult<T> = Result<T, AppError>;

#[macro_export]
macro_rules! some_or_err {
	($opt:expr, $code:expr) => {
		match $opt {
			Some(val) => val,
			None => {
				return $crate::err!($code);
			}
		}
	};
}

#[macro_export]
macro_rules! err_ctx {
	($e:expr, $msg:expr) => {
		$e.with_context(|| $msg)
	};
}

pub trait AnyhowCtx<T, C>
where
	C: Display + Send + Sync + 'static,
{
	fn with_ctx(self, f: impl FnOnce() -> C) -> anyhow::Result<T>;
}

impl<T, E, C> AnyhowCtx<T, C> for Result<T, E>
where
	E: std::error::Error + Send + Sync + 'static,
	C: Display + Send + Sync + 'static,
{
	fn with_ctx(self, f: impl FnOnce() -> C) -> anyhow::Result<T> {
		use anyhow::Context;
		self.with_context(f)
	}
}

#[cfg(test)]
mod tests {
	use crate::logger::init_tracing;
	use crate::nar_err;
	use crate::result::AnyhowCtx;

	crate::gen_impl_code_enum! {
		TstErr {
			TestErr = ("TEST01", "Test error"),
		}
	}
	#[test]
	fn test_with_ctx() {
		let guard = init_tracing();

		let path = "config.toml";
		let res = std::fs::read_to_string(path).with_ctx(nar_err!(&TstErr::TestErr));
		match res {
			Ok(cfg) => {
				tracing::info!("{}", cfg);
			}
			Err(e) => {
				tracing::error!("{}", e);
			}
		}
	}
}
