use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::Response;
use tokio::sync::broadcast;
use tokio::time::Duration;

use crate::models::{HttpMethod, LogEvent, MockApi, RequestLog};
use crate::server::template;
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
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request,
) -> Response {
    let start = Instant::now();

    let method_str = req.method().as_str().to_uppercase();
    let method: HttpMethod = method_str.parse().unwrap_or(HttpMethod::GET);
    let path = req.uri().path().to_owned();
    let query_string = req.uri().query().map(|q| q.to_owned());

    // Resolve client IP: honour proxy headers before falling back to socket peer.
    let request_headers: HashMap<String, String> = req
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_owned(), v.to_str().unwrap_or("").to_owned()))
        .collect();

    let client_ip = request_headers
        .get("x-forwarded-for")
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_owned())
        .or_else(|| request_headers.get("x-real-ip").cloned())
        .unwrap_or_else(|| peer.ip().to_string());

    // Read request body.
    let request_body = axum::body::to_bytes(req.into_body(), 1024 * 1024)
        .await
        .ok()
        .and_then(|b| String::from_utf8(b.to_vec()).ok())
        .filter(|s| !s.is_empty());

    // Find matching mock (exact method first, then ANY).
    let matched = find_mock(&state.mocks, &method, &path);

    let (response, mock_id, resp_body_str, resp_headers_map) = if let Some(mock) = matched {
        if mock.response_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(mock.response_delay_ms)).await;
        }

        let raw_body = if let Some(file_path) = mock.response_body.strip_prefix("file://") {
            match tokio::fs::read_to_string(file_path).await {
                Ok(contents) => contents,
                Err(e) => format!("{{\"error\":\"file read failed: {}\"}}", e),
            }
        } else {
            mock.response_body.clone()
        };
        let rendered = template::render(&raw_body);
        let body = apply_filter_and_pagination(&rendered, mock, &method, query_string.as_deref(), request_body.as_deref());

        let mut builder =
            Response::builder().status(StatusCode::from_u16(mock.response_status).unwrap_or(StatusCode::OK));

        for (k, v) in &mock.response_headers {
            if let (Ok(name), Ok(value)) = (k.parse::<HeaderName>(), v.parse::<HeaderValue>()) {
                builder = builder.header(name, value);
            }
        }

        let id = mock.id;
        let resp_headers = mock.response_headers.clone();
        (
            builder.body(Body::from(body.clone())).unwrap_or_default(),
            Some(id),
            Some(body),
            resp_headers,
        )
    } else {
        (
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(r#"{"error":"no mock matched"}"#))
                .unwrap_or_default(),
            None,
            Some(r#"{"error":"no mock matched"}"#.to_owned()),
            HashMap::new(),
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
        response_headers: resp_headers_map,
        response_body: resp_body_str,
        duration_ms,
        client_ip: Some(client_ip),
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

fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            result.push(' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i+1..i+3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    result.push(byte as char);
                    i += 3;
                    continue;
                }
            }
            result.push(bytes[i] as char);
            i += 1;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

fn parse_query_params(query_string: Option<&str>) -> HashMap<String, String> {
    query_string
        .unwrap_or("")
        .split('&')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let k = percent_decode(parts.next()?);
            let v = percent_decode(parts.next().unwrap_or(""));
            Some((k, v))
        })
        .collect()
}

/// Extracts top-level scalar fields from a JSON object body as a string map.
fn extract_body_params(body: Option<&str>) -> HashMap<String, String> {
    let Some(b) = body else { return HashMap::new() };
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(b) else {
        return HashMap::new();
    };
    map.into_iter()
        .filter_map(|(k, v)| {
            let s = match &v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => return None,
            };
            Some((k, s))
        })
        .collect()
}

fn apply_filter_and_pagination(
    body: &str,
    mock: &MockApi,
    method: &HttpMethod,
    query_string: Option<&str>,
    req_body: Option<&str>,
) -> String {
    let needs_filter = !mock.request_params.is_empty();
    let needs_pagination = mock.pagination_enabled;
    if !needs_filter && !needs_pagination {
        return body.to_owned();
    }

    let json = match serde_json::from_str::<serde_json::Value>(body) {
        Ok(v) => v,
        Err(_) => return body.to_owned(),
    };

    let query_params = parse_query_params(query_string);
    let body_params = if *method == HttpMethod::POST {
        extract_body_params(req_body)
    } else {
        HashMap::new()
    };

    // Query params take priority over body params; only keys defined in mock.request_params are used.
    let filters: HashMap<String, String> = mock
        .request_params
        .keys()
        .filter_map(|k| {
            query_params
                .get(k)
                .or_else(|| body_params.get(k))
                .map(|v| (k.clone(), v.clone()))
        })
        .collect();

    tracing::debug!(
        mock_id = mock.id,
        mock_request_params = ?mock.request_params,
        query_params = ?query_params,
        body_params = ?body_params,
        active_filters = ?filters,
        "apply_filter: resolved filters"
    );

    let filtered = if filters.is_empty() {
        tracing::debug!(mock_id = mock.id, "apply_filter: no active filters, returning body as-is");
        json
    } else {
        tracing::debug!(mock_id = mock.id, active_filters = ?filters, "apply_filter: applying filters");
        apply_filter_recursive(json, &filters)
    };

    if !needs_pagination {
        return serde_json::to_string(&filtered).unwrap_or_else(|_| body.to_owned());
    }

    let data_field  = mock.pagination_data_field.trim();
    let total_field = mock.pagination_total_field.trim();
    let page_param  = mock.pagination_page_param.as_str();
    let size_param  = mock.pagination_size_param.as_str();
    let page: usize = query_params.get(page_param).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).max(1);
    let page_size: usize = query_params
        .get(size_param)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(mock.pagination_page_size as usize)
        .max(1);

    if data_field.is_empty() {
        // No data_field configured: paginate a top-level array and wrap in a new envelope.
        let arr = match filtered {
            serde_json::Value::Array(a) => a,
            other => return serde_json::to_string(&other).unwrap_or_else(|_| body.to_owned()),
        };
        let total = arr.len();
        let start = (page - 1) * page_size;
        let items: Vec<serde_json::Value> = arr.into_iter().skip(start).take(page_size).collect();
        serde_json::to_string(&serde_json::json!({
            "items": items,
            "total": total,
            "page": page,
            "page_size": page_size,
        }))
        .unwrap_or_else(|_| body.to_owned())
    } else {
        // data_field configured: find the array, paginate it, and write results back in-place.
        let arr = match find_array_by_field(&filtered, data_field) {
            Some(a) => a,
            None => return serde_json::to_string(&filtered).unwrap_or_else(|_| body.to_owned()),
        };
        let total = arr.len();
        let start = (page - 1) * page_size;
        let items: Vec<serde_json::Value> = arr.into_iter().skip(start).take(page_size).collect();

        // Replace data array in the original structure.
        let result = set_json_field(filtered, data_field, &serde_json::Value::Array(items));

        // Write computed total into the total field if configured.
        let result = if total_field.is_empty() {
            result
        } else {
            set_json_field(result, total_field, &serde_json::json!(total))
        };

        serde_json::to_string(&result).unwrap_or_else(|_| body.to_owned())
    }
}

/// Replace the field at `path` with `new_value`.
/// Dot-notation paths (e.g. `body.list`) traverse explicitly; plain names do DFS.
fn set_json_field(json: serde_json::Value, path: &str, new_value: &serde_json::Value) -> serde_json::Value {
    let (result, _) = set_json_field_inner(json, path, new_value);
    result
}

fn set_json_field_inner(
    json: serde_json::Value,
    path: &str,
    new_value: &serde_json::Value,
) -> (serde_json::Value, bool) {
    match json {
        serde_json::Value::Object(mut map) => {
            if let Some((head, rest)) = path.split_once('.') {
                // Explicit path: traverse head, recurse with rest.
                if let Some(child) = map.remove(head) {
                    let (new_child, found) = set_json_field_inner(child, rest, new_value);
                    map.insert(head.to_owned(), new_child);
                    return (serde_json::Value::Object(map), found);
                }
                return (serde_json::Value::Object(map), false);
            }
            // Plain name: exact match first, then DFS.
            if map.contains_key(path) {
                map.insert(path.to_owned(), new_value.clone());
                return (serde_json::Value::Object(map), true);
            }
            let mut found = false;
            let new_map = map
                .into_iter()
                .map(|(k, v)| {
                    if found {
                        (k, v)
                    } else {
                        let (new_v, f) = set_json_field_inner(v, path, new_value);
                        if f { found = true; }
                        (k, new_v)
                    }
                })
                .collect();
            (serde_json::Value::Object(new_map), found)
        }
        other => (other, false),
    }
}

/// Find the array at `path`.
/// Dot-notation paths (e.g. `body.list`) traverse explicitly; plain names do DFS.
fn find_array_by_field(json: &serde_json::Value, path: &str) -> Option<Vec<serde_json::Value>> {
    if let Some((head, rest)) = path.split_once('.') {
        // Explicit path: step into head, recurse with rest.
        return match json {
            serde_json::Value::Object(map) => {
                map.get(head).and_then(|v| find_array_by_field(v, rest))
            }
            _ => None,
        };
    }
    // Plain name: DFS search.
    match json {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(arr)) = map.get(path) {
                return Some(arr.clone());
            }
            for v in map.values() {
                if let Some(found) = find_array_by_field(v, path) {
                    return Some(found);
                }
            }
            None
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(found) = find_array_by_field(item, path) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

/// Recursively walk the JSON tree and filter arrays of objects by `filters`.
///
/// Rules:
/// - **Array whose items are objects**: filter out items that do not match all filters.
///   Non-object items within a mixed array are kept as-is.
/// - **Array of primitives** (strings, numbers, booleans): returned unchanged —
///   these are config/enum lists and should never be filtered.
/// - **Object**: recurse into every field value.
/// - **Scalar**: pass through unchanged.
fn apply_filter_recursive(
    json: serde_json::Value,
    filters: &HashMap<String, String>,
) -> serde_json::Value {
    match json {
        serde_json::Value::Array(arr) => {
            // Only filter if the array contains at least one object item.
            // Pure primitive arrays (e.g. ["GET","POST"] or [200,404]) pass through.
            if arr.iter().any(|v| matches!(v, serde_json::Value::Object(_))) {
                let result: Vec<serde_json::Value> = arr
                    .into_iter()
                    .filter(|item| match item {
                        serde_json::Value::Object(_) => item_matches_filters(item, filters),
                        _ => true, // keep non-object items in a mixed array
                    })
                    .collect();
                serde_json::Value::Array(result)
            } else {
                serde_json::Value::Array(arr)
            }
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, apply_filter_recursive(v, filters)))
                .collect(),
        ),
        other => other,
    }
}

/// Returns true when every filter `(key, expected)` is found at any depth within `item`.
fn item_matches_filters(item: &serde_json::Value, filters: &HashMap<String, String>) -> bool {
    filters.iter().all(|(key, expected)| json_has_field(item, key, expected))
}

fn json_has_field(json: &serde_json::Value, key: &str, expected: &str) -> bool {
    match json {
        serde_json::Value::Object(map) => {
            if let Some(val) = map.get(key) {
                if json_value_eq(val, expected) {
                    return true;
                }
            }
            map.values().any(|v| json_has_field(v, key, expected))
        }
        serde_json::Value::Array(arr) => arr.iter().any(|v| json_has_field(v, key, expected)),
        _ => false,
    }
}

fn json_value_eq(val: &serde_json::Value, expected: &str) -> bool {
    match val {
        serde_json::Value::String(s) => s == expected,
        serde_json::Value::Number(n) => n.to_string() == expected,
        serde_json::Value::Bool(b) => b.to_string() == expected,
        serde_json::Value::Null => expected == "null",
        _ => false,
    }
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

/// Path matching: `*` matches any single segment (trailing `*` matches remaining),
/// `{param}` matches any single segment (named parameter style).
fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern == path {
        return true;
    }
    let is_wildcard = |seg: &str| seg == "*" || (seg.starts_with('{') && seg.ends_with('}'));
    if !pattern.contains('*') && !pattern.contains('{') {
        return false;
    }
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();
    if pattern_parts.len() != path_parts.len() {
        // Allow trailing `*` to match remaining segments.
        if pattern_parts.last() != Some(&"*") {
            return false;
        }
        if path_parts.len() < pattern_parts.len() - 1 {
            return false;
        }
        return pattern_parts[..pattern_parts.len() - 1]
            .iter()
            .zip(path_parts.iter())
            .all(|(p, s)| is_wildcard(p) || p == s);
    }
    pattern_parts
        .iter()
        .zip(path_parts.iter())
        .all(|(p, s)| is_wildcard(p) || p == s)
}
