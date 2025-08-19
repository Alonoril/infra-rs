use crate::result::{DynErrCode, ErrorCode, SysErr};
use anyhow::anyhow;
#[cfg(feature = "http")]
use http::StatusCode;
use std::fmt::{Debug, Display, Formatter};

/// map_err with ErrCode to AppError with log
#[macro_export]
macro_rules! only_code {
    ($code:expr) => {
        |e| {
            tracing::debug!("ErrCode[{}] reason: {:?}", $code, e);
            tracing::error!("ErrCode[{}] reason: {}", $code, e);
            $crate::result::AppError::ErrCode($code)
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
            tracing::debug!("ErrCode[{}] reason: {:?}", $code, err);
            tracing::error!("ErrCode[{}] reason: {}", $code, err);
            $crate::result::AppError::Anyhow($code, anyhow::anyhow!(err))
        }
    };

    ($code:expr, $ext:expr) => {
        |err| {
            tracing::debug!("ErrCode[{}] {}, reason: {:?}", $code, $ext, err);
            tracing::error!("ErrCode[{}] {}, reason: {}", $code, $ext, err);
            $crate::result::AppError::Anyhow($code, anyhow::anyhow!("{}: {}", $ext, err))
        }
    };

    (http $status:expr) => {
        |err| {
            tracing::debug!("Http Status[{}]: {:?}", $status, err);
            tracing::error!("Http Status[{}]: {}", $status, err);
            match err {
                $crate::result::AppError::ErrCode(code) => {
                    $crate::result::AppError::HttpErr(code, $status)
                }
                $crate::result::AppError::Anyhow(code, _) => {
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
        tracing::error!("ErrCode[{}]", $code);
        Err($crate::result::AppError::ErrCode($code))
    }};

    ($code:expr, $err:expr) => {{
        tracing::debug!("ErrCode[{}] error: {:?}", $code, $err);
        tracing::error!("ErrCode[{}] error: {}", $code, $err);
        Err($crate::result::AppError::Anyhow(
            $code,
            anyhow::anyhow!($err),
        ))
    }};
}

#[macro_export]
macro_rules! log_err {
    ($code:expr, $log:expr) => {{
        tracing::error!("ErrCode[{}] error msg: {}", $code, $log);
        $crate::err!($code)
    }};
}

/// use this macro to return AppError
///
/// return AppError::*
#[macro_export]
macro_rules! app_err {
    ($code:expr) => {{ $crate::result::AppError::ErrCode($code) }};

    ($code:expr, $msg:expr) => {{ $crate::result::AppError::Anyhow($code, anyhow::anyhow!($msg)) }};
}

#[macro_export]
macro_rules! log_app_err {
    ($code:expr, $log:expr) => {{
        tracing::error!("ErrCode[{}] error msg: {}", $code, $log);
        $crate::app_err!($code)
    }};
}

/// use this macro to Option .ok_or_else
///
/// `option.ok_or_else(else_err!(&AccountDaoErr::TransactionNotFound))?;`
#[macro_export]
macro_rules! else_err {
    ($code:expr) => {
        || {
            tracing::error!("ErrCode[{}]", $code);
            $crate::result::AppError::ErrCode($code)
        }
    };

    ($code:expr, $msg:expr) => {
        || {
            tracing::error!("ErrCode[{}] error: {}", $code, $msg);
            $crate::result::AppError::Anyhow($code, anyhow::anyhow!($msg))
        }
    };
}

#[macro_export]
macro_rules! or_err {
    ($code:expr) => {{
        tracing::error!("ErrCode[{}]", $code);
        $crate::result::AppError::ErrCode($code)
    }};

    ($code:expr, $msg:expr) => {{
        tracing::error!("ErrCode[{}] error msg: {}", $code, $msg);
        $crate::result::AppError::Anyhow($code, anyhow::anyhow!($msg))
    }};
}

/// map_err(E any error) to AppError
pub fn any_err<E>(code: &'static DynErrCode) -> impl FnOnce(E) -> AppError
where
    E: std::error::Error + Into<anyhow::Error>,
{
    // move |source| AppError::Anyhow(code, anyhow!("{}", source))
    move |source| {
        tracing::debug!("ErrCode[{}] reason: {:?}", code, source);
        tracing::error!("ErrCode[{}] reason: {}", code, source);
        AppError::Anyhow(code, anyhow!(source))
    }
}

#[derive(thiserror::Error)]
pub enum AppError {
    ErrCode(&'static DynErrCode),
    Anyhow(&'static DynErrCode, anyhow::Error),
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
            AppError::Anyhow(code, e) => f
                .debug_struct("AppError")
                .field("code", &code.code())
                .field("msg", &code.message())
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
            AppError::ErrCode(code) => write!(f, "ErrCode [{code}]"),
            AppError::Anyhow(code, e) => write!(f, "ErrCode [{code}], error: {e}"),
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
            AppError::ErrCode(code) => code.to_string(),
            AppError::Anyhow(_code, e) => format!("{e}"),
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
    fn from(value: anyhow::Error) -> Self {
        AppError::Anyhow(&SysErr::InternalError, value)
    }
}
