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
        Some(Command::Restart) => {
            daemon::restart(&cli.db, cli.port)?;
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
    if !daemon::is_external_daemon_running(&cli.db) {
        port_manager.start_all_enabled().await?;
    }

    let state = AppState {
        mock_store,
        port_store,
        log_store,
        port_manager: port_manager.clone(),
        log_tx,
        management_port: cli.port,
    };

    match cli.command {
        Some(Command::Serve) => {
            // Foreground server: ports + dashboard, no TUI.
            // Runs until SIGINT or SIGTERM.
            run_serve(state, cli.port).await?;
        }
        _ => {
            tui::run(state).await?;
        }
    }

    port_manager.stop_all().await?;
    Ok(())
}

async fn run_serve(state: AppState, port: u16) -> anyhow::Result<()> {
    use tokio_util::sync::CancellationToken;

    let cancel = CancellationToken::new();
    let recon = tokio::spawn(reconciliation_loop(state.clone(), cancel.clone()));

    println!("Mock server running. Dashboard: http://localhost:{}", port);
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

    cancel.cancel();
    recon.await.ok();
    Ok(())
}

async fn reconciliation_loop(state: AppState, cancel: tokio_util::sync::CancellationToken) {
    let our_pid = std::process::id();
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = ticker.tick() => {}
        }
        let Ok(ports) = state.port_store.list_ports().await else { continue };
        for p in ports {
            if p.enabled {
                match p.owner_pid {
                    // No owner and not running: new port, start it.
                    None if !p.running => {
                        let _ = state.port_manager.start_port(p.id).await;
                    }
                    // Stale PID from a crashed process: clear and start.
                    Some(pid) if !crate::daemon::is_process_alive(pid) => {
                        let _ = state.port_store.set_port_running(p.id, false, None).await;
                        let _ = state.port_manager.start_port(p.id).await;
                    }
                    // Live owner (running or intentionally stopped): respect their decision.
                    _ => {}
                }
            } else if p.running && p.owner_pid == Some(our_pid) {
                let _ = state.port_manager.stop_port(p.id).await;
            }
        }
    }
}

/// Shared application state threaded through every subsystem.
#[derive(Clone)]
pub struct AppState {
    pub mock_store: Arc<dyn traits::MockStore>,
    pub port_store: Arc<dyn traits::PortStore>,
    pub log_store: Arc<dyn traits::LogStore>,
    pub port_manager: Arc<dyn PortManager>,
    pub log_tx: broadcast::Sender<LogEvent>,
    /// Management HTTP port (dashboard / API) — used by TUI to delegate to a running daemon.
    pub management_port: u16,
}
