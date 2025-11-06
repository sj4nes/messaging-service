use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;

pub async fn list_conversations(
    State(_state): State<crate::AppState>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Stubbed list with paging metadata
    let payload = json!({
        "items": [],
        "page": 1,
        "pageSize": 0,
        "total": 0
    });
    (StatusCode::OK, Json(payload))
}

pub async fn list_messages(
    State(_state): State<crate::AppState>,
    Path(_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let payload = json!({
        "items": [],
        "page": 1,
        "pageSize": 0,
        "total": 0
    });
    (StatusCode::OK, Json(payload))
}
