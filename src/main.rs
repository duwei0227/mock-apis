mod cli;
mod daemon;
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

use crate::cli::{Cli, Command};
use crate::db::{SqliteLogStore, SqliteMockStore, SqlitePortStore};
use crate::models::LogEvent;
use crate::server::manager::LivePortManager;
use crate::traits::PortManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Start) => {
            daemon::start(&cli.db, cli.port)?;
            return Ok(());
        }
        Some(Command::Stop) => {
            daemon::stop(&cli.db)?;
            return Ok(());
        }
        Some(Command::Status) => {
            daemon::status(&cli.db);
            return Ok(());
        }
        Some(Command::Serve) | None => {
            // Fall through to full server startup below.
        }
    }

    // Open DB + run migrations.
    let conn = db::open(&cli.db).await?;

    let mock_store: Arc<dyn traits::MockStore> = Arc::new(SqliteMockStore::new(conn.clone()));
    let port_store: Arc<dyn traits::PortStore> = Arc::new(SqlitePortStore::new(conn.clone()));
    let log_store: Arc<dyn traits::LogStore> = Arc::new(SqliteLogStore::new(conn.clone()));

    let (log_tx, _) = broadcast::channel::<LogEvent>(1024);
    logging::init(log_store.clone(), log_tx.clone());

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

    match cli.command {
        Some(Command::Serve) => {
            // Foreground server: ports + dashboard, no TUI.
            // Runs until SIGINT or SIGTERM.
            run_serve(state, cli.port).await?;
        }
        _ => {
            if cli.dashboard {
                dashboard::run(state, cli.port).await?;
            } else {
                tui::run(state).await?;
            }
        }
    }

    port_manager.stop_all().await?;
    Ok(())
}

async fn run_serve(state: AppState, port: u16) -> anyhow::Result<()> {
    let db_url = format!("http://localhost:{}", port);
    println!("Mock server running. Dashboard: {}", db_url);
    println!("Press Ctrl+C to stop.");

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate())?;
        tokio::select! {
            _ = dashboard::run(state, port) => {}
            _ = sigterm.recv() => { println!("\nReceived SIGTERM, stopping..."); }
            _ = tokio::signal::ctrl_c() => { println!("\nStopping..."); }
        }
    }
    #[cfg(not(unix))]
    {
        tokio::select! {
            _ = dashboard::run(state, port) => {}
            _ = tokio::signal::ctrl_c() => { println!("\nStopping..."); }
        }
    }

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
