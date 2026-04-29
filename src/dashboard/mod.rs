pub mod routes;
pub mod static_files;
pub mod ws;

use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::error::Result;
use crate::AppState;

use routes::{logs, mocks, ports};

pub async fn run(state: AppState, mgmt_port: u16) -> Result<()> {
    let app = build_router(state);
    let addr = format!("0.0.0.0:{}", mgmt_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Dashboard listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    let api = Router::new()
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
