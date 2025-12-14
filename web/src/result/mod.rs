mod axum;
mod error;
pub mod pagination;

pub use axum::*;
use base_infra::result::RespData;
pub use error::*;
use serde::Serialize;
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

pub type AxumResult<T> = Result<T, AxumError>;

#[cfg(feature = "utoipa")]
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AxumResp<T: ToSchema> {
	code: String,
	msg: String,
	data: Option<T>,
}
#[cfg(not(feature = "utoipa"))]
#[derive(Debug, Clone, Serialize)]
pub struct AxumResp<T> {
	code: String,
	msg: String,
	data: Option<T>,
}

#[cfg(feature = "utoipa")]
impl<T: ToSchema> From<RespData<T>> for AxumResp<T> {
	fn from(value: RespData<T>) -> Self {
		Self {
			code: value.code,
			msg: value.msg,
			data: value.data,
		}
	}
}
#[cfg(not(feature = "utoipa"))]
impl<T> From<RespData<T>> for AxumResp<T> {
	fn from(value: RespData<T>) -> Self {
		Self {
			code: value.code,
			msg: value.msg,
			data: value.data,
		}
	}
}

/// Ok(AppJson(RespData::success(admin)))
#[macro_export]
macro_rules! success {
	($data:expr) => {{
		tracing::debug!(response_data=?$data);
		Ok($crate::result::AppJson(base_infra::result::RespData::success($data)))
	}};
}

/// return Err(AxumError::*)
#[macro_export]
macro_rules! fail {
	($code:expr) => {{
		tracing::error!("ErrCode[{}]", $code);
		Err($crate::result::AxumError::AppError(
			base_infra::result::AppError::ErrCode($code),
		))
	}};

	($code:expr, $err:expr) => {{
		tracing::error!("ErrCode[{}] error msg: {}", $code, $err);
		Err($crate::result::AxumError::AppError(
			base_infra::result::AppError::Anyhow($code, anyhow::anyhow!($err)),
		))
	}};

	($code:expr, http $status:expr) => {{
		tracing::error!("ErrCode[{}] http status: {}", $code, $status);
		Err($crate::result::AxumError::AppError(
			base_infra::result::AppError::HttpErr($code, $status),
		))
	}};
}

#[macro_export]
macro_rules! map_http_err {
	($status:expr) => {
		|err| {
			tracing::error!("Http Status[{}] ErrCode: {}", $status, err);
			let app_err = match err {
				base_infra::result::AppError::ErrCode(code) => {
					base_infra::result::AppError::HttpErr(code, $status)
				}
				base_infra::result::AppError::Anyhow(code, _) => {
					base_infra::result::AppError::HttpErr(code, $status)
				}
				base_infra::result::AppError::HttpErr(code, _) => {
					base_infra::result::AppError::HttpErr(code, $status)
				}
			};
			$crate::result::AxumError::AppError(app_err)
		}
	};
}
