use crate::models::{LogEvent, MockApi, PortConfig, RequestLog, SystemLog};
use crate::traits::{CreateMockRequest, UpdateMockRequest};
use crate::AppState;

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

    // Modal
    pub modal: Option<ModalKind>,
    pub modal_fields: Vec<String>,
    pub modal_field_idx: usize,
    pub confirm_message: String,
    pub confirm_action: Option<ConfirmAction>,

    pub status_msg: Option<String>,
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
            modal: None,
            modal_fields: Vec::new(),
            modal_field_idx: 0,
            confirm_message: String::new(),
            confirm_action: None,
            status_msg: None,
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

    pub fn open_port_create(&mut self) {
        self.modal = Some(ModalKind::PortCreate);
        self.modal_fields = vec![String::new(), String::new()]; // [port, label]
        self.modal_field_idx = 0;
        self.status_msg = None;
    }

    pub fn open_port_edit(&mut self) {
        let data = self.selected_port().map(|p| (p.port, p.label.clone()));
        if let Some((port, label)) = data {
            self.modal = Some(ModalKind::PortEdit);
            self.modal_fields = vec![port.to_string(), label];
            self.modal_field_idx = 0;
        }
    }

    pub fn open_mock_create(&mut self) {
        self.modal = Some(ModalKind::MockCreate);
        // fields: [port_id, method, path, name, desc, status, delay, body]
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
        ];
        self.modal_field_idx = 0;
    }

    pub fn open_mock_edit(&mut self) {
        if let Some(m) = self.selected_mock().cloned() {
            self.modal = Some(ModalKind::MockEdit);
            self.modal_fields = vec![
                m.port_id.to_string(),
                m.method.to_string(),
                m.path.clone(),
                m.name.clone(),
                m.description.clone(),
                m.response_status.to_string(),
                m.response_delay_ms.to_string(),
                m.response_body.clone(),
            ];
            self.modal_field_idx = 0;
        }
    }

    pub fn modal_field_next(&mut self) {
        let len = self.modal_fields.len();
        if len > 0 {
            self.modal_field_idx = (self.modal_field_idx + 1) % len;
        }
    }

    pub fn modal_field_prev(&mut self) {
        let len = self.modal_fields.len();
        if len > 0 {
            self.modal_field_idx = (self.modal_field_idx + len - 1) % len;
        }
    }

    pub fn modal_type_char(&mut self, c: char) {
        if let Some(field) = self.modal_fields.get_mut(self.modal_field_idx) {
            field.push(c);
        }
    }

    pub fn modal_backspace(&mut self) {
        if let Some(field) = self.modal_fields.get_mut(self.modal_field_idx) {
            field.pop();
        }
    }

    pub fn dismiss_modal(&mut self) {
        self.modal = None;
        self.modal_fields.clear();
        self.modal_field_idx = 0;
    }

    /// Build a CreateMockRequest from current modal fields.
    pub fn build_create_mock(&self) -> Option<CreateMockRequest> {
        let f = &self.modal_fields;
        if f.len() < 8 { return None; }
        use crate::models::HttpMethod;
        use std::str::FromStr;
        Some(CreateMockRequest {
            port_id: f[0].parse().ok()?,
            method: HttpMethod::from_str(&f[1]).unwrap_or(HttpMethod::GET),
            path: f[2].clone(),
            name: f[3].clone(),
            description: f[4].clone(),
            request_schema: None,
            response_status: f[5].parse().unwrap_or(200),
            response_headers: Default::default(),
            response_body: f[7].clone(),
            response_delay_ms: f[6].parse().unwrap_or(0),
        })
    }

    pub fn build_update_mock(&self) -> UpdateMockRequest {
        let f = &self.modal_fields;
        use crate::models::HttpMethod;
        use std::str::FromStr;
        UpdateMockRequest {
            method: f.get(1).and_then(|s| HttpMethod::from_str(s).ok()),
            path: f.get(2).cloned(),
            name: f.get(3).cloned(),
            description: f.get(4).cloned(),
            response_status: f.get(5).and_then(|s| s.parse().ok()),
            response_delay_ms: f.get(6).and_then(|s| s.parse().ok()),
            response_body: f.get(7).cloned(),
            ..Default::default()
        }
    }
}
