use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::{LogEvent, StateResource};

#[derive(Deserialize)]
pub struct CreatePortBody {
    pub port: u16,
    pub label: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePortBody {
    pub label: String,
    pub enabled: bool,
}

#[derive(Serialize)]
struct PortStatusResponse {
    running: bool,
}

pub async fn list_ports(State(state): State<AppState>) -> impl IntoResponse {
    match state.port_store.list_ports().await {
        Ok(ports) => Json(ports).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_port(
    State(state): State<AppState>,
    Json(body): Json<CreatePortBody>,
) -> impl IntoResponse {
    let label = body.label.unwrap_or_default();
    match state.port_store.create_port(body.port, &label).await {
        Ok(p) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            (StatusCode::CREATED, Json(p)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn get_port(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.port_store.get_port(id).await {
        Ok(Some(p)) => Json(p).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_port(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePortBody>,
) -> impl IntoResponse {
    match state.port_store.update_port(id, &body.label, body.enabled).await {
        Ok(p) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            Json(p).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn delete_port(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // Stop the server first if running.
    let _ = state.port_manager.stop_port(id).await;
    match state.port_store.delete_port(id).await {
        Ok(()) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn start_port(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.port_manager.start_port(id).await {
        Ok(()) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn stop_port(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.port_manager.stop_port(id).await {
        Ok(()) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn port_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let running = state.port_manager.is_running(id).await;
    Json(PortStatusResponse { running })
}
