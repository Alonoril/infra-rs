use crate::result::{DynErrCode, ErrorCode, SysErr};
use anyhow::anyhow;
#[cfg(feature = "http")]
use http::StatusCode;
use std::fmt::{Debug, Display, Formatter};

/// map_err with ErrCode to AppError with log
#[macro_export]
macro_rules! only_code {
    ($code:expr) => {
        |err| {
            tracing::error!("{}, reason: {}", $code, err);
            $crate::result::AppError::Anyhow($code, anyhow::anyhow!(err))
        }
    };
}

/// map_err with ErrCode and any Error to AppError
///
/// use for `.map_err(map_err!(ErrCode::InternalError))`
#[macro_export]
macro_rules! map_err {
    ($code:expr) => {
        |err| {
            tracing::debug!("{}, reason: {:?}", $code, err);
            tracing::error!("{}, reason: {}", $code, err);
            $crate::result::AppError::Anyhow($code, anyhow::anyhow!(err))
        }
    };

    ($code:expr, $msg:expr) => {
        |err| {
            tracing::debug!("{} {}, reason: {:?}", $code, $msg, err);
            tracing::error!("{} {}, reason: {}", $code, $msg, err);
            let msg = ($msg).to_string();
            $crate::result::AppError::ExtAnyhow($code, msg, anyhow::anyhow!(err))
        }
    };

    ($code:expr, any $msg:expr) => {{
        tracing::error!("(via any_err_ext) {}", $code);
        $crate::result::any_err_ext($code, $msg)
    }};

    (http $status:expr) => {
        |err| {
            tracing::debug!("Http Status[{}]: {:?}", $status, err);
            tracing::error!("Http Status[{}]: {}", $status, err);
            match err {
                $crate::result::AppError::ErrCode(code) => {
                    $crate::result::AppError::HttpErr(code, $status)
                }
                $crate::result::AppError::ExtCode(code, _ext) => {
                    $crate::result::AppError::HttpErr(code, $status)
                }
                $crate::result::AppError::Anyhow(code, _) => {
                    $crate::result::AppError::HttpErr(code, $status)
                }
                $crate::result::AppError::ExtAnyhow(code, _, _) => {
                    $crate::result::AppError::HttpErr(code, $status)
                }
                $crate::result::AppError::HttpErr(code, _) => {
                    $crate::result::AppError::HttpErr(code, $status)
                }
            }
        }
    };
}

/// use this macro to return Err(AppError)
///
/// return Err(AppError::*)
#[macro_export]
macro_rules! err {
    ($code:expr) => {{
        tracing::error!("{}", $code);
        Err($crate::result::AppError::ErrCode($code))
    }};

    ($code:expr, $msg:expr) => {{
        tracing::error!("{} {}", $code, $msg);
        let msg = ($msg).to_string();
        Err($crate::result::AppError::ExtCode($code, msg))
    }};
}

#[macro_export]
macro_rules! log_err {
    ($code:expr, $msg:expr) => {{
        tracing::error!("{} {}", $code, $msg);
        $crate::err!($code)
    }};
}

/// use this macro to return AppError
///
/// return AppError::*
#[macro_export]
macro_rules! app_err {
    ($code:expr) => {{
        tracing::error!("{}", $code);
        $crate::result::AppError::ErrCode($code)
    }};

    ($code:expr, $msg:expr) => {{
        tracing::error!("{} {}", $code, $msg);
        let msg = ($msg).to_string();
        $crate::result::AppError::ExtCode($code, msg)
    }};
}

// #[macro_export]
// macro_rules! log_app_err {
//     ($code:expr, $msg:expr) => {{
//         tracing::error!("{} {}", $code, $msg);
//         $crate::app_err!($code)
//     }};
// }

/// use this macro to Option .ok_or_else
///
/// `option.ok_or_else(else_err!(&AccountDaoErr::TransactionNotFound))?;`
#[macro_export]
macro_rules! else_err {
    ($code:expr) => {
        || {
            tracing::error!("{}", $code);
            $crate::result::AppError::ErrCode($code)
        }
    };

    ($code:expr, $msg:expr) => {
        || {
            tracing::error!("{} {}", $code, $msg);
            let msg = ($msg).to_string();
            $crate::result::AppError::ExtCode($code, msg)
        }
    };
}

#[macro_export]
macro_rules! or_err {
    ($code:expr) => {{
        tracing::error!("{}", $code);
        $crate::result::AppError::ErrCode($code)
    }};

    ($code:expr, $msg:expr) => {{
        tracing::error!("{} {}", $code, $msg);
        let msg = ($msg).to_string();
        $crate::result::AppError::ExtCode($code, msg)
    }};
}

/// map_err(E any error) to AppError
pub fn any_err<E>(code: &'static DynErrCode) -> impl FnOnce(E) -> AppError
where
    E: std::error::Error + Into<anyhow::Error>,
{
    // move |source| AppError::Anyhow(code, anyhow!("{}", source))
    move |err| {
        tracing::debug!("{}, reason: {:?}", code, err);
        tracing::error!("{}, reason: {}", code, err);
        AppError::Anyhow(code, anyhow!(err))
    }
}

/// map_err(E any error) to AppError
pub fn any_err_ext<E, S>(code: &'static DynErrCode, msg: S) -> impl FnOnce(E) -> AppError
where
    E: std::error::Error + Into<anyhow::Error>,
    S: Into<String> + Display,
{
    move |err| {
        tracing::error!("{} {}, reason: {}", code, msg, err);
        AppError::Anyhow(code, anyhow!(err))
    }
}

#[derive(thiserror::Error)]
pub enum AppError {
    ErrCode(&'static DynErrCode),
    ExtCode(&'static DynErrCode, String),
    Anyhow(&'static DynErrCode, anyhow::Error),
    ExtAnyhow(&'static DynErrCode, String, anyhow::Error),
    #[cfg(feature = "http")]
    HttpErr(&'static DynErrCode, StatusCode),
}

impl Debug for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ErrCode(code) => f
                .debug_struct("AppError")
                .field("code", &code.code())
                .field("msg", &code.message())
                .finish(),
            AppError::ExtCode(code, ext) => f
                .debug_struct("AppError")
                .field("code", &code.code())
                .field("msg", &format!("{} {}", &code.message(), ext))
                .finish(),
            AppError::Anyhow(code, e) => f
                .debug_struct("AppError")
                .field("code", &code.code())
                .field("msg", &code.message())
                .field("error", e)
                .finish(),
            AppError::ExtAnyhow(code, ext, e) => f
                .debug_struct("AppError")
                .field("code", &code.code())
                .field("msg", &format!("{} {}", &code.message(), ext))
                .field("error", e)
                .finish(),
            #[cfg(feature = "http")]
            AppError::HttpErr(code, status) => f
                .debug_struct("AppError")
                .field("http_status", status)
                .field("code", &code.code())
                .field("msg", &code.message())
                .finish(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ErrCode(code) => {
                write!(f, "ErrCode[{}] {}", code.code(), code.message())
            }
            AppError::ExtCode(code, ext) => {
                write!(f, "ErrCode[{}] {} {}", code.code(), code.message(), ext)
            }
            AppError::Anyhow(code, e) => {
                write!(f, "ErrCode[{}] {}, error: {e}", code.code(), code.message(),)
            }
            AppError::ExtAnyhow(code, ext, e) => {
                write!(
                    f,
                    "ErrCode[{}] {} {}, error: {e}",
                    code.code(),
                    code.message(),
                    ext
                )
            }
            #[cfg(feature = "http")]
            AppError::HttpErr(code, status) => {
                write!(
                    f,
                    "HttpStatus [{}], ErrCode: [{}] message: {}",
                    status,
                    code.code(),
                    code.message()
                )
            }
        }
    }
}

impl AppError {
    pub fn get_reason(&self) -> String {
        match self {
            AppError::ErrCode(code) => format!("{}", &code.message()),
            AppError::ExtCode(code, ext) => format!("{} {ext}", &code.message()),
            AppError::Anyhow(code, e) => format!("{}, reason: {e}", code.message()),
            AppError::ExtAnyhow(code, ext, e) => format!("{} {ext}, reason: {e}", code.message()),
            #[cfg(feature = "http")]
            AppError::HttpErr(code, status) => format!(
                "HttpStatus [{}] ErrCode[{}] message: {}",
                status,
                code.code(),
                code.message()
            ),
        }
    }
}

impl<T: ErrorCode> From<&'static T> for AppError {
    fn from(value: &'static T) -> Self {
        AppError::ErrCode(value)
    }
}
impl<T: ErrorCode> From<(&'static T, anyhow::Error)> for AppError {
    fn from(value: (&'static T, anyhow::Error)) -> Self {
        AppError::Anyhow(value.0, value.1)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Anyhow(&SysErr::InternalError, err)
    }
}
