# SQLite-Shared Port Runtime Status

**Date:** 2026-05-06
**Status:** Approved

## Problem

Port running state lives only in `LivePortManager::handles` — an in-process `HashMap`. When the
daemon (`mock start`) owns a port, a separately-launched TUI process has an empty map and shows
every port as "Stopped". Pressing Space in the TUI silently does nothing to daemon-owned ports.

**Goal:** TUI, dashboard, and daemon all read and write port running state through SQLite so that
stopping or starting a port in any mode is reflected everywhere and the daemon actually reacts.

---

## Approach

SQLite is used as the shared desired-state store. A reconciliation loop in the daemon process
polls the DB every 500 ms and starts or stops ports to match. The TUI detects whether a daemon is
alive and switches between direct mode (acts immediately) and control mode (writes to DB, daemon
reacts).

---

## Schema

Migration `0007_add_port_runtime_state` drops and recreates `mock_apis` and `port_configs` to add
two new columns. Existing data is lost on upgrade; this is acceptable at v0.1.0.

```sql
DROP TABLE IF EXISTS mock_apis;
DROP TABLE IF EXISTS port_configs;

CREATE TABLE port_configs (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    port       INTEGER NOT NULL UNIQUE,
    label      TEXT    NOT NULL DEFAULT '',
    enabled    INTEGER NOT NULL DEFAULT 1,
    running    INTEGER NOT NULL DEFAULT 0,
    owner_pid  INTEGER,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE mock_apis (
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
```

### Column semantics

| `running` | `owner_pid`      | Meaning                                      |
|-----------|------------------|----------------------------------------------|
| 0         | NULL             | Stopped, no owner                            |
| 1         | alive PID        | Running, owned by that process               |
| 1         | dead PID         | Stale — process crashed; cleanup resets this |
| 0         | alive PID        | Stop requested — reconciliation will stop it |

---

## Model

`src/models/mod.rs` — `PortConfig` gains two fields:

```rust
pub running: bool,
pub owner_pid: Option<u32>,
```

---

## `PortStore` Trait

Two new methods added to `src/traits/mod.rs`:

```rust
/// Set the runtime running state and owning PID for a port.
async fn set_port_running(&self, id: i64, running: bool, owner_pid: Option<u32>) -> Result<()>;

/// Clear running=0, owner_pid=NULL for any port whose owner_pid points to a dead process.
async fn cleanup_stale_running(&self) -> Result<()>;
```

`cleanup_stale_running` queries all rows where `owner_pid IS NOT NULL`, calls
`daemon::is_process_alive` on each PID, and resets dead ones.

---

## `LivePortManager`

`src/server/manager.rs`:

- **`start_port`**: after binding the `TcpListener`, calls
  `set_port_running(id, true, Some(std::process::id()))`.
- **`stop_port`**: after cancelling the `JoinHandle`, calls
  `set_port_running(id, false, None)`.
- **`start_all_enabled`**: calls `cleanup_stale_running()` first, then starts enabled ports as
  before.

---

## Reconciliation Loop (daemon / serve mode only)

Spawned as a background tokio task in `run_serve` (`src/main.rs`), after `start_all_enabled`.

```
every 500 ms:
  ports ← port_store.list_ports()
  my_pid ← std::process::id()

  for each port in ports:
    owned_by_other ← port.owner_pid is Some(pid) where pid ≠ my_pid AND is_process_alive(pid)
    if owned_by_other → skip

    in_handles ← port_manager.is_running(port.id)

    if port.running AND NOT in_handles:
      port_manager.start_port(port.id)   // desired=running, actual=stopped → start

    if NOT port.running AND in_handles:
      port_manager.stop_port(port.id)    // desired=stopped, actual=running → stop
```

The loop runs only in `serve`/daemon mode. TUI standalone mode does not need it because the TUI
acts on its own `LivePortManager` directly.

---

## TUI Changes

### Daemon detection

`daemon_alive: bool` is added to the `App` struct and re-evaluated on every `refresh_ports` call
(the daemon may start or stop while the TUI is open). It is derived by calling `daemon::read_pid`
and `daemon::is_process_alive`.

`daemon::read_pid` must be made `pub` (currently private).

### `refresh_ports`

Replace TCP probing entirely. Derive all state from DB and re-check daemon liveness each call:

```rust
async fn refresh_ports(app: &mut App, db_path: &str) {
    app.daemon_alive = daemon::read_pid(db_path)
        .map(|pid| daemon::is_process_alive(pid))
        .unwrap_or(false);

    if let Ok(ports) = app.state.port_store.list_ports().await {
        let my_pid = std::process::id();
        app.running_port_ids = ports.iter()
            .filter(|p| p.running)
            .map(|p| p.id)
            .collect();
        app.daemon_port_ids = ports.iter()
            .filter(|p| p.running && p.owner_pid.map(|pid| pid != my_pid).unwrap_or(false))
            .map(|p| p.id)
            .collect();
        app.ports = ports;
        app.port_selected = app.port_selected.min(app.ports.len().saturating_sub(1));
    }
}
```

`db_path` (the CLI `--db` value) must be threaded into the TUI run loop so `refresh_ports` can
locate the PID file.

### Space toggle

```
if port is running:
    if port is in daemon_port_ids:          // daemon owns it
        port_store.set_port_running(id, false, None)  // signal daemon to stop
    else:                                   // this process owns it
        port_manager.stop_port(id)          // stop immediately
    port_store.set_port_enabled(id, false)
else:                                       // port is stopped
    port_store.set_port_enabled(id, true)
    if app.daemon_alive:
        port_store.set_port_running(id, true, None)   // signal daemon to start
    else:
        port_manager.start_port(id)         // start immediately
```

### Port status display (`src/tui/views/ports.rs`)

| Condition | Label | Color |
|---|---|---|
| `daemon_port_ids` contains port | `● Daemon` | Yellow |
| `running_port_ids` contains port | `● Running` | Green |
| neither | `○ Stopped` | Gray |

---

## Dashboard

No changes required. `GET /api/v1/ports` returns `PortConfig` structs; the new `running` and
`owner_pid` fields are serialized automatically and visible to the frontend.

---

## Files Changed

| File | Change |
|---|---|
| `src/db/schema.rs` | Add migration `0007` |
| `src/models/mod.rs` | Add `running`, `owner_pid` to `PortConfig` |
| `src/traits/mod.rs` | Add `set_port_running`, `cleanup_stale_running` to `PortStore` |
| `src/db/port_store.rs` | Implement new methods; update row mapping |
| `src/server/manager.rs` | `start_port`/`stop_port` write to DB; `start_all_enabled` calls cleanup |
| `src/main.rs` | Spawn reconciliation loop in `run_serve` |
| `src/daemon.rs` | Make `read_pid` `pub` |
| `src/tui/mod.rs` | Daemon detection; updated `refresh_ports`; updated Space toggle |
| `src/tui/app.rs` | Add `daemon_alive: bool`; keep `daemon_port_ids`; `running_port_ids` sourced from DB |
| `src/tui/views/ports.rs` | Drive status label from `owner_pid` vs current PID |

---

## Out of Scope

- Frontend (Vue) changes to display `running` state — deferred
- Per-mock "serving" status — mock `enabled` in DB is sufficient
- Configurable poll interval
