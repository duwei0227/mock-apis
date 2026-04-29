use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

use crate::models::{LogEvent, SystemLog};
use crate::traits::LogStore;

/// A `tracing` Layer that writes every log event to SQLite and broadcasts it.
pub struct DbLogLayer {
    log_store: Arc<dyn LogStore>,
    log_tx: broadcast::Sender<LogEvent>,
}

impl DbLogLayer {
    pub fn new(log_store: Arc<dyn LogStore>, log_tx: broadcast::Sender<LogEvent>) -> Self {
        Self { log_store, log_tx }
    }
}

struct MessageVisitor {
    message: String,
    fields: serde_json::Map<String, serde_json::Value>,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value).trim_matches('"').to_owned();
        } else {
            self.fields.insert(
                field.name().to_owned(),
                serde_json::Value::String(format!("{:?}", value)),
            );
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_owned();
        } else {
            self.fields.insert(
                field.name().to_owned(),
                serde_json::Value::String(value.to_owned()),
            );
        }
    }
}

impl<S: Subscriber> Layer<S> for DbLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let level = event.metadata().level().to_string().to_uppercase();
        let target = event.metadata().target().to_owned();

        let mut visitor = MessageVisitor {
            message: String::new(),
            fields: serde_json::Map::new(),
        };
        event.record(&mut visitor);

        let fields = if visitor.fields.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(visitor.fields))
        };

        let log = SystemLog {
            id: 0,
            level,
            target,
            message: visitor.message,
            fields,
            created_at: chrono::Utc::now(),
        };

        let log_store = self.log_store.clone();
        let log_tx = self.log_tx.clone();
        let log_clone = log.clone();

        // Spawn a task so we don't block the tracing machinery.
        tokio::spawn(async move {
            if let Ok(id) = log_store.append_system_log(log_clone).await {
                let mut entry = log;
                entry.id = id;
                let _ = log_tx.send(LogEvent::System(entry));
            }
        });
    }
}
