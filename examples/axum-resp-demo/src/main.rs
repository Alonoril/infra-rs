use std::time::Duration;

use axum::response::IntoResponse;
use axum_resp_macro::axum_resp;
use base_infra::result::SysErr;
use serde::Serialize;
use tokio::time::sleep;

#[derive(Debug, Serialize)]
struct BalanceResp {
    user: String,
    balance: u64,
}

#[axum_resp]
async fn query_balance(
    user: String,
    should_fail: bool,
) -> base_infra::result::AppResult<BalanceResp> {
    if should_fail {
        base_infra::err!(&SysErr::InvalidParams)
    } else {
        sleep(Duration::from_millis(10)).await;
        Ok(BalanceResp { user, balance: 42 })
    }
}

#[tokio::main]
async fn main() {
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
