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
    // 如果body太大，不记录
    if body_bytes.len() > 1024 * 10 {
        // 10KB限制
        return false;
    }

    // 检查content-type，只记录文本类型
    if let Some(content_type) = req.headers().get(CONTENT_TYPE) {
        if let Ok(content_type_str) = content_type.to_str() {
            return content_type_str.starts_with("application/json")
                || content_type_str.starts_with("text/")
                || content_type_str.starts_with("application/x-www-form-urlencoded");
        }
    }

    // 如果没有content-type，尝试判断是否是UTF-8
    std::str::from_utf8(body_bytes).is_ok()
}

fn contains_sensitive_fields(body_str: &str) -> bool {
    let sensitive_fields = [
        // 私钥相关
        "privatekey", "private_key", "pri_key", "prikey", "priv_key",
        "sk", "secretkey", "secret_key",
        // 密码相关
        "password", "pwd", "pass", "passwd", "passwork",
        // Token相关
        // "token", "accesstoken", "access_token", "authtoken", "auth_token",
        // "apikey", "api_key",
        // 其他敏感信息
        "secret", "mnemonic", "seed",
        "wallet_key", "walletkey", "auth_key", "authkey",
        "credential", "credentials",
        // 签名相关
        // "signature", "sign",
    ];
    
    let body_lower = body_str.to_lowercase();
    sensitive_fields.iter().any(|&field| body_lower.contains(field))
}

pub async fn http_trace(req: Request, next: Next) -> Response {
    let request_info = RequestInfo::new(&req);
    // 分离request parts和body
    let (parts, body) = req.into_parts();

    // 读取request body
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_else(|_| Bytes::new());

    // 重新构造request，恢复body
    let req = Request::from_parts(parts, Body::from(body_bytes.clone()));

    // 将body内容记录到日志（可能需要根据content-type判断是否记录）
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

    // 创建一个包含 request_id 的 span，所有后续的 API handler 都会在这个 span 中运行
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

        // 在响应头中添加 request-id
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
