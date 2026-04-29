pub mod subscriber;

use std::sync::Arc;

use tokio::sync::broadcast;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::models::LogEvent;
use crate::traits::LogStore;

pub fn init(log_store: Arc<dyn LogStore>, log_tx: broadcast::Sender<LogEvent>) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true);

    let db_layer = subscriber::DbLogLayer::new(log_store, log_tx);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(db_layer)
        .init();
}
