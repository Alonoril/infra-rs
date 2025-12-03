use axum::extract::Query;
use crate::result::WebErr;
use axum::Json;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use base_infra::result::{AppError, RespData};

#[derive(Debug, thiserror::Error)]
pub enum AxumError {
    #[error("Request JSON error : {0}")]
    AxumJson(#[from] JsonRejection),
    #[error("Invalid query parameter: {0}")]
    AxumParams(#[from] QueryRejection),
    #[error("{0}")]
    AppError(#[from] AppError),
}

#[derive(axum_macros::FromRequest)]
#[from_request(via(axum::Json), rejection(AxumError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

#[derive(axum_macros::FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(AxumError))]
pub struct AxumQuery<T>(pub T);

impl<T> IntoResponse for AxumQuery<T>
where
    Query<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Query(self.0).into_response()
    }
}

impl IntoResponse for AxumError {
    fn into_response(self) -> Response {
        match self {
            AxumError::AxumJson(err) => {
                tracing::error!("ErrorCode[{}] reason: {:?}", WebErr::ReqJsonErr, err);
                let resp = RespData::with_anyhow(&WebErr::ReqJsonErr, err.into());
                (StatusCode::OK, AppJson(resp)).into_response()
            }
            AxumError::AxumParams(err) => {
                tracing::error!("ErrorCode[{}] reason: {:?}", WebErr::QueryParamsErr, err);
                let resp = RespData::with_anyhow(&WebErr::QueryParamsErr, err.into());
                (StatusCode::OK, AppJson(resp)).into_response()
            }
            AxumError::AppError(err) => match err {
                AppError::ErrCode(code) => {
                    (StatusCode::OK, AppJson(RespData::with_code(code))).into_response()
                }
                AppError::ExtCode(code, ext) => {
                    (StatusCode::OK, AppJson(RespData::with_ext_code(code, ext))).into_response()
                }
                AppError::Anyhow(code, e) => {
                    (StatusCode::OK, AppJson(RespData::with_anyhow(code, e))).into_response()
                }
                AppError::ExtAnyhow(code, ext, e) => (
                    StatusCode::OK,
                    AppJson(RespData::with_ext_anyhow(code, ext, e)),
                )
                    .into_response(),
                AppError::HttpErr(code, status) => {
                    (status, AppJson(RespData::with_code(code))).into_response()
                }
            },
        }
    }
}
