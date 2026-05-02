use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::traits::LogQuery;
use crate::AppState;

#[derive(Deserialize)]
pub struct RequestLogQuery {
    pub port: Option<u16>,
    pub mock_api_id: Option<i64>,
    pub path: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Deserialize)]
pub struct SystemLogQuery {
    pub level: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn list_request_logs(
    State(state): State<AppState>,
    Query(q): Query<RequestLogQuery>,
) -> impl IntoResponse {
    let query = LogQuery {
        port: q.port,
        mock_api_id: q.mock_api_id,
        path: q.path,
        since: q.since,
        until: q.until,
        page: q.page.unwrap_or(0),
        page_size: q.page_size.unwrap_or(50),
        level: None,
    };
    match state.log_store.list_request_logs(query).await {
        Ok(page) => Json(page).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_request_log(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.log_store.get_request_log(id).await {
        Ok(Some(log)) => Json(log).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn clear_request_logs(State(state): State<AppState>) -> impl IntoResponse {
    match state.log_store.clear_request_logs().await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn list_system_logs(
    State(state): State<AppState>,
    Query(q): Query<SystemLogQuery>,
) -> impl IntoResponse {
    let query = LogQuery {
        level: q.level,
        since: q.since,
        until: q.until,
        page: q.page.unwrap_or(0),
        page_size: q.page_size.unwrap_or(50),
        port: None,
        mock_api_id: None,
        path: None,
    };
    match state.log_store.list_system_logs(query).await {
        Ok(page) => Json(page).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn clear_system_logs(State(state): State<AppState>) -> impl IntoResponse {
    match state.log_store.clear_system_logs().await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
