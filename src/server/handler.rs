use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::Response;
use tokio::sync::broadcast;
use tokio::time::Duration;

use crate::models::{HttpMethod, LogEvent, MockApi, RequestLog};
use crate::traits::LogStore;

#[derive(Clone)]
pub struct MockHandlerState {
    pub port: u16,
    pub mocks: Arc<Vec<MockApi>>,
    pub log_store: Arc<dyn LogStore>,
    pub log_tx: broadcast::Sender<LogEvent>,
}

pub async fn mock_fallback(
    State(state): State<MockHandlerState>,
    req: Request,
) -> Response {
    let start = Instant::now();

    let method_str = req.method().as_str().to_uppercase();
    let method: HttpMethod = method_str.parse().unwrap_or(HttpMethod::GET);
    let path = req.uri().path().to_owned();
    let query_string = req.uri().query().map(|q| q.to_owned());

    // Collect request headers.
    let request_headers: HashMap<String, String> = req
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_owned(),
                v.to_str().unwrap_or("").to_owned(),
            )
        })
        .collect();

    // Read request body.
    let request_body = axum::body::to_bytes(req.into_body(), 1024 * 1024)
        .await
        .ok()
        .and_then(|b| String::from_utf8(b.to_vec()).ok())
        .filter(|s| !s.is_empty());

    // Find matching mock (exact method first, then ANY).
    let matched = find_mock(&state.mocks, &method, &path);

    let (response, mock_id) = if let Some(mock) = matched {
        if mock.response_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(mock.response_delay_ms)).await;
        }

        let mut builder =
            Response::builder().status(StatusCode::from_u16(mock.response_status).unwrap_or(StatusCode::OK));

        for (k, v) in &mock.response_headers {
            if let (Ok(name), Ok(value)) = (
                k.parse::<HeaderName>(),
                v.parse::<HeaderValue>(),
            ) {
                builder = builder.header(name, value);
            }
        }

        let body = mock.response_body.clone();
        let id = mock.id;
        (
            builder.body(Body::from(body)).unwrap_or_default(),
            Some(id),
        )
    } else {
        (
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(r#"{"error":"no mock matched"}"#))
                .unwrap_or_default(),
            None,
        )
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let resp_status = response.status().as_u16();

    // Persist request log asynchronously.
    let log_entry = RequestLog {
        id: 0,
        mock_api_id: mock_id,
        port: state.port,
        method: method_str.clone(),
        path: path.clone(),
        query_string,
        request_headers,
        request_body,
        response_status: resp_status,
        response_body: None, // body already consumed
        duration_ms,
        created_at: chrono::Utc::now(),
    };

    // Fire-and-forget: don't block the response path.
    let log_store = state.log_store.clone();
    let log_tx = state.log_tx.clone();
    let log_clone = log_entry.clone();
    tokio::spawn(async move {
        if let Ok(id) = log_store.append_request_log(log_clone).await {
            let mut entry = log_entry;
            entry.id = id;
            let _ = log_tx.send(LogEvent::Request(entry));
        }
    });

    response
}

/// Iterate mocks and return the best match: exact method beats ANY.
fn find_mock<'a>(mocks: &'a [MockApi], method: &HttpMethod, path: &str) -> Option<&'a MockApi> {
    let exact = mocks.iter().find(|m| {
        m.enabled && &m.method == method && path_matches(&m.path, path)
    });
    if exact.is_some() {
        return exact;
    }
    mocks.iter().find(|m| {
        m.enabled && m.method == HttpMethod::ANY && path_matches(&m.path, path)
    })
}

/// Simple glob-style path matching where `*` matches any segment.
fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern == path {
        return true;
    }
    if !pattern.contains('*') {
        return false;
    }
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();
    if pattern_parts.len() != path_parts.len() {
        // Allow trailing wildcard to match remaining segments.
        if pattern_parts.last() != Some(&"*") {
            return false;
        }
        if path_parts.len() < pattern_parts.len() - 1 {
            return false;
        }
        return pattern_parts[..pattern_parts.len() - 1]
            .iter()
            .zip(path_parts.iter())
            .all(|(p, s)| *p == "*" || p == s);
    }
    pattern_parts
        .iter()
        .zip(path_parts.iter())
        .all(|(p, s)| *p == "*" || p == s)
}
