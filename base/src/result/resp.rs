use crate::result::{AppError, DynErrCode, ErrorCode, SysErr};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RespData<T> {
    pub code: String,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> RespData<T> {
    pub fn success(data: T) -> Self {
        let success = SysErr::Success;
        Self {
            code: success.code().into(),
            msg: success.message().into(),
            data: Some(data),
        }
    }
}

impl RespData<()> {
    pub fn with_code(code: &DynErrCode, ext: &str) -> Self {
        Self {
            code: code.code().into(),
            msg: format!("{} {}", code.message(), ext),
            data: None,
        }
    }

    pub fn with_anyhow(code: &DynErrCode, ext: &str, e: anyhow::Error) -> Self {
        Self {
            code: code.code().into(),
            msg: format!("{} {}: {}", code.message(), ext, e),
            data: None,
        }
    }

    pub fn with_app_error(error: AppError) -> Self {
        match error {
            AppError::ErrCode(code, ext) => Self::with_code(code, ext),
            AppError::Anyhow(code, ext, e) => Self::with_anyhow(code, ext, e),
            #[cfg(feature = "http")]
            AppError::HttpErr(code, status) => Self::with_code(code, &status.to_string()),
        }
    }

    pub fn with(code: &str, msg: &str) -> Self {
        Self {
            code: code.into(),
            msg: msg.into(),
            data: None,
        }
    }
}
