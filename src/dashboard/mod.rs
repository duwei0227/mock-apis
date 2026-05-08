pub mod routes;
pub mod static_files;
pub mod ws;

use std::net::UdpSocket;

use axum::routing::{get, patch, post};
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::error::Result;
use crate::AppState;

use routes::{info, logs, mocks, ports};

pub async fn run(state: AppState, mgmt_port: u16) -> Result<()> {
    let app = build_router(state);
    let addr = format!("0.0.0.0:{}", mgmt_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let display_ip = local_ip().unwrap_or_else(|| "127.0.0.1".to_owned());
    tracing::info!("Dashboard listening on http://{}:{}", display_ip, mgmt_port);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    Ok(())
}

fn local_ip() -> Option<String> {
    // Connect a UDP socket to a public address without sending data — the OS
    // picks the outbound interface and we read back the local address.
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

fn build_router(state: AppState) -> Router {
    let api = Router::new()
        // Info
        .route("/info", get(info::get_info))
        // Ports
        .route("/ports", get(ports::list_ports).post(ports::create_port))
        .route(
            "/ports/:id",
            get(ports::get_port)
                .put(ports::update_port)
                .delete(ports::delete_port),
        )
        .route("/ports/:id/start", post(ports::start_port))
        .route("/ports/:id/stop", post(ports::stop_port))
        .route("/ports/:id/restart", post(ports::restart_port))
        .route("/ports/:id/status", get(ports::port_status))
        // Mocks
        .route("/mocks", get(mocks::list_mocks).post(mocks::create_mock))
        .route(
            "/mocks/:id",
            get(mocks::get_mock)
                .put(mocks::update_mock)
                .delete(mocks::delete_mock),
        )
        .route("/mocks/:id/enabled", patch(mocks::set_mock_enabled))
        // Logs
        .route(
            "/logs/requests",
            get(logs::list_request_logs).delete(logs::clear_request_logs),
        )
        .route("/logs/requests/:id", get(logs::get_request_log))
        .route(
            "/logs/system",
            get(logs::list_system_logs).delete(logs::clear_system_logs),
        );

    Router::new()
        .nest("/api/v1", api)
        .route("/ws/logs", get(ws::ws_logs))
        .fallback(static_files::static_handler)
        .layer(CorsLayer::permissive())
        .with_state(state)
}
