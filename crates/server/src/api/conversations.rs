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
    State(state): State<crate::AppState>,
    Query(paging): Query<PagingQuery>,
) -> (StatusCode, Json<ListResponse<ConversationDto>>) {
    let page = paging.page.unwrap_or(1);
    let page_size = paging.page_size.unwrap_or(0);
    // If DB is available, read from database; else fallback to in-memory store
    let (items, total) = if let Some(pool) = state.db() {
        let limit = if page_size == 0 {
            100
        } else {
            page_size as i64
        };
        let offset = if page_size == 0 {
            0
        } else {
            ((page.max(1) - 1) * page_size) as i64
        };
        match crate::store_db::conversations::list_conversations(&pool, limit, offset).await {
            Ok(rows) => {
                let dtos: Vec<ConversationDto> = rows
                    .into_iter()
                    .map(|r| ConversationDto {
                        id: r.id.to_string(),
                        key: r.topic.unwrap_or_else(|| "".into()),
                        message_count: r.message_count as u32,
                        last_activity_at: r
                            .last_message_at
                            .map(|t| t.to_rfc3339())
                            .unwrap_or_else(|| "".into()),
                    })
                    .collect();
                // Total conversations count (approximate using number returned if page_size=0)
                let total_count = dtos.len() as u64;
                (dtos, total_count)
            }
            Err(_) => (Vec::new(), 0),
        }
    } else {
        crate::store::conversations::list(page, page_size)
    };
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
    State(state): State<crate::AppState>,
    Path(id): Path<String>,
    Query(paging): Query<PagingQuery>,
) -> (StatusCode, Json<ListResponse<MessageDto>>) {
    let page = paging.page.unwrap_or(1);
    let page_size = paging.page_size.unwrap_or(0);
    let (items, total) = if let Some(pool) = state.db() {
        let limit = if page_size == 0 {
            100
        } else {
            page_size as i64
        };
        let offset = if page_size == 0 {
            0
        } else {
            ((page.max(1) - 1) * page_size) as i64
        };
        match id.parse::<i64>() {
            Ok(conv_id) => {
                match crate::store_db::conversations::list_messages(&pool, conv_id, limit, offset)
                    .await
                {
                    Ok(rows) => {
                        let dtos: Vec<MessageDto> = rows
                            .into_iter()
                            .map(|m| MessageDto {
                                id: m.id.to_string(),
                                from: "".into(),
                                to: "".into(),
                                r#type: m.direction,
                                snippet: "".into(),
                                timestamp: m.received_at.unwrap_or(m.sent_at).to_rfc3339(),
                            })
                            .collect();
                        (dtos, 0)
                    }
                    Err(_) => (Vec::new(), 0),
                }
            }
            Err(_) => (Vec::new(), 0),
        }
    } else {
        crate::store::conversations::list_messages(&id, page, page_size)
    };
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
