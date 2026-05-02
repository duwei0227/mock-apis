use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;

use crate::models::HttpMethod;
use crate::traits::{CreateMockRequest, UpdateMockRequest};
use crate::AppState;

#[derive(Deserialize)]
pub struct ListMocksQuery {
    pub port_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateMockBody {
    pub port_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub method: HttpMethod,
    pub path: String,
    pub request_schema: Option<serde_json::Value>,
    pub response_status: Option<u16>,
    pub response_headers: Option<HashMap<String, String>>,
    pub response_body: Option<String>,
    pub response_delay_ms: Option<u64>,
}

#[derive(Deserialize)]
pub struct UpdateMockBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub method: Option<HttpMethod>,
    pub path: Option<String>,
    pub request_schema: Option<Option<serde_json::Value>>,
    pub response_status: Option<u16>,
    pub response_headers: Option<HashMap<String, String>>,
    pub response_body: Option<String>,
    pub response_delay_ms: Option<u64>,
    pub enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct SetEnabledBody {
    pub enabled: bool,
}

pub async fn list_mocks(
    State(state): State<AppState>,
    Query(q): Query<ListMocksQuery>,
) -> impl IntoResponse {
    match state.mock_store.list_mocks(q.port_id).await {
        Ok(mocks) => Json(mocks).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_mock(
    State(state): State<AppState>,
    Json(body): Json<CreateMockBody>,
) -> impl IntoResponse {
    let req = CreateMockRequest {
        port_id: body.port_id,
        name: body.name,
        description: body.description.unwrap_or_default(),
        method: body.method,
        path: body.path,
        request_schema: body.request_schema,
        response_status: body.response_status.unwrap_or(200),
        response_headers: body.response_headers.unwrap_or_default(),
        response_body: body.response_body.unwrap_or_default(),
        response_delay_ms: body.response_delay_ms.unwrap_or(0),
    };
    match state.mock_store.create_mock(req).await {
        Ok(m) => {
            // Restart the port server so the new mock is active.
            let _ = state.port_manager.restart_port(m.port_id).await;
            (StatusCode::CREATED, Json(m)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn get_mock(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.mock_store.get_mock(id).await {
        Ok(Some(m)) => Json(m).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_mock(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateMockBody>,
) -> impl IntoResponse {
    let req = UpdateMockRequest {
        name: body.name,
        description: body.description,
        method: body.method,
        path: body.path,
        request_schema: body.request_schema,
        response_status: body.response_status,
        response_headers: body.response_headers,
        response_body: body.response_body,
        response_delay_ms: body.response_delay_ms,
        enabled: body.enabled,
    };
    match state.mock_store.update_mock(id, req).await {
        Ok(m) => {
            let _ = state.port_manager.restart_port(m.port_id).await;
            Json(m).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn delete_mock(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // Fetch port_id before deletion so we can restart the server.
    let port_id = state
        .mock_store
        .get_mock(id)
        .await
        .ok()
        .flatten()
        .map(|m| m.port_id);

    match state.mock_store.delete_mock(id).await {
        Ok(()) => {
            if let Some(pid) = port_id {
                let _ = state.port_manager.restart_port(pid).await;
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn set_mock_enabled(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<SetEnabledBody>,
) -> impl IntoResponse {
    let port_id = state
        .mock_store
        .get_mock(id)
        .await
        .ok()
        .flatten()
        .map(|m| m.port_id);

    match state.mock_store.set_mock_enabled(id, body.enabled).await {
        Ok(()) => {
            if let Some(pid) = port_id {
                let _ = state.port_manager.restart_port(pid).await;
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
