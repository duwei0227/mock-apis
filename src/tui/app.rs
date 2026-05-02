use std::collections::HashMap;

use crate::models::{LogEvent, MockApi, PortConfig, RequestLog, SystemLog};
use crate::traits::{CreateMockRequest, UpdateMockRequest};
use crate::AppState;

pub const PORT_ID_FIELD_IDX: usize = 0;
pub const METHOD_FIELD_IDX: usize = 1;
pub const PATH_FIELD_IDX: usize = 2;
pub const HEADER_FIELD_IDX: usize = 7;
pub const BODY_SOURCE_FIELD_IDX: usize = 8;
pub const BODY_FIELD_IDX: usize = 9;
pub const HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE"];
pub const BODY_SOURCES: &[&str] = &["Inline", "File"];
pub const COMMON_HEADERS: &[&str] = &[
    "Content-Type: application/json",
    "Content-Type: text/plain",
    "Content-Type: text/html; charset=utf-8",
    "Content-Type: application/xml",
    "Cache-Control: no-cache",
    "Cache-Control: no-store, max-age=0",
    "Access-Control-Allow-Origin: *",
];

fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn encode_body_field(source: &str, body: &str) -> String {
    if source == "File" && !body.is_empty() {
        format!("file://{}", body)
    } else {
        body.to_owned()
    }
}

fn parse_headers(s: &str) -> HashMap<String, String> {
    s.split('|')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, ':');
            let key = parts.next()?.trim().to_owned();
            let val = parts.next()?.trim().to_owned();
            if key.is_empty() { None } else { Some((key, val)) }
        })
        .collect()
}

fn format_headers(h: &HashMap<String, String>) -> String {
    h.iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join(" | ")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Ports  = 0,
    Mocks  = 1,
    Logs   = 2,
}

impl Tab {
    pub fn next(self) -> Self {
        match self {
            Tab::Ports => Tab::Mocks,
            Tab::Mocks => Tab::Logs,
            Tab::Logs  => Tab::Ports,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogTab { Request, System }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKind { PortCreate, PortEdit, MockCreate, MockEdit, Confirm }

pub struct App {
    pub state: AppState,
    pub active_tab: Tab,

    // Ports tab
    pub ports: Vec<PortConfig>,
    pub running_port_ids: Vec<i64>,
    pub port_selected: usize,

    // Mocks tab
    pub mocks: Vec<MockApi>,
    pub mock_selected: usize,
    pub mock_port_filter: Option<i64>,

    // Logs tab
    pub log_tab: LogTab,
    pub request_logs: Vec<RequestLog>,
    pub system_logs: Vec<SystemLog>,
    pub log_follow: bool,
    pub log_selected: usize,
    pub log_detail_open: bool,

    // Modal
    pub modal: Option<ModalKind>,
    pub modal_fields: Vec<String>,
    pub modal_field_idx: usize,
    pub modal_cursor_pos: usize,
    pub cancel_confirm_pending: bool,
    pub confirm_message: String,
    pub confirm_action: Option<ConfirmAction>,

    pub status_msg: Option<String>,
    pub modal_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeletePort(i64),
    DeleteMock(i64),
}

impl App {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            active_tab: Tab::Ports,
            ports: Vec::new(),
            running_port_ids: Vec::new(),
            port_selected: 0,
            mocks: Vec::new(),
            mock_selected: 0,
            mock_port_filter: None,
            log_tab: LogTab::Request,
            request_logs: Vec::new(),
            system_logs: Vec::new(),
            log_follow: true,
            log_selected: 0,
            log_detail_open: false,
            modal: None,
            modal_fields: Vec::new(),
            modal_field_idx: 0,
            modal_cursor_pos: 0,
            cancel_confirm_pending: false,
            confirm_message: String::new(),
            confirm_action: None,
            status_msg: None,
            modal_error: None,
        }
    }

    pub fn port_list_nav_down(&mut self) {
        if !self.ports.is_empty() {
            self.port_selected = (self.port_selected + 1).min(self.ports.len() - 1);
        }
    }
    pub fn port_list_nav_up(&mut self) {
        self.port_selected = self.port_selected.saturating_sub(1);
    }
    pub fn selected_port(&self) -> Option<&PortConfig> {
        self.ports.get(self.port_selected)
    }

    pub fn mock_list_nav_down(&mut self) {
        if !self.mocks.is_empty() {
            self.mock_selected = (self.mock_selected + 1).min(self.mocks.len() - 1);
        }
    }
    pub fn mock_list_nav_up(&mut self) {
        self.mock_selected = self.mock_selected.saturating_sub(1);
    }
    pub fn selected_mock(&self) -> Option<&MockApi> {
        self.mocks.get(self.mock_selected)
    }

    pub fn log_nav_down(&mut self) {
        let len = match self.log_tab {
            LogTab::Request => self.request_logs.len(),
            LogTab::System  => self.system_logs.len(),
        };
        if len > 0 {
            self.log_selected = (self.log_selected + 1).min(len - 1);
        }
    }
    pub fn log_nav_up(&mut self) {
        self.log_selected = self.log_selected.saturating_sub(1);
    }

    pub fn push_log_event(&mut self, ev: LogEvent) {
        match ev {
            LogEvent::Request(r) => {
                self.request_logs.insert(0, r);
                if self.request_logs.len() > 500 { self.request_logs.truncate(500); }
                if self.log_follow { self.log_selected = 0; }
            }
            LogEvent::System(s) => {
                self.system_logs.insert(0, s);
                if self.system_logs.len() > 500 { self.system_logs.truncate(500); }
                if self.log_follow { self.log_selected = 0; }
            }
        }
    }

    pub fn cycle_port_field(&mut self, forward: bool) {
        if self.ports.is_empty() { return; }
        let current_id: i64 = self.modal_fields
            .get(PORT_ID_FIELD_IDX)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let pos = self.ports.iter().position(|p| p.id == current_id).unwrap_or(0);
        let next = if forward {
            (pos + 1) % self.ports.len()
        } else {
            (pos + self.ports.len() - 1) % self.ports.len()
        };
        if let Some(field) = self.modal_fields.get_mut(PORT_ID_FIELD_IDX) {
            *field = self.ports[next].id.to_string();
        }
    }

    pub fn cycle_method_field(&mut self, forward: bool) {
        if let Some(current) = self.modal_fields.get(METHOD_FIELD_IDX) {
            let pos = HTTP_METHODS.iter().position(|&m| m == current.as_str()).unwrap_or(0);
            let next = if forward {
                (pos + 1) % HTTP_METHODS.len()
            } else {
                (pos + HTTP_METHODS.len() - 1) % HTTP_METHODS.len()
            };
            if let Some(field) = self.modal_fields.get_mut(METHOD_FIELD_IDX) {
                *field = HTTP_METHODS[next].to_owned();
            }
        }
    }

    pub fn open_port_create(&mut self) {
        self.modal = Some(ModalKind::PortCreate);
        self.modal_fields = vec![String::new(), String::new()]; // [port, label]
        self.modal_field_idx = 0;
        self.modal_cursor_pos = 0;
        self.status_msg = None;
    }

    pub fn open_port_edit(&mut self) {
        let data = self.selected_port().map(|p| (p.port, p.label.clone()));
        if let Some((port, label)) = data {
            self.modal = Some(ModalKind::PortEdit);
            self.modal_fields = vec![port.to_string(), label];
            self.modal_field_idx = 0;
            self.modal_cursor_pos = self.modal_fields[0].chars().count();
        }
    }

    pub fn header_autocomplete_suggestion(&self) -> Option<&'static str> {
        let current = self.modal_fields.get(HEADER_FIELD_IDX)?;
        let prefix = current.rsplit(" | ").next().unwrap_or(current.as_str());
        if prefix.is_empty() { return None; }
        COMMON_HEADERS.iter().find(|&&h| {
            h.to_lowercase().starts_with(&prefix.to_lowercase())
        }).copied()
    }

    pub fn accept_header_autocomplete(&mut self) {
        if let Some(suggestion) = self.header_autocomplete_suggestion() {
            if let Some(field) = self.modal_fields.get_mut(HEADER_FIELD_IDX) {
                if let Some(pos) = field.rfind(" | ") {
                    field.truncate(pos + 3);
                    field.push_str(suggestion);
                } else {
                    *field = suggestion.to_owned();
                }
            }
        }
    }

    pub fn cycle_body_source_field(&mut self, forward: bool) {
        if let Some(current) = self.modal_fields.get(BODY_SOURCE_FIELD_IDX) {
            let pos = BODY_SOURCES.iter().position(|&s| s == current.as_str()).unwrap_or(0);
            let next = if forward {
                (pos + 1) % BODY_SOURCES.len()
            } else {
                (pos + BODY_SOURCES.len() - 1) % BODY_SOURCES.len()
            };
            if let Some(field) = self.modal_fields.get_mut(BODY_SOURCE_FIELD_IDX) {
                *field = BODY_SOURCES[next].to_owned();
            }
        }
    }

    pub fn open_mock_create(&mut self) {
        self.modal = Some(ModalKind::MockCreate);
        // fields: [port_id, method, path, name, desc, status, delay, headers, body_source, body]
        let port_id = self.ports.first().map(|p| p.id.to_string()).unwrap_or_default();
        self.modal_fields = vec![
            port_id,
            "GET".into(),
            "/".into(),
            String::new(),
            String::new(),
            "200".into(),
            "0".into(),
            String::new(),
            "Inline".into(),
            String::new(),
        ];
        self.modal_field_idx = 0;
        self.modal_cursor_pos = 0;
    }

    pub fn open_mock_edit(&mut self) {
        if let Some(m) = self.selected_mock().cloned() {
            self.modal = Some(ModalKind::MockEdit);
            let (body_source, body) = if m.response_body.starts_with("file://") {
                ("File".to_owned(), m.response_body["file://".len()..].to_owned())
            } else {
                ("Inline".to_owned(), m.response_body.clone())
            };
            self.modal_fields = vec![
                m.port_id.to_string(),
                m.method.to_string(),
                m.path.clone(),
                m.name.clone(),
                m.description.clone(),
                m.response_status.to_string(),
                m.response_delay_ms.to_string(),
                format_headers(&m.response_headers),
                body_source,
                body,
            ];
            self.modal_field_idx = 0;
            self.modal_cursor_pos = self.modal_fields[0].chars().count();
        }
    }

    pub fn modal_field_next(&mut self) {
        let len = self.modal_fields.len();
        if len > 0 {
            self.modal_field_idx = (self.modal_field_idx + 1) % len;
            self.modal_cursor_pos = self.modal_fields
                .get(self.modal_field_idx)
                .map(|f| f.chars().count())
                .unwrap_or(0);
        }
    }

    pub fn modal_field_prev(&mut self) {
        let len = self.modal_fields.len();
        if len > 0 {
            self.modal_field_idx = (self.modal_field_idx + len - 1) % len;
            self.modal_cursor_pos = self.modal_fields
                .get(self.modal_field_idx)
                .map(|f| f.chars().count())
                .unwrap_or(0);
        }
    }

    pub fn modal_type_char(&mut self, c: char) {
        if let Some(field) = self.modal_fields.get_mut(self.modal_field_idx) {
            let char_count = field.chars().count();
            let pos = self.modal_cursor_pos.min(char_count);
            let byte_pos = field.char_indices().nth(pos).map(|(i, _)| i).unwrap_or(field.len());
            field.insert(byte_pos, c);
            self.modal_cursor_pos = pos + 1;
        }
    }

    pub fn modal_backspace(&mut self) {
        if let Some(field) = self.modal_fields.get_mut(self.modal_field_idx) {
            if self.modal_cursor_pos > 0 {
                let pos = self.modal_cursor_pos - 1;
                let byte_pos = field.char_indices().nth(pos).map(|(i, _)| i).unwrap_or(0);
                field.remove(byte_pos);
                self.modal_cursor_pos = pos;
            }
        }
    }

    pub fn modal_paste(&mut self, text: &str) {
        let trimmed = text.trim();
        let cleaned = if trimmed.starts_with('{') || trimmed.starts_with('[') {
            // Try compact JSON serialization first; collapse whitespace as fallback.
            serde_json::from_str::<serde_json::Value>(trimmed)
                .ok()
                .and_then(|v| serde_json::to_string(&v).ok())
                .unwrap_or_else(|| collapse_whitespace(trimmed))
        } else {
            collapse_whitespace(trimmed)
        };
        if let Some(field) = self.modal_fields.get_mut(self.modal_field_idx) {
            let char_count = field.chars().count();
            let pos = self.modal_cursor_pos.min(char_count);
            let byte_pos = field.char_indices().nth(pos).map(|(i, _)| i).unwrap_or(field.len());
            field.insert_str(byte_pos, &cleaned);
            self.modal_cursor_pos = pos + cleaned.chars().count();
        }
    }

    pub fn modal_cursor_left(&mut self) {
        self.modal_cursor_pos = self.modal_cursor_pos.saturating_sub(1);
    }

    pub fn modal_cursor_right(&mut self) {
        if let Some(field) = self.modal_fields.get(self.modal_field_idx) {
            let max = field.chars().count();
            self.modal_cursor_pos = (self.modal_cursor_pos + 1).min(max);
        }
    }

    pub fn dismiss_modal(&mut self) {
        self.modal = None;
        self.modal_fields.clear();
        self.modal_field_idx = 0;
        self.modal_cursor_pos = 0;
        self.cancel_confirm_pending = false;
        self.modal_error = None;
    }

    pub fn validate_port_modal(&self) -> Option<String> {
        let port = self.modal_fields.get(0).map(|s| s.as_str()).unwrap_or("");
        if port.is_empty() {
            return Some("Port number is required".to_owned());
        }
        if port.parse::<u16>().map(|p| p == 0).unwrap_or(true) {
            return Some("Port number must be a valid number (1–65535)".to_owned());
        }
        None
    }

    pub fn validate_mock_modal(&self) -> Option<String> {
        let path = self.modal_fields.get(PATH_FIELD_IDX).map(|s| s.as_str()).unwrap_or("");
        if path.is_empty() {
            return Some("Path is required".to_owned());
        }
        let name = self.modal_fields.get(3).map(|s| s.as_str()).unwrap_or("");
        if name.is_empty() {
            return Some("Name is required".to_owned());
        }
        let body_source = self.modal_fields.get(BODY_SOURCE_FIELD_IDX).map(|s| s.as_str()).unwrap_or("Inline");
        let body = self.modal_fields.get(BODY_FIELD_IDX).map(|s| s.as_str()).unwrap_or("");
        if body_source == "File" && body.is_empty() {
            return Some("File path is required when Body Source is File".to_owned());
        }
        None
    }

    /// Build a CreateMockRequest from current modal fields.
    pub fn build_create_mock(&self) -> Option<CreateMockRequest> {
        let f = &self.modal_fields;
        if f.len() < 10 { return None; }
        use crate::models::HttpMethod;
        use std::str::FromStr;
        let response_body = encode_body_field(&f[8], &f[9]);
        Some(CreateMockRequest {
            port_id: f[0].parse().ok()?,
            method: HttpMethod::from_str(&f[1]).unwrap_or(HttpMethod::GET),
            path: f[2].clone(),
            name: f[3].clone(),
            description: f[4].clone(),
            request_schema: None,
            response_status: f[5].parse().unwrap_or(200),
            response_delay_ms: f[6].parse().unwrap_or(0),
            response_headers: parse_headers(&f[7]),
            response_body,
        })
    }

    pub fn build_update_mock(&self) -> UpdateMockRequest {
        let f = &self.modal_fields;
        use crate::models::HttpMethod;
        use std::str::FromStr;
        let response_body = if f.len() >= 10 {
            Some(encode_body_field(f.get(8).map(|s| s.as_str()).unwrap_or("Inline"),
                                   f.get(9).map(|s| s.as_str()).unwrap_or("")))
        } else {
            f.get(9).cloned()
        };
        UpdateMockRequest {
            method: f.get(1).and_then(|s| HttpMethod::from_str(s).ok()),
            path: f.get(2).cloned(),
            name: f.get(3).cloned(),
            description: f.get(4).cloned(),
            response_status: f.get(5).and_then(|s| s.parse().ok()),
            response_delay_ms: f.get(6).and_then(|s| s.parse().ok()),
            response_headers: f.get(7).map(|s| parse_headers(s)),
            response_body,
            ..Default::default()
        }
    }
}
