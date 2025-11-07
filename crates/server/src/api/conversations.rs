use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::types::{ConversationDto, ListResponse, MessageDto, PageMeta};

#[derive(Debug, Deserialize)]
pub struct PagingQuery {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(rename = "pageSize", default)]
    pub page_size: Option<u32>,
}

pub(crate) async fn list_conversations(
    State(_state): State<crate::AppState>,
    Query(paging): Query<PagingQuery>,
) -> (StatusCode, Json<ListResponse<ConversationDto>>) {
    let page = paging.page.unwrap_or(1);
    let page_size = paging.page_size.unwrap_or(0);
    let (items, total) = crate::store::conversations::list(page, page_size);
    let resp = ListResponse {
        items,
        meta: PageMeta {
            page,
            page_size,
            total,
        },
    };
    (StatusCode::OK, Json(resp))
}

pub(crate) async fn list_messages(
    State(_state): State<crate::AppState>,
    Path(id): Path<String>,
    Query(paging): Query<PagingQuery>,
) -> (StatusCode, Json<ListResponse<MessageDto>>) {
    let page = paging.page.unwrap_or(1);
    let page_size = paging.page_size.unwrap_or(0);
    let (items, total) = crate::store::conversations::list_messages(&id, page, page_size);
    let resp = ListResponse {
        items,
        meta: PageMeta {
            page,
            page_size,
            total,
        },
    };
    (StatusCode::OK, Json(resp))
}
