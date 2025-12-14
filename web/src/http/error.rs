use crate::HTTP_TIMEOUT;
use crate::result::WebErr;
use anyhow::anyhow;
use axum::response::IntoResponse;
use axum::{BoxError, Json};
use base_infra::result::RespData;
use http::StatusCode;

/// Adds a custom handler for tower's `TimeoutLayer`, see https://docs.rs/axum/latest/axum/middleware/index.html#commonly-used-middleware.
pub async fn handle_timeout_error(err: BoxError) -> impl IntoResponse {
	if err.is::<tower::timeout::error::Elapsed>() {
		let timeout_err = anyhow!(
			"request took longer than the configured {} second timeout",
			*HTTP_TIMEOUT
		);

		(
			StatusCode::REQUEST_TIMEOUT,
			Json(RespData::with_anyhow(&WebErr::RequestTimeout, timeout_err)),
		)
	} else {
		let err = anyhow!("unhandled internal error: {}", err);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(RespData::with_anyhow(&WebErr::InternalServerError, err)),
		)
	}
}

pub async fn handle_404() -> impl IntoResponse {
	(
		StatusCode::NOT_FOUND,
		Json(RespData::with_code(&WebErr::NotFound)),
	)
}
