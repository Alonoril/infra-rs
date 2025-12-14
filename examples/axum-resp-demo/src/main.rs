use axum::{Router, routing::get};
use axum_resp_macro::resp_data;
use base_infra::err;
use base_infra::result::{AppError, AppResult};
use serde::Serialize;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use web_infra::result::WebErr;

#[resp_data]
async fn ret_empty() -> base_infra::result::AppResult<()> {
	empty().await
}

async fn empty() -> AppResult<()> {
	println!("empty response");
	Ok(())
}

#[derive(Debug, Serialize)]
struct User {
	name: String,
	age: u8,
}

async fn user_info() -> AppResult<Option<User>> {
	let user = Some(User {
		name: "Zimu".to_string(),
		age: 30,
	});
	Ok(user)
}

#[resp_data]
async fn get_user() -> AppResult<Option<User>> {
	user_info().await
}

#[resp_data]
async fn user_null() -> AppResult<Option<User>> {
	let user = None;
	Ok::<_, AppError>(user)
}

#[resp_data]
async fn user_err() -> AppResult<Option<User>> {
	err!(&WebErr::NotFound, "user not found")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let app = Router::new()
		.route("/empty", get(ret_empty))
		.route("/user", get(get_user))
		.route("/user-null", get(user_null))
		.route("/user-error", get(user_err));

	let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
	println!("Server running on http://127.0.0.1:3000");
	axum::serve(TcpListener::bind(addr).await?, app.into_make_service()).await?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use axum::response::IntoResponse;
	use axum_resp_macro::resp_data;
	use base_infra::result::SysErr;
	use serde::Serialize;
	use std::time::Duration;
	use tokio::time::sleep;

	#[derive(Debug, Serialize)]
	struct BalanceResp {
		user: String,
		balance: u64,
	}

	#[resp_data]
	async fn query_balance(user: String, should_fail: bool) -> AppResult<BalanceResp> {
		if should_fail {
			base_infra::err!(&SysErr::InvalidParams)
		} else {
			sleep(Duration::from_millis(10)).await;
			Ok(BalanceResp { user, balance: 42 })
		}
	}

	#[tokio::test]
	async fn test_resp_data() {
		let ok_resp = query_balance("alice".into(), false)
			.await
			.expect("handler should succeed")
			.into_response();
		println!("success http status: {}", ok_resp.status());

		match query_balance("bob".into(), true).await {
			Ok(resp) => {
				let resp = resp.into_response();
				println!("unexpected success http status: {}", resp.status());
			}
			Err(err) => {
				let resp = err.into_response();
				println!("error http status: {}", resp.status());
			}
		}
	}
}
