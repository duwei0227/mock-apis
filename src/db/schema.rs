use rusqlite::Connection;
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_schema_migrations", "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version    TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        );
    "),
    ("0002_port_configs", "
        CREATE TABLE IF NOT EXISTS port_configs (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            port       INTEGER NOT NULL UNIQUE,
            label      TEXT    NOT NULL DEFAULT '',
            enabled    INTEGER NOT NULL DEFAULT 1,
            created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        );
    "),
    ("0003_mock_apis", "
        CREATE TABLE IF NOT EXISTS mock_apis (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            port_id           INTEGER NOT NULL REFERENCES port_configs(id) ON DELETE CASCADE,
            name              TEXT    NOT NULL,
            description       TEXT    NOT NULL DEFAULT '',
            method            TEXT    NOT NULL DEFAULT 'ANY',
            path              TEXT    NOT NULL,
            request_schema    TEXT,
            response_status   INTEGER NOT NULL DEFAULT 200,
            response_headers  TEXT    NOT NULL DEFAULT '{}',
            response_body     TEXT    NOT NULL DEFAULT '',
            response_delay_ms INTEGER NOT NULL DEFAULT 0,
            enabled           INTEGER NOT NULL DEFAULT 1,
            created_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            updated_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            UNIQUE(port_id, method, path)
        );
    "),
    ("0004_request_logs", "
        CREATE TABLE IF NOT EXISTS request_logs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            mock_api_id     INTEGER REFERENCES mock_apis(id) ON DELETE SET NULL,
            port            INTEGER NOT NULL,
            method          TEXT    NOT NULL,
            path            TEXT    NOT NULL,
            query_string    TEXT,
            request_headers TEXT    NOT NULL DEFAULT '{}',
            request_body    TEXT,
            response_status INTEGER NOT NULL,
            response_body   TEXT,
            duration_ms     INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_request_logs_mock_api_id ON request_logs(mock_api_id);
        CREATE INDEX IF NOT EXISTS idx_request_logs_created_at  ON request_logs(created_at DESC);
    "),
    ("0005_system_logs", "
        CREATE TABLE IF NOT EXISTS system_logs (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            level      TEXT NOT NULL,
            target     TEXT NOT NULL DEFAULT '',
            message    TEXT NOT NULL,
            fields     TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_system_logs_level      ON system_logs(level);
        CREATE INDEX IF NOT EXISTS idx_system_logs_created_at ON system_logs(created_at DESC);
    "),
    ("0006_request_logs_ip_resp_headers", "
        ALTER TABLE request_logs ADD COLUMN client_ip TEXT;
        ALTER TABLE request_logs ADD COLUMN response_headers TEXT NOT NULL DEFAULT '{}';
    "),
    ("0007_port_runtime_status", "
        ALTER TABLE port_configs ADD COLUMN running   INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE port_configs ADD COLUMN owner_pid INTEGER;
    "),
];

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    // Ensure the migrations table itself exists first.
    conn.execute_batch(MIGRATIONS[0].1)?;

    for (version, sql) in &MIGRATIONS[1..] {
        let already_applied: bool = conn.query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
            rusqlite::params![version],
            |row| row.get::<_, i64>(0),
        )? > 0;

        if !already_applied {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                rusqlite::params![version],
            )?;
        }
    }
    Ok(())
}
