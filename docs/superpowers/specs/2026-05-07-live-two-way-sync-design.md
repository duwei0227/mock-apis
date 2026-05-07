# Live Two-Way Sync: TUI + Dashboard + Daemon

**Date:** 2026-05-07
**Status:** Approved

## Goal

When `mock start` launches a background daemon, both the TUI (`mock`) and the web dashboard (`mock --dashboard`) can be open simultaneously and reflect each other's changes in near-real-time. Changes made in either UI are immediately visible in the other.

## Problems Being Solved

1. **Startup conflict**: when the TUI launches alongside a running daemon, it calls `start_all_enabled()`, which races with the daemon for port ownership and can cause duplicate port binds.
2. **Dashboard is not live**: the Vue frontend only fetches data on user action — no auto-refresh. Changes made via the TUI are invisible in the dashboard until the user manually refreshes.

## Architecture

```
mock start  →  spawns "mock serve" daemon (port manager + dashboard on :9999)
                         │
                    SQLite DB (shared source of truth)
                    broadcast::Sender<LogEvent> (internal event bus)
                         │
          ┌──────────────┴────────────────┐
   mock (TUI)                    mock --dashboard (browser)
   reads SQLite directly         reads via daemon HTTP API
   start/stop → HTTP to daemon   start/stop → HTTP to daemon
   config → SQLite directly      config → HTTP to daemon
   polls SQLite every 500ms      live events via existing WebSocket
```

**Source of truth:** SQLite.
**Port execution owner:** the daemon (single owner via `owner_pid`).
**Config changes** (create/update/delete): written to SQLite directly by either UI; daemon reconciliation loop (1 s) picks them up.
**Start/stop operations**: delegated to daemon via HTTP (already implemented for TUI; native for dashboard).
**Live updates to dashboard**: the existing `/ws/logs` WebSocket is extended to carry `state_changed` signals; the frontend re-fetches the affected resource on receipt.

## Changes

### 1. Startup Conflict Fix

**`src/daemon.rs`** — add:
```rust
pub fn is_external_daemon_running(db: &str) -> bool {
    let my_pid = std::process::id();
    match read_pid(db) {
        Some(pid) if pid != my_pid => is_process_alive(pid),
        _ => false,
    }
}
```
The `my_pid != pid` guard ensures the daemon process (`mock serve`) does not treat its own PID file as evidence of an external daemon.

**`src/main.rs`** — gate `start_all_enabled()`:
```rust
if !daemon::is_external_daemon_running(&cli.db) {
    port_manager.start_all_enabled().await?;
}
```

Behavior by mode:
| Mode | Daemon running? | `start_all_enabled` called? |
|---|---|---|
| `mock serve` (daemon itself) | — | Yes (own PID ≠ external) |
| `mock` (TUI) | No | Yes (standalone mode) |
| `mock` (TUI) | Yes | No (daemon owns ports) |
| `mock --dashboard` | No | Yes (standalone mode) |
| `mock --dashboard` | Yes | No (daemon owns ports) |

### 2. State Change Events

**`src/models/mod.rs`** — extend `LogEvent`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateResource {
    Ports,
    Mocks,
}

// LogEvent gains a new variant:
StateChanged { resource: StateResource }
```
Serializes as `{"type":"state_changed","resource":"ports"}`.

**Broadcast points** — each fires `log_tx.send(LogEvent::StateChanged { resource: ... })`:

| File | Operations | Resource |
|---|---|---|
| `src/server/manager.rs` | `start_port`, `stop_port`, `restart_port` | `Ports` |
| `src/dashboard/routes/ports.rs` | `create_port`, `update_port`, `delete_port`, `start_port`, `stop_port` | `Ports` |
| `src/dashboard/routes/mocks.rs` | `create_mock`, `update_mock`, `delete_mock`, `set_mock_enabled` | `Mocks` |

**`src/dashboard/ws.rs`** — no changes needed. It already serializes all `LogEvent` variants via `serde_json::to_string(&event)` and forwards to connected WebSocket clients.

### 3. Frontend Live Sync

**`frontend/src/App.vue`** — connect the WebSocket on app mount so it is always active (not only when the Logs view is open):
```typescript
import { onMounted, onUnmounted } from 'vue'
import { useLogsStore } from './stores/logs'

const logsStore = useLogsStore()
onMounted(() => logsStore.connectLive())
onUnmounted(() => logsStore.disconnectLive())
```

**`frontend/src/stores/logs.ts`** — handle `state_changed` in the existing `onmessage` handler:
```typescript
import { usePortsStore } from './ports'
import { useMocksStore } from './mocks'

// Inside onmessage, before existing type checks:
if (payload.type === 'state_changed') {
    if (payload.resource === 'ports') usePortsStore().fetchPorts()
    else if (payload.resource === 'mocks') useMocksStore().fetchMocks()
    return
}
```

Both `fetchPorts()` and `fetchMocks()` already exist. No other store changes needed.

## End-to-End Flows

**TUI → Dashboard:**
```
user presses Space on a port in TUI
  → daemon_post() HTTP POST /api/v1/ports/{id}/start
  → daemon start_port() runs, updates SQLite
  → broadcasts StateChanged{Ports} on log_tx
  → ws.rs forwards JSON to all WebSocket clients
  → frontend onmessage → usePortsStore().fetchPorts()
  → dashboard port list updates (< 1 s)
```

**Dashboard → TUI:**
```
user clicks Start in dashboard
  → POST /api/v1/ports/{id}/start
  → daemon start_port() runs, updates SQLite (running=true, owner_pid=daemon)
  → broadcasts StateChanged{Ports}
  → TUI polls SQLite every 500 ms → port shows Running
```

**TUI creates a new port:**
```
user presses n, fills form, Enter
  → port_store.create_port() writes to SQLite directly
  → daemon reconciliation loop (1 s) sees new enabled port with owner_pid=None
  → daemon starts the port, updates SQLite
  → broadcasts StateChanged{Ports}
  → dashboard frontend re-fetches port list
```

## Files Changed

| File | Change |
|---|---|
| `src/daemon.rs` | Add `is_external_daemon_running()` |
| `src/main.rs` | Gate `start_all_enabled()` behind external daemon check |
| `src/models/mod.rs` | Add `StateResource` enum and `StateChanged` variant to `LogEvent` |
| `src/server/manager.rs` | Broadcast `StateChanged{Ports}` in `start_port`, `stop_port`, `restart_port` |
| `src/dashboard/routes/ports.rs` | Broadcast `StateChanged{Ports}` after each mutating handler |
| `src/dashboard/routes/mocks.rs` | Broadcast `StateChanged{Mocks}` after each mutating handler |
| `frontend/src/App.vue` | Connect WebSocket on mount |
| `frontend/src/stores/logs.ts` | Handle `state_changed` message, trigger store refresh |

`src/dashboard/ws.rs` — no changes needed.

## Out of Scope

- Making the TUI receive push updates (it already polls SQLite every 500 ms, which is sufficient)
- Syncing the TUI's mock list in real time (mock changes are rare; polling on the 4-tick interval is adequate)
- Cross-machine sync or multi-daemon scenarios
