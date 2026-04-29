mod cli;
mod dashboard;
mod db;
mod error;
mod logging;
mod models;
mod server;
mod traits;
mod tui;

use std::sync::Arc;

use clap::Parser;
use tokio::sync::broadcast;

use crate::cli::Cli;
use crate::db::{SqliteLogStore, SqliteMockStore, SqlitePortStore};
use crate::models::LogEvent;
use crate::server::manager::LivePortManager;
use crate::traits::PortManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Open DB + run migrations.
    let conn = db::open(&cli.db).await?;

    // Construct stores.
    let mock_store: Arc<dyn traits::MockStore> =
        Arc::new(SqliteMockStore::new(conn.clone()));
    let port_store: Arc<dyn traits::PortStore> =
        Arc::new(SqlitePortStore::new(conn.clone()));
    let log_store: Arc<dyn traits::LogStore> =
        Arc::new(SqliteLogStore::new(conn.clone()));

    // Broadcast channel for real-time log events.
    let (log_tx, _) = broadcast::channel::<LogEvent>(1024);

    // Initialise tracing (logs to stderr + writes SystemLog rows to DB).
    logging::init(log_store.clone(), log_tx.clone());

    // Build the port manager and start all enabled mock servers.
    let port_manager: Arc<dyn PortManager> = Arc::new(LivePortManager::new(
        port_store.clone(),
        mock_store.clone(),
        log_store.clone(),
        log_tx.clone(),
    ));
    port_manager.start_all_enabled().await?;

    let state = AppState {
        mock_store,
        port_store,
        log_store,
        port_manager: port_manager.clone(),
        log_tx,
    };

    if cli.dashboard {
        dashboard::run(state, cli.port).await?;
    } else {
        tui::run(state).await?;
    }

    port_manager.stop_all().await?;
    Ok(())
}

/// Shared application state threaded through every subsystem.
#[derive(Clone)]
pub struct AppState {
    pub mock_store: Arc<dyn traits::MockStore>,
    pub port_store: Arc<dyn traits::PortStore>,
    pub log_store: Arc<dyn traits::LogStore>,
    pub port_manager: Arc<dyn PortManager>,
    pub log_tx: broadcast::Sender<LogEvent>,
}
