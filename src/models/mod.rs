use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    pub id: i64,
    pub port: u16,
    pub label: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    ANY,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HttpMethod::GET     => "GET",
            HttpMethod::POST    => "POST",
            HttpMethod::PUT     => "PUT",
            HttpMethod::PATCH   => "PATCH",
            HttpMethod::DELETE  => "DELETE",
            HttpMethod::HEAD    => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::ANY     => "ANY",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET"     => Ok(HttpMethod::GET),
            "POST"    => Ok(HttpMethod::POST),
            "PUT"     => Ok(HttpMethod::PUT),
            "PATCH"   => Ok(HttpMethod::PATCH),
            "DELETE"  => Ok(HttpMethod::DELETE),
            "HEAD"    => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "ANY"     => Ok(HttpMethod::ANY),
            other     => Err(format!("unknown method: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockApi {
    pub id: i64,
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
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    pub id: i64,
    pub mock_api_id: Option<i64>,
    pub port: u16,
    pub method: String,
    pub path: String,
    pub query_string: Option<String>,
    pub request_headers: HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_status: u16,
    pub response_headers: HashMap<String, String>,
    pub response_body: Option<String>,
    pub duration_ms: u64,
    pub client_ip: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLog {
    pub id: i64,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Unified log event broadcast over the internal channel and WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LogEvent {
    Request(RequestLog),
    System(SystemLog),
}
