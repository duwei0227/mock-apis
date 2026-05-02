use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, Result};
use crate::models::LogEvent;
use crate::server::handler::{mock_fallback, MockHandlerState};
use crate::traits::{LogStore, MockStore, PortManager, PortStore};

type PortHandle = (JoinHandle<()>, CancellationToken);

pub struct LivePortManager {
    port_store: Arc<dyn PortStore>,
    mock_store: Arc<dyn MockStore>,
    log_store: Arc<dyn LogStore>,
    log_tx: broadcast::Sender<LogEvent>,
    handles: Mutex<HashMap<i64, PortHandle>>,
}

impl LivePortManager {
    pub fn new(
        port_store: Arc<dyn PortStore>,
        mock_store: Arc<dyn MockStore>,
        log_store: Arc<dyn LogStore>,
        log_tx: broadcast::Sender<LogEvent>,
    ) -> Self {
        Self {
            port_store,
            mock_store,
            log_store,
            log_tx,
            handles: Mutex::new(HashMap::new()),
        }
    }

    async fn spawn_server(&self, port_id: i64) -> Result<PortHandle> {
        let config = self
            .port_store
            .get_port(port_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("port id {}", port_id)))?;

        let mocks = self.mock_store.list_mocks(Some(port_id)).await?;
        let mocks: Vec<_> = mocks.into_iter().filter(|m| m.enabled).collect();

        let state = MockHandlerState {
            port: config.port,
            mocks: Arc::new(mocks),
            log_store: self.log_store.clone(),
            log_tx: self.log_tx.clone(),
        };

        let token = CancellationToken::new();
        let token_clone = token.clone();
        let port = config.port;

        let listener =
            tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
                .await
                .map_err(|e| AppError::Io(e))?;

        let app = axum::Router::new()
            .fallback(mock_fallback)
            .with_state(state)
            .into_make_service_with_connect_info::<SocketAddr>();

        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move { token_clone.cancelled().await })
                .await
                .ok();
        });

        Ok((handle, token))
    }
}

#[async_trait]
impl PortManager for LivePortManager {
    async fn start_port(&self, port_id: i64) -> Result<()> {
        let mut handles = self.handles.lock().await;
        if handles.contains_key(&port_id) {
            return Ok(());
        }
        let handle = self.spawn_server(port_id).await?;
        handles.insert(port_id, handle);
        Ok(())
    }

    async fn stop_port(&self, port_id: i64) -> Result<()> {
        let mut handles = self.handles.lock().await;
        if let Some((join, token)) = handles.remove(&port_id) {
            token.cancel();
            join.await.ok();
        }
        Ok(())
    }

    async fn restart_port(&self, port_id: i64) -> Result<()> {
        {
            let mut handles = self.handles.lock().await;
            if let Some((join, token)) = handles.remove(&port_id) {
                token.cancel();
                join.await.ok();
            }
        }
        // Re-check enabled state before restarting.
        let config = self.port_store.get_port(port_id).await?;
        if config.map(|c| c.enabled).unwrap_or(false) {
            let mut handles = self.handles.lock().await;
            let handle = self.spawn_server(port_id).await?;
            handles.insert(port_id, handle);
        }
        Ok(())
    }

    async fn is_running(&self, port_id: i64) -> bool {
        self.handles.lock().await.contains_key(&port_id)
    }

    async fn running_ports(&self) -> Vec<i64> {
        self.handles.lock().await.keys().copied().collect()
    }

    async fn start_all_enabled(&self) -> Result<()> {
        let ports = self.port_store.list_ports().await?;
        for p in ports.into_iter().filter(|p| p.enabled) {
            self.start_port(p.id).await?;
        }
        Ok(())
    }

    async fn stop_all(&self) -> Result<()> {
        let ids: Vec<i64> = {
            let handles = self.handles.lock().await;
            handles.keys().copied().collect()
        };
        for id in ids {
            self.stop_port(id).await?;
        }
        Ok(())
    }
}
