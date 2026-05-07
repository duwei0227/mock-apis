use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;
use crate::models::{HttpMethod, MockApi, PortConfig, RequestLog, SystemLog};

// ---------------------------------------------------------------------------
// PortStore
// ---------------------------------------------------------------------------

#[async_trait]
pub trait PortStore: Send + Sync {
    async fn list_ports(&self) -> Result<Vec<PortConfig>>;
    async fn get_port(&self, id: i64) -> Result<Option<PortConfig>>;
    async fn get_port_by_number(&self, port: u16) -> Result<Option<PortConfig>>;
    async fn create_port(&self, port: u16, label: &str) -> Result<PortConfig>;
    async fn update_port(&self, id: i64, label: &str, enabled: bool) -> Result<PortConfig>;
    async fn delete_port(&self, id: i64) -> Result<()>;
    async fn set_port_enabled(&self, id: i64, enabled: bool) -> Result<()>;
    async fn set_port_running(&self, id: i64, running: bool, owner_pid: Option<u32>) -> Result<()>;
}

// ---------------------------------------------------------------------------
// MockStore
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMockRequest {
    pub port_id: i64,
    pub name: String,
    pub description: String,
    pub method: HttpMethod,
    pub path: String,
    pub request_schema: Option<serde_json::Value>,
    pub response_status: u16,
    pub response_headers: HashMap<String, String>,
    pub response_body: String,
    pub response_delay_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateMockRequest {
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

#[async_trait]
pub trait MockStore: Send + Sync {
    async fn list_mocks(&self, port_id: Option<i64>) -> Result<Vec<MockApi>>;
    async fn get_mock(&self, id: i64) -> Result<Option<MockApi>>;
    async fn create_mock(&self, req: CreateMockRequest) -> Result<MockApi>;
    async fn update_mock(&self, id: i64, req: UpdateMockRequest) -> Result<MockApi>;
    async fn delete_mock(&self, id: i64) -> Result<()>;
    async fn set_mock_enabled(&self, id: i64, enabled: bool) -> Result<()>;
    /// Returns the best-matching enabled mock: exact method beats ANY; first match wins.
    async fn find_matching_mock(
        &self,
        port_id: i64,
        method: &HttpMethod,
        path: &str,
    ) -> Result<Option<MockApi>>;
}

// ---------------------------------------------------------------------------
// LogStore
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct LogQuery {
    pub port: Option<u16>,
    pub mock_api_id: Option<i64>,
    pub path: Option<String>,
    pub level: Option<String>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPage<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[async_trait]
pub trait LogStore: Send + Sync {
    async fn append_request_log(&self, log: RequestLog) -> Result<i64>;
    async fn list_request_logs(&self, query: LogQuery) -> Result<LogPage<RequestLog>>;
    async fn get_request_log(&self, id: i64) -> Result<Option<RequestLog>>;
    async fn clear_request_logs(&self) -> Result<()>;

    async fn append_system_log(&self, log: SystemLog) -> Result<i64>;
    async fn list_system_logs(&self, query: LogQuery) -> Result<LogPage<SystemLog>>;
    async fn clear_system_logs(&self) -> Result<()>;
}

// ---------------------------------------------------------------------------
// PortManager — runtime lifecycle
// ---------------------------------------------------------------------------

#[async_trait]
pub trait PortManager: Send + Sync {
    async fn start_port(&self, port_id: i64) -> Result<()>;
    async fn stop_port(&self, port_id: i64) -> Result<()>;
    async fn restart_port(&self, port_id: i64) -> Result<()>;
    async fn is_running(&self, port_id: i64) -> bool;
    async fn running_ports(&self) -> Vec<i64>;
    async fn start_all_enabled(&self) -> Result<()>;
    async fn stop_all(&self) -> Result<()>;
}
