use async_trait::async_trait;
use chrono::DateTime;
use std::collections::HashMap;
use tokio_rusqlite::Connection;

use crate::error::{AppError, Result};
use crate::models::{RequestLog, SystemLog};
use crate::traits::{LogPage, LogQuery, LogStore};

pub struct SqliteLogStore {
    conn: Connection,
}

impl SqliteLogStore {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

fn row_to_request_log(row: &rusqlite::Row<'_>) -> rusqlite::Result<RequestLog> {
    let request_headers: HashMap<String, String> = row
        .get::<_, String>(7)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let created_at = row
        .get::<_, String>(11)
        .ok()
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_default();

    Ok(RequestLog {
        id: row.get(0)?,
        mock_api_id: row.get(1)?,
        port: row.get::<_, i64>(2)? as u16,
        method: row.get(3)?,
        path: row.get(4)?,
        query_string: row.get(5)?,
        request_headers,
        request_body: row.get(6)?,
        response_status: row.get::<_, i64>(8)? as u16,
        response_body: row.get(9)?,
        duration_ms: row.get::<_, i64>(10)? as u64,
        created_at,
    })
}

fn row_to_system_log(row: &rusqlite::Row<'_>) -> rusqlite::Result<SystemLog> {
    let fields: Option<serde_json::Value> = row
        .get::<_, Option<String>>(4)?
        .and_then(|s| serde_json::from_str(&s).ok());

    let created_at = row
        .get::<_, String>(5)
        .ok()
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_default();

    Ok(SystemLog {
        id: row.get(0)?,
        level: row.get(1)?,
        target: row.get(2)?,
        message: row.get(3)?,
        fields,
        created_at,
    })
}

#[async_trait]
impl LogStore for SqliteLogStore {
    async fn append_request_log(&self, log: RequestLog) -> Result<i64> {
        let headers_json =
            serde_json::to_string(&log.request_headers).unwrap_or_else(|_| "{}".into());
        let port = log.port as i64;
        let status = log.response_status as i64;
        let dur = log.duration_ms as i64;

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO request_logs \
                     (mock_api_id, port, method, path, query_string, \
                      request_body, request_headers, response_status, response_body, duration_ms) \
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                    rusqlite::params![
                        log.mock_api_id,
                        port,
                        log.method,
                        log.path,
                        log.query_string,
                        log.request_body,
                        headers_json,
                        status,
                        log.response_body,
                        dur,
                    ],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .await
            .map_err(AppError::from)
    }

    async fn list_request_logs(&self, query: LogQuery) -> Result<LogPage<RequestLog>> {
        let page_size = if query.page_size == 0 { 50 } else { query.page_size };
        let offset = query.page * page_size;

        self.conn
            .call(move |conn| {
                let mut conditions: Vec<String> = Vec::new();
                let mut count_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
                let mut data_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

                if let Some(port) = query.port {
                    conditions.push(format!("port = ?{}", conditions.len() + 1));
                    count_params.push(Box::new(port as i64));
                    data_params.push(Box::new(port as i64));
                }
                if let Some(mid) = query.mock_api_id {
                    conditions.push(format!("mock_api_id = ?{}", conditions.len() + 1));
                    count_params.push(Box::new(mid));
                    data_params.push(Box::new(mid));
                }
                if let Some(since) = query.since {
                    conditions.push(format!("created_at >= ?{}", conditions.len() + 1));
                    let s = since.to_rfc3339();
                    count_params.push(Box::new(s.clone()));
                    data_params.push(Box::new(s));
                }
                if let Some(until) = query.until {
                    conditions.push(format!("created_at <= ?{}", conditions.len() + 1));
                    let s = until.to_rfc3339();
                    count_params.push(Box::new(s.clone()));
                    data_params.push(Box::new(s));
                }

                let where_clause = if conditions.is_empty() {
                    String::new()
                } else {
                    format!("WHERE {}", conditions.join(" AND "))
                };

                let count_sql = format!("SELECT COUNT(*) FROM request_logs {}", where_clause);
                let count_refs: Vec<&dyn rusqlite::ToSql> =
                    count_params.iter().map(|b| b.as_ref()).collect();
                let total: i64 =
                    conn.query_row(&count_sql, count_refs.as_slice(), |r| r.get(0))?;

                let n_data = data_params.len();
                let data_sql = format!(
                    "SELECT id, mock_api_id, port, method, path, query_string, \
                     request_body, request_headers, response_status, response_body, \
                     duration_ms, created_at \
                     FROM request_logs {} ORDER BY created_at DESC \
                     LIMIT ?{} OFFSET ?{}",
                    where_clause,
                    n_data + 1,
                    n_data + 2,
                );
                data_params.push(Box::new(page_size as i64));
                data_params.push(Box::new(offset as i64));
                let data_refs: Vec<&dyn rusqlite::ToSql> =
                    data_params.iter().map(|b| b.as_ref()).collect();

                let mut stmt = conn.prepare(&data_sql)?;
                let items = stmt
                    .query_map(data_refs.as_slice(), row_to_request_log)?
                    .collect::<rusqlite::Result<Vec<_>>>()?;

                Ok(LogPage {
                    items,
                    total: total as u64,
                    page: query.page,
                    page_size,
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn get_request_log(&self, id: i64) -> Result<Option<RequestLog>> {
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, mock_api_id, port, method, path, query_string, \
                     request_body, request_headers, response_status, response_body, \
                     duration_ms, created_at \
                     FROM request_logs WHERE id = ?1",
                )?;
                let mut rows = stmt.query_map(rusqlite::params![id], row_to_request_log)?;
                Ok(rows.next().transpose()?)
            })
            .await
            .map_err(AppError::from)
    }

    async fn clear_request_logs(&self) -> Result<()> {
        self.conn
            .call(|conn| {
                conn.execute("DELETE FROM request_logs", [])?;
                Ok(())
            })
            .await
            .map_err(AppError::from)
    }

    async fn append_system_log(&self, log: SystemLog) -> Result<i64> {
        let fields_json = log
            .fields
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok());

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO system_logs (level, target, message, fields) VALUES (?1,?2,?3,?4)",
                    rusqlite::params![log.level, log.target, log.message, fields_json],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .await
            .map_err(AppError::from)
    }

    async fn list_system_logs(&self, query: LogQuery) -> Result<LogPage<SystemLog>> {
        let page_size = if query.page_size == 0 { 50 } else { query.page_size };
        let offset = query.page * page_size;

        self.conn
            .call(move |conn| {
                let mut conditions: Vec<String> = Vec::new();
                let mut count_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
                let mut data_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

                if let Some(level) = query.level {
                    conditions.push(format!("level = ?{}", conditions.len() + 1));
                    count_params.push(Box::new(level.clone()));
                    data_params.push(Box::new(level));
                }
                if let Some(since) = query.since {
                    conditions.push(format!("created_at >= ?{}", conditions.len() + 1));
                    let s = since.to_rfc3339();
                    count_params.push(Box::new(s.clone()));
                    data_params.push(Box::new(s));
                }
                if let Some(until) = query.until {
                    conditions.push(format!("created_at <= ?{}", conditions.len() + 1));
                    let s = until.to_rfc3339();
                    count_params.push(Box::new(s.clone()));
                    data_params.push(Box::new(s));
                }

                let where_clause = if conditions.is_empty() {
                    String::new()
                } else {
                    format!("WHERE {}", conditions.join(" AND "))
                };

                let count_sql = format!("SELECT COUNT(*) FROM system_logs {}", where_clause);
                let count_refs: Vec<&dyn rusqlite::ToSql> =
                    count_params.iter().map(|b| b.as_ref()).collect();
                let total: i64 =
                    conn.query_row(&count_sql, count_refs.as_slice(), |r| r.get(0))?;

                let n_data = data_params.len();
                let data_sql = format!(
                    "SELECT id, level, target, message, fields, created_at \
                     FROM system_logs {} ORDER BY created_at DESC \
                     LIMIT ?{} OFFSET ?{}",
                    where_clause,
                    n_data + 1,
                    n_data + 2,
                );
                data_params.push(Box::new(page_size as i64));
                data_params.push(Box::new(offset as i64));
                let data_refs: Vec<&dyn rusqlite::ToSql> =
                    data_params.iter().map(|b| b.as_ref()).collect();

                let mut stmt = conn.prepare(&data_sql)?;
                let items = stmt
                    .query_map(data_refs.as_slice(), row_to_system_log)?
                    .collect::<rusqlite::Result<Vec<_>>>()?;

                Ok(LogPage {
                    items,
                    total: total as u64,
                    page: query.page,
                    page_size,
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn clear_system_logs(&self) -> Result<()> {
        self.conn
            .call(|conn| {
                conn.execute("DELETE FROM system_logs", [])?;
                Ok(())
            })
            .await
            .map_err(AppError::from)
    }
}
