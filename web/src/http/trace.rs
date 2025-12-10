use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::header::CONTENT_TYPE;
use axum::middleware::Next;
use axum::response::Response;
use base_infra::utils::uuid::UID;
use std::time::Instant;
use tracing::{Instrument, info, info_span};

#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub user_agent: Option<String>,
    pub remote_addr: Option<String>,
    pub start_time: Instant,
}

impl RequestInfo {
    pub fn new(req: &Request) -> Self {
        let request_id = UID.v4_simple_str();
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let remote_addr = req
            .headers()
            .get("x-forwarded-for")
            .or_else(|| req.headers().get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            request_id,
            method,
            path,
            user_agent,
            remote_addr,
            start_time: Instant::now(),
        }
    }
}

fn should_log_body(req: &Request, body_bytes: &Bytes) -> bool {
    // Skip logging if body is too large
    if body_bytes.len() > 1024 * 10 {
        // 10KB limit
        return false;
    }

    // Check content-type; only log text types
    if let Some(content_type) = req.headers().get(CONTENT_TYPE) {
        if let Ok(content_type_str) = content_type.to_str() {
            return content_type_str.starts_with("application/json")
                || content_type_str.starts_with("text/")
                || content_type_str.starts_with("application/x-www-form-urlencoded");
        }
    }

    // If no content-type, try to detect UTF-8
    std::str::from_utf8(body_bytes).is_ok()
}

fn contains_sensitive_fields(body_str: &str) -> bool {
    let sensitive_fields = [
        // Private keys
        "privatekey",
        "private_key",
        "pri_key",
        "prikey",
        "priv_key",
        "sk",
        "secretkey",
        "secret_key",
        // Passwords
        "password",
        "pwd",
        "pass",
        "passwd",
        "passwork",
        // Tokens
        // "token", "accesstoken", "access_token", "authtoken", "auth_token",
        // "apikey", "api_key",
        // Other sensitive info
        "secret",
        "mnemonic",
        "seed",
        "wallet_key",
        "walletkey",
        "auth_key",
        "authkey",
        "credential",
        "credentials",
        // Signatures
        // "signature", "sign",
    ];

    let body_lower = body_str.to_lowercase();
    sensitive_fields
        .iter()
        .any(|&field| body_lower.contains(field))
}

// if !req.uri().path().starts_with("/api") {
//     return next.run(req).await;
// }
pub async fn http_trace(req: Request, next: Next) -> Response {
    let filter = ["/api/", "/v1/", "/v2/", "/v3/"];
    let ok = filter.into_iter().any(|p| req.uri().path().starts_with(p));
    if !ok {
        return next.run(req).await;
    }

    let request_info = RequestInfo::new(&req);
    // Split request parts and body
    let (parts, body) = req.into_parts();

    // Read request body
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_else(|_| Bytes::new());

    // Rebuild request to restore body
    let req = Request::from_parts(parts, Body::from(body_bytes.clone()));

    // Log body content (may need to check content-type)
    let body_str = if should_log_body(&req, &body_bytes) {
        let body_content = String::from_utf8_lossy(&body_bytes).to_string();
        if contains_sensitive_fields(&body_content) {
            "<request contains sensitive data>".to_string()
        } else {
            body_content
        }
    } else {
        format!("<binary data {} bytes>", body_bytes.len())
    };

    // Create a span with request_id; subsequent API handlers run within it
    let span = info_span!(
        "api",
        // api = %request_info.path,
        tid = %request_info.request_id,
    );

    async move {
        info!(
            target: "http_request",
            path = %request_info.path,
            method = %request_info.method,
            user_agent = ?request_info.user_agent,
            remote_addr = ?request_info.remote_addr,
            request_body = %body_str,
            ">>>Request started:"
        );

        let mut response = next.run(req).await;

        let duration = request_info.start_time.elapsed();
        let status_code = response.status().as_u16();

        // Add request-id to response headers
        response
            .headers_mut()
            .insert("request-id", request_info.request_id.parse().unwrap());

        info!(
            target: "http_request",
            status_code = status_code,
            duration_ms = duration.as_millis(),
            "<<<Request completed:"
        );

        response
    }
    .instrument(span)
    .await
}
