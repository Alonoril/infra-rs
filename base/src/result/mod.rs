mod code;
mod error;
mod resp;

pub use code::*;
pub use error::*;
pub use resp::*;

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
