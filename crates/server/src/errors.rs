use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }
}

pub fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new("bad_request", message)),
    )
}

pub fn unsupported_media_type() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        Json(ErrorResponse::new(
            "unsupported_media_type",
            "Unsupported Content-Type",
        )),
    )
}

pub fn not_acceptable() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_ACCEPTABLE,
        Json(ErrorResponse::new(
            "not_acceptable",
            "Unsupported Accept header",
        )),
    )
}

pub fn too_many_requests(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(ErrorResponse::new("rate_limited", message)),
    )
}

pub fn service_unavailable(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse::new("service_unavailable", message)),
    )
}
