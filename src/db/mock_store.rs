use async_trait::async_trait;
use chrono::DateTime;
use std::collections::HashMap;
use tokio_rusqlite::Connection;

use crate::error::{AppError, Result};
use crate::models::{HttpMethod, MockApi};
use crate::traits::{CreateMockRequest, MockStore, UpdateMockRequest};

pub struct SqliteMockStore {
    conn: Connection,
}

impl SqliteMockStore {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

fn row_to_mock(row: &rusqlite::Row<'_>) -> rusqlite::Result<MockApi> {
    let method_str: String = row.get(4)?;
    let method: HttpMethod = method_str.parse().unwrap_or(HttpMethod::ANY);

    let request_schema: Option<serde_json::Value> = row
        .get::<_, Option<String>>(6)?
        .and_then(|s| serde_json::from_str(&s).ok());

    let response_headers: HashMap<String, String> = row
        .get::<_, String>(8)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let created_at = row
        .get::<_, String>(11)
        .ok()
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_default();

    let updated_at = row
        .get::<_, String>(12)
        .ok()
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_default();

    Ok(MockApi {
        id: row.get(0)?,
        port_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        method,
        path: row.get(5)?,
        request_schema,
        response_status: row.get::<_, i64>(7)? as u16,
        response_headers,
        response_body: row.get(9)?,
        response_delay_ms: row.get::<_, i64>(10)? as u64,
        enabled: row.get::<_, i64>(13).map(|v| v != 0).unwrap_or(true),
        pagination_enabled: row.get::<_, i64>(14).map(|v| v != 0).unwrap_or(false),
        pagination_page_size: row.get::<_, i64>(15).unwrap_or(10) as u32,
        request_params: row.get::<_, String>(16).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
        pagination_page_param: row.get::<_, String>(17).unwrap_or_else(|_| "page".into()),
        pagination_size_param: row.get::<_, String>(18).unwrap_or_else(|_| "page_size".into()),
        pagination_data_field: row.get::<_, String>(19).unwrap_or_default(),
        pagination_total_field: row.get::<_, String>(20).unwrap_or_default(),
        created_at,
        updated_at,
    })
}

const SELECT_COLS: &str =
    "id, port_id, name, description, method, path, request_schema, \
     response_status, response_headers, response_body, response_delay_ms, \
     created_at, updated_at, enabled, \
     pagination_enabled, pagination_page_size, request_params, \
     pagination_page_param, pagination_size_param, pagination_data_field, pagination_total_field";

#[async_trait]
impl MockStore for SqliteMockStore {
    async fn list_mocks(&self, port_id: Option<i64>) -> Result<Vec<MockApi>> {
        self.conn
            .call(move |conn| {
                let sql = if port_id.is_some() {
                    format!("SELECT {} FROM mock_apis WHERE port_id = ?1 ORDER BY id", SELECT_COLS)
                } else {
                    format!("SELECT {} FROM mock_apis ORDER BY id", SELECT_COLS)
                };
                let mut stmt = conn.prepare(&sql)?;
                let items = if let Some(pid) = port_id {
                    stmt.query_map(rusqlite::params![pid], row_to_mock)?
                        .collect::<rusqlite::Result<Vec<_>>>()?
                } else {
                    stmt.query_map([], row_to_mock)?
                        .collect::<rusqlite::Result<Vec<_>>>()?
                };
                Ok(items)
            })
            .await
            .map_err(AppError::from)
    }

    async fn get_mock(&self, id: i64) -> Result<Option<MockApi>> {
        self.conn
            .call(move |conn| {
                let sql = format!("SELECT {} FROM mock_apis WHERE id = ?1", SELECT_COLS);
                let mut stmt = conn.prepare(&sql)?;
                let item = stmt
                    .query_map(rusqlite::params![id], row_to_mock)?
                    .next()
                    .transpose()?;
                Ok(item)
            })
            .await
            .map_err(AppError::from)
    }

    async fn create_mock(&self, req: CreateMockRequest) -> Result<MockApi> {
        let headers_json =
            serde_json::to_string(&req.response_headers).unwrap_or_else(|_| "{}".to_owned());
        let req_params_json =
            serde_json::to_string(&req.request_params).unwrap_or_else(|_| "{}".to_owned());
        let schema_json = req
            .request_schema
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok());
        let method_str = req.method.to_string();
        let delay = req.response_delay_ms as i64;
        let status = req.response_status as i64;
        let pag_enabled = req.pagination_enabled as i64;
        let pag_page_size = req.pagination_page_size as i64;

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO mock_apis \
                     (port_id, name, description, method, path, request_schema, \
                      response_status, response_headers, response_body, response_delay_ms, \
                      pagination_enabled, pagination_page_size, request_params, \
                      pagination_page_param, pagination_size_param, pagination_data_field, pagination_total_field) \
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
                    rusqlite::params![
                        req.port_id, req.name, req.description, method_str,
                        req.path, schema_json, status, headers_json,
                        req.response_body, delay,
                        pag_enabled, pag_page_size, req_params_json,
                        req.pagination_page_param, req.pagination_size_param,
                        req.pagination_data_field, req.pagination_total_field,
                    ],
                )?;
                let id = conn.last_insert_rowid();
                let sql = format!("SELECT {} FROM mock_apis WHERE id = ?1", SELECT_COLS);
                let mut stmt = conn.prepare(&sql)?;
                let item = stmt
                    .query_map(rusqlite::params![id], row_to_mock)?
                    .next()
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)??;
                Ok(item)
            })
            .await
            .map_err(AppError::from)
    }

    async fn update_mock(&self, id: i64, req: UpdateMockRequest) -> Result<MockApi> {
        self.conn
            .call(move |conn| {
                let mut sets: Vec<String> = Vec::new();
                let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

                macro_rules! push {
                    ($col:expr, $val:expr) => {{
                        sets.push(format!("{} = ?{}", $col, params.len() + 1));
                        params.push(Box::new($val));
                    }};
                }

                if let Some(v) = req.name           { push!("name", v); }
                if let Some(v) = req.description    { push!("description", v); }
                if let Some(v) = req.method         { push!("method", v.to_string()); }
                if let Some(v) = req.path           { push!("path", v); }
                if let Some(v) = req.response_status { push!("response_status", v as i64); }
                if let Some(v) = req.response_body  { push!("response_body", v); }
                if let Some(v) = req.response_delay_ms { push!("response_delay_ms", v as i64); }
                if let Some(v) = req.enabled        { push!("enabled", v as i64); }
                if let Some(v) = req.pagination_enabled      { push!("pagination_enabled", v as i64); }
                if let Some(v) = req.request_params {
                    let j = serde_json::to_string(&v).unwrap_or_else(|_| "{}".into());
                    push!("request_params", j);
                }
                if let Some(v) = req.pagination_page_size    { push!("pagination_page_size", v as i64); }
                if let Some(v) = req.pagination_page_param   { push!("pagination_page_param", v); }
                if let Some(v) = req.pagination_size_param   { push!("pagination_size_param", v); }
                if let Some(v) = req.pagination_data_field   { push!("pagination_data_field", v); }
                if let Some(v) = req.pagination_total_field  { push!("pagination_total_field", v); }
                if let Some(v) = req.response_headers {
                    let j = serde_json::to_string(&v).unwrap_or_else(|_| "{}".into());
                    push!("response_headers", j);
                }
                if let Some(v) = req.request_schema {
                    let j: Option<String> = v.as_ref().and_then(|val| serde_json::to_string(val).ok());
                    push!("request_schema", j);
                }

                if sets.is_empty() {
                    let sql = format!("SELECT {} FROM mock_apis WHERE id = ?1", SELECT_COLS);
                    let mut stmt = conn.prepare(&sql)?;
                    let item = stmt
                        .query_map(rusqlite::params![id], row_to_mock)?
                        .next()
                        .ok_or(rusqlite::Error::QueryReturnedNoRows)??;
                    return Ok(item);
                }

                sets.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')".to_owned());
                let idx = params.len() + 1;
                params.push(Box::new(id));

                let sql = format!("UPDATE mock_apis SET {} WHERE id = ?{}", sets.join(", "), idx);
                let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
                conn.execute(&sql, refs.as_slice())?;

                let sel_sql = format!("SELECT {} FROM mock_apis WHERE id = ?1", SELECT_COLS);
                let mut stmt = conn.prepare(&sel_sql)?;
                let item = stmt
                    .query_map(rusqlite::params![id], row_to_mock)?
                    .next()
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)??;
                Ok(item)
            })
            .await
            .map_err(AppError::from)
    }

    async fn delete_mock(&self, id: i64) -> Result<()> {
        self.conn
            .call(move |conn| {
                conn.execute("DELETE FROM mock_apis WHERE id = ?1", rusqlite::params![id])?;
                Ok(())
            })
            .await
            .map_err(AppError::from)
    }

    async fn set_mock_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        let v = enabled as i64;
        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE mock_apis SET enabled = ?1 WHERE id = ?2",
                    rusqlite::params![v, id],
                )?;
                Ok(())
            })
            .await
            .map_err(AppError::from)
    }

    async fn find_matching_mock(
        &self,
        port_id: i64,
        method: &HttpMethod,
        path: &str,
    ) -> Result<Option<MockApi>> {
        let method_str = method.to_string();
        let path = path.to_owned();
        self.conn
            .call(move |conn| {
                let sql = format!(
                    "SELECT {} FROM mock_apis \
                     WHERE port_id = ?1 AND enabled = 1 \
                       AND (method = ?2 OR method = 'ANY') \
                       AND ?3 LIKE (REPLACE(path, '*', '%')) \
                     ORDER BY CASE WHEN method = ?2 THEN 0 ELSE 1 END, id \
                     LIMIT 1",
                    SELECT_COLS
                );
                let mut stmt = conn.prepare(&sql)?;
                let item = stmt
                    .query_map(rusqlite::params![port_id, method_str, path], row_to_mock)?
                    .next()
                    .transpose()?;
                Ok(item)
            })
            .await
            .map_err(AppError::from)
    }
}
