use async_trait::async_trait;
use chrono::DateTime;
use tokio_rusqlite::Connection;

use crate::error::{AppError, Result};
use crate::models::PortConfig;
use crate::traits::PortStore;

pub struct SqlitePortStore {
    conn: Connection,
}

impl SqlitePortStore {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

fn row_to_port(row: &rusqlite::Row<'_>) -> rusqlite::Result<PortConfig> {
    let created_at_str: String = row.get(4)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_default();
    let owner_pid: Option<i64> = row.get(6)?;
    Ok(PortConfig {
        id: row.get(0)?,
        port: row.get::<_, i64>(1)? as u16,
        label: row.get(2)?,
        enabled: row.get::<_, i64>(3)? != 0,
        created_at,
        running: row.get::<_, i64>(5)? != 0,
        owner_pid: owner_pid.map(|p| p as u32),
    })
}

#[async_trait]
impl PortStore for SqlitePortStore {
    async fn list_ports(&self) -> Result<Vec<PortConfig>> {
        self.conn
            .call(|conn| {
                let mut stmt =
                    conn.prepare("SELECT id, port, label, enabled, created_at, running, owner_pid FROM port_configs ORDER BY id")?;
                let items = stmt
                    .query_map([], row_to_port)?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                Ok(items)
            })
            .await
            .map_err(AppError::from)
    }

    async fn get_port(&self, id: i64) -> Result<Option<PortConfig>> {
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, port, label, enabled, created_at, running, owner_pid FROM port_configs WHERE id = ?1",
                )?;
                let item = stmt
                    .query_map(rusqlite::params![id], row_to_port)?
                    .next()
                    .transpose()?;
                Ok(item)
            })
            .await
            .map_err(AppError::from)
    }

    async fn get_port_by_number(&self, port: u16) -> Result<Option<PortConfig>> {
        let port_i64 = port as i64;
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, port, label, enabled, created_at, running, owner_pid FROM port_configs WHERE port = ?1",
                )?;
                let item = stmt
                    .query_map(rusqlite::params![port_i64], row_to_port)?
                    .next()
                    .transpose()?;
                Ok(item)
            })
            .await
            .map_err(AppError::from)
    }

    async fn create_port(&self, port: u16, label: &str) -> Result<PortConfig> {
        let port_i64 = port as i64;
        let label = label.to_owned();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO port_configs (port, label) VALUES (?1, ?2)",
                    rusqlite::params![port_i64, label],
                )?;
                let id = conn.last_insert_rowid();
                let mut stmt = conn.prepare(
                    "SELECT id, port, label, enabled, created_at, running, owner_pid FROM port_configs WHERE id = ?1",
                )?;
                let item = stmt
                    .query_map(rusqlite::params![id], row_to_port)?
                    .next()
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)??;
                Ok(item)
            })
            .await
            .map_err(AppError::from)
    }

    async fn update_port(&self, id: i64, label: &str, enabled: bool) -> Result<PortConfig> {
        let label = label.to_owned();
        let enabled_i64 = enabled as i64;
        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE port_configs SET label = ?1, enabled = ?2 WHERE id = ?3",
                    rusqlite::params![label, enabled_i64, id],
                )?;
                let mut stmt = conn.prepare(
                    "SELECT id, port, label, enabled, created_at, running, owner_pid FROM port_configs WHERE id = ?1",
                )?;
                let item = stmt
                    .query_map(rusqlite::params![id], row_to_port)?
                    .next()
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)??;
                Ok(item)
            })
            .await
            .map_err(AppError::from)
    }

    async fn delete_port(&self, id: i64) -> Result<()> {
        self.conn
            .call(move |conn| {
                conn.execute("DELETE FROM port_configs WHERE id = ?1", rusqlite::params![id])?;
                Ok(())
            })
            .await
            .map_err(AppError::from)
    }

    async fn set_port_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        let enabled_i64 = enabled as i64;
        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE port_configs SET enabled = ?1 WHERE id = ?2",
                    rusqlite::params![enabled_i64, id],
                )?;
                Ok(())
            })
            .await
            .map_err(AppError::from)
    }

    async fn set_port_running(&self, id: i64, running: bool, owner_pid: Option<u32>) -> Result<()> {
        let running_i64 = running as i64;
        let owner_pid_i64: Option<i64> = owner_pid.map(|p| p as i64);
        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE port_configs SET running = ?1, owner_pid = ?2 WHERE id = ?3",
                    rusqlite::params![running_i64, owner_pid_i64, id],
                )?;
                Ok(())
            })
            .await
            .map_err(AppError::from)
    }
}
