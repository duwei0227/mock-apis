# Live Two-Way Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable TUI and web dashboard to both reflect live state when a background daemon is running, with changes in either UI immediately visible in the other.

**Architecture:** SQLite is the shared source of truth; the daemon owns port execution. A new `StateChanged` variant is added to the existing `LogEvent` broadcast channel — the daemon emits it after every port/mock mutation, `ws.rs` forwards it to the browser over the already-connected WebSocket, and the Vue frontend re-fetches the affected resource on receipt. A companion `is_external_daemon_running()` check prevents the TUI from racing with the daemon for port ownership at startup.

**Tech Stack:** Rust (tokio, axum, serde_json, tokio-rusqlite), Vue 3 + Pinia + TypeScript

---

## File Map

| File | Change |
|---|---|
| `src/models/mod.rs` | Add `StateResource` enum; add `StateChanged` variant to `LogEvent` |
| `src/daemon.rs` | Add `is_external_daemon_running()` |
| `src/main.rs` | Gate `start_all_enabled()` behind external-daemon check |
| `src/server/manager.rs` | Broadcast `StateChanged{Ports}` in `start_port`, `stop_port`, `restart_port` |
| `src/dashboard/routes/ports.rs` | Broadcast `StateChanged{Ports}` from each mutating handler |
| `src/dashboard/routes/mocks.rs` | Broadcast `StateChanged{Mocks}` from each mutating handler |
| `src/dashboard/ws.rs` | **No changes** — already serializes all `LogEvent` variants |
| `frontend/src/App.vue` | Call `logsStore.connectLive()` on mount |
| `frontend/src/stores/logs.ts` | Handle `state_changed` message; trigger store re-fetch |

---

### Task 1: Add StateResource and StateChanged to LogEvent

**Files:**
- Modify: `src/models/mod.rs`

- [ ] **Step 1: Extend the LogEvent enum**

Open `src/models/mod.rs`. The bottom of the file currently reads:

```rust
/// Unified log event broadcast over the internal channel and WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LogEvent {
    Request(RequestLog),
    System(SystemLog),
}
```

Replace it with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateResource {
    Ports,
    Mocks,
}

/// Unified log event broadcast over the internal channel and WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LogEvent {
    Request(RequestLog),
    System(SystemLog),
    StateChanged { resource: StateResource },
}
```

`StateChanged { resource: StateResource::Ports }` serializes to `{"type":"state_changed","resource":"ports"}`.

- [ ] **Step 2: Verify it compiles**

```bash
cargo check
```

Expected: no errors. Any exhaustive match on `LogEvent` elsewhere will now produce a compile error listing the missing arm — fix those as they appear (none exist yet in the current codebase).

- [ ] **Step 3: Commit**

```bash
git add src/models/mod.rs
git commit -m "feat: add StateResource enum and StateChanged variant to LogEvent"
```

---

### Task 2: Add is_external_daemon_running to daemon.rs

**Files:**
- Modify: `src/daemon.rs`

- [ ] **Step 1: Add the helper function**

Open `src/daemon.rs`. After the `is_process_alive` function (currently ending around line 47), add:

```rust
/// Returns true if a daemon process *other than the current process* owns the PID file.
/// Prevents TUI/dashboard from calling start_all_enabled when a daemon already manages ports.
pub fn is_external_daemon_running(db: &str) -> bool {
    let my_pid = std::process::id();
    match read_pid(db) {
        Some(pid) if pid != my_pid => is_process_alive(pid),
        _ => false,
    }
}
```

The `my_pid != pid` guard prevents the daemon itself (`mock serve`) — whose own PID is written to the file — from treating itself as an external daemon.

- [ ] **Step 2: Verify it compiles**

```bash
cargo check
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/daemon.rs
git commit -m "feat: add is_external_daemon_running helper to daemon module"
```

---

### Task 3: Gate start_all_enabled when external daemon is running

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Wrap start_all_enabled with the daemon check**

Open `src/main.rs`. Find this line (around line 61):

```rust
    port_manager.start_all_enabled().await?;
```

Replace it with:

```rust
    if !daemon::is_external_daemon_running(&cli.db) {
        port_manager.start_all_enabled().await?;
    }
```

Behavior after this change:

| Process | Daemon running? | `start_all_enabled` called? |
|---|---|---|
| `mock serve` (the daemon itself) | — | Yes (own PID ≠ external) |
| `mock` (TUI, no daemon) | No | Yes — standalone mode |
| `mock` (TUI, daemon alive) | Yes | No — daemon owns ports |
| `mock --dashboard` (no daemon) | No | Yes — standalone mode |
| `mock --dashboard` (daemon alive) | Yes | No — daemon owns ports |

- [ ] **Step 2: Build to confirm**

```bash
cargo build
```

Expected: compiles cleanly.

- [ ] **Step 3: Quick smoke test (requires a DB with at least one enabled port)**

```bash
# Terminal 1
./target/debug/mock start
# Expected output: "Mock server started (PID: XXXX)"
# Expected output: "Dashboard: http://localhost:9999"

# Terminal 2
./target/debug/mock
# Expected: TUI opens with no "address already in use" errors in either terminal.
# Port status in TUI should match what the daemon is running.

# Cleanup
./target/debug/mock stop
```

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "fix: skip start_all_enabled when an external daemon already owns ports"
```

---

### Task 4: Broadcast StateChanged{Ports} in LivePortManager

**Files:**
- Modify: `src/server/manager.rs`

- [ ] **Step 1: Extend the models import**

Open `src/server/manager.rs`. Find:

```rust
use crate::models::LogEvent;
```

Replace with:

```rust
use crate::models::{LogEvent, StateResource};
```

- [ ] **Step 2: Broadcast in start_port**

In the `start_port` method, the last three lines currently read:

```rust
        handles.insert(port_id, handle);
        drop(handles);
        let _ = self.port_store.set_port_running(port_id, true, Some(our_pid)).await;
        Ok(())
```

Replace those last two lines with:

```rust
        handles.insert(port_id, handle);
        drop(handles);
        let _ = self.port_store.set_port_running(port_id, true, Some(our_pid)).await;
        let _ = self.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
        Ok(())
```

- [ ] **Step 3: Broadcast in stop_port**

In the `stop_port` method, the `if let Some` block currently ends with:

```rust
    if let Some((join, token)) = handle {
        token.cancel();
        join.await.ok();
        // Keep our PID so the reconciliation loop treats this as "intentionally stopped"
        // and doesn't immediately restart. Stale-PID detection handles cleanup on restart.
        let our_pid = std::process::id();
        let _ = self.port_store.set_port_running(port_id, false, Some(our_pid)).await;
    }
    Ok(())
```

Replace with:

```rust
    if let Some((join, token)) = handle {
        token.cancel();
        join.await.ok();
        // Keep our PID so the reconciliation loop treats this as "intentionally stopped"
        // and doesn't immediately restart. Stale-PID detection handles cleanup on restart.
        let our_pid = std::process::id();
        let _ = self.port_store.set_port_running(port_id, false, Some(our_pid)).await;
        let _ = self.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
    }
    Ok(())
```

- [ ] **Step 4: Broadcast in restart_port**

In the `restart_port` method, find the closing block:

```rust
        if config.map(|c| c.enabled).unwrap_or(false) {
            let handle = self.spawn_server(port_id).await?;
            let our_pid = std::process::id();
            let _ = self.port_store.set_port_running(port_id, true, Some(our_pid)).await;
            let mut handles = self.handles.lock().await;
            handles.insert(port_id, handle);
        }
        Ok(())
```

Replace with:

```rust
        if config.map(|c| c.enabled).unwrap_or(false) {
            let handle = self.spawn_server(port_id).await?;
            let our_pid = std::process::id();
            let _ = self.port_store.set_port_running(port_id, true, Some(our_pid)).await;
            let mut handles = self.handles.lock().await;
            handles.insert(port_id, handle);
        }
        let _ = self.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
        Ok(())
```

- [ ] **Step 5: Verify it compiles**

```bash
cargo check
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add src/server/manager.rs
git commit -m "feat: broadcast StateChanged{Ports} in LivePortManager start/stop/restart"
```

---

### Task 5: Broadcast StateChanged{Ports} in dashboard port route handlers

**Files:**
- Modify: `src/dashboard/routes/ports.rs`

- [ ] **Step 1: Add the models import**

Open `src/dashboard/routes/ports.rs`. After the existing `use crate::AppState;` line, add:

```rust
use crate::models::{LogEvent, StateResource};
```

- [ ] **Step 2: Broadcast in create_port**

Find the success arm of the `create_port` match:

```rust
        Ok(p) => (StatusCode::CREATED, Json(p)).into_response(),
```

Replace with:

```rust
        Ok(p) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            (StatusCode::CREATED, Json(p)).into_response()
        }
```

- [ ] **Step 3: Broadcast in update_port**

Find the success arm of the `update_port` match:

```rust
        Ok(p) => Json(p).into_response(),
```

Replace with:

```rust
        Ok(p) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            Json(p).into_response()
        }
```

- [ ] **Step 4: Broadcast in delete_port**

Find the success arm of the `delete_port` match:

```rust
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
```

The `delete_port` function has two `Ok(())` arms — target the one inside `match state.port_store.delete_port(id).await`. Replace:

```rust
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
```

with:

```rust
        Ok(()) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            StatusCode::NO_CONTENT.into_response()
        }
```

- [ ] **Step 5: Broadcast in start_port route handler**

Find the `start_port` route function (not the manager method). Its success arm:

```rust
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn stop_port(
```

Replace the `Ok(())` arm with:

```rust
        Ok(()) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            StatusCode::NO_CONTENT.into_response()
        }
```

- [ ] **Step 6: Broadcast in stop_port route handler**

Find the `stop_port` route function. Its success arm:

```rust
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn port_status(
```

Replace the `Ok(())` arm with:

```rust
        Ok(()) => {
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Ports });
            StatusCode::NO_CONTENT.into_response()
        }
```

- [ ] **Step 7: Verify it compiles**

```bash
cargo check
```

Expected: no errors.

- [ ] **Step 8: Commit**

```bash
git add src/dashboard/routes/ports.rs
git commit -m "feat: broadcast StateChanged{Ports} from all port route handlers"
```

---

### Task 6: Broadcast StateChanged{Mocks} in dashboard mock route handlers

**Files:**
- Modify: `src/dashboard/routes/mocks.rs`

- [ ] **Step 1: Add the models import**

Open `src/dashboard/routes/mocks.rs`. After the existing imports, add:

```rust
use crate::models::{LogEvent, StateResource};
```

- [ ] **Step 2: Broadcast in create_mock**

Find the success arm of `create_mock`:

```rust
        Ok(m) => {
            // Restart the port server so the new mock is active.
            let _ = state.port_manager.restart_port(m.port_id).await;
            (StatusCode::CREATED, Json(m)).into_response()
        }
```

Replace with:

```rust
        Ok(m) => {
            let _ = state.port_manager.restart_port(m.port_id).await;
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Mocks });
            (StatusCode::CREATED, Json(m)).into_response()
        }
```

- [ ] **Step 3: Broadcast in update_mock**

Find the success arm of `update_mock`:

```rust
        Ok(m) => {
            let _ = state.port_manager.restart_port(m.port_id).await;
            Json(m).into_response()
        }
```

Replace with:

```rust
        Ok(m) => {
            let _ = state.port_manager.restart_port(m.port_id).await;
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Mocks });
            Json(m).into_response()
        }
```

- [ ] **Step 4: Broadcast in delete_mock**

Find the success arm of `delete_mock`:

```rust
        Ok(()) => {
            if let Some(pid) = port_id {
                let _ = state.port_manager.restart_port(pid).await;
            }
            StatusCode::NO_CONTENT.into_response()
        }
```

Replace with:

```rust
        Ok(()) => {
            if let Some(pid) = port_id {
                let _ = state.port_manager.restart_port(pid).await;
            }
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Mocks });
            StatusCode::NO_CONTENT.into_response()
        }
```

- [ ] **Step 5: Broadcast in set_mock_enabled**

Find the success arm of `set_mock_enabled`:

```rust
        Ok(()) => {
            if let Some(pid) = port_id {
                let _ = state.port_manager.restart_port(pid).await;
            }
            StatusCode::NO_CONTENT.into_response()
        }
```

Replace with:

```rust
        Ok(()) => {
            if let Some(pid) = port_id {
                let _ = state.port_manager.restart_port(pid).await;
            }
            let _ = state.log_tx.send(LogEvent::StateChanged { resource: StateResource::Mocks });
            StatusCode::NO_CONTENT.into_response()
        }
```

- [ ] **Step 6: Build the full backend**

```bash
cargo build
```

Expected: no errors. The backend is now complete. `ws.rs` requires no changes — it already serializes and forwards every `LogEvent` variant via `serde_json::to_string(&event)`.

- [ ] **Step 7: Commit**

```bash
git add src/dashboard/routes/mocks.rs
git commit -m "feat: broadcast StateChanged{Mocks} from all mock route handlers"
```

---

### Task 7: Connect WebSocket on App.vue mount

**Files:**
- Modify: `frontend/src/App.vue`

The WebSocket is currently connected only when the user opens the Logs view. Moving it to `App.vue` keeps it alive regardless of which tab is open.

- [ ] **Step 1: Update App.vue**

Open `frontend/src/App.vue`. Replace the entire file with:

```vue
<template>
  <div class="flex h-screen bg-surface-100 dark:bg-surface-950">
    <AppSidebar />
    <div class="flex flex-col flex-1 overflow-hidden">
      <AppTopBar />
      <main class="flex-1 overflow-auto p-5">
        <router-view />
      </main>
    </div>
  </div>
  <Toast position="bottom-right" />
  <ConfirmDialog />
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import Toast from 'primevue/toast'
import ConfirmDialog from 'primevue/confirmdialog'
import AppSidebar from './components/layout/AppSidebar.vue'
import AppTopBar from './components/layout/AppTopBar.vue'
import { useLogsStore } from './stores/logs'

const logsStore = useLogsStore()
onMounted(() => logsStore.connectLive())
onUnmounted(() => logsStore.disconnectLive())
</script>
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/App.vue
git commit -m "feat: connect WebSocket on app mount so live events work on all views"
```

---

### Task 8: Handle state_changed in logs store and build frontend

**Files:**
- Modify: `frontend/src/stores/logs.ts`

- [ ] **Step 1: Add store imports**

Open `frontend/src/stores/logs.ts`. After the existing import line:

```typescript
import { LogsApi, type RequestLog, type SystemLog, type LogPage } from '../api/client'
```

Add:

```typescript
import { usePortsStore } from './ports'
import { useMocksStore } from './mocks'
```

- [ ] **Step 2: Handle state_changed in onmessage**

Find the `ws.onmessage` handler. It currently reads:

```typescript
    ws.onmessage = (ev) => {
      try {
        const payload = JSON.parse(ev.data)
        if (payload.type === 'request') {
          liveEvents.value.unshift({ type: 'request', request: payload })
        } else if (payload.type === 'system') {
          liveEvents.value.unshift({ type: 'system', system: payload })
        }
        // Keep at most 200 live events in memory.
        if (liveEvents.value.length > 200) liveEvents.value.length = 200
      } catch { /* ignore parse errors */ }
    }
```

Replace it with:

```typescript
    ws.onmessage = (ev) => {
      try {
        const payload = JSON.parse(ev.data)
        if (payload.type === 'state_changed') {
          if (payload.resource === 'ports') {
            usePortsStore().fetchPorts()
          } else if (payload.resource === 'mocks') {
            const mocksStore = useMocksStore()
            mocksStore.fetchMocks(mocksStore.selectedPortId ?? undefined)
          }
          return
        }
        if (payload.type === 'request') {
          liveEvents.value.unshift({ type: 'request', request: payload })
        } else if (payload.type === 'system') {
          liveEvents.value.unshift({ type: 'system', system: payload })
        }
        // Keep at most 200 live events in memory.
        if (liveEvents.value.length > 200) liveEvents.value.length = 200
      } catch { /* ignore parse errors */ }
    }
```

`selectedPortId` is the currently active port filter in the mocks view; passing it preserves the user's current view context.

- [ ] **Step 3: Build the frontend**

```bash
cd frontend && npm run build
```

Expected: build succeeds with no TypeScript errors, output in `frontend/dist/`.

- [ ] **Step 4: Build the full binary (embeds the compiled frontend)**

```bash
cd .. && cargo build
```

Expected: no errors.

- [ ] **Step 5: End-to-end test**

```bash
# Start the daemon
./target/debug/mock start

# Open the dashboard in a browser
# http://localhost:9999

# Open the TUI in a separate terminal
./target/debug/mock

# --- Test: TUI → Dashboard ---
# In TUI: navigate to Ports tab, press Space on a stopped port
# Expected: dashboard browser auto-refreshes and shows the port as Running
#           within ~1 second (no manual page refresh needed)

# --- Test: Dashboard → TUI ---
# In browser: click Stop on a running port
# Expected: TUI shows the port as Stopped within ~500 ms (next poll cycle)

# --- Test: Dashboard → Dashboard (mock change) ---
# In browser: create a new mock via the dashboard
# Expected: mocks list in the browser updates immediately (WebSocket re-fetch)

# Cleanup
./target/debug/mock stop
```

- [ ] **Step 6: Commit**

```bash
git add frontend/src/stores/logs.ts
git commit -m "feat: re-fetch ports/mocks stores on state_changed WebSocket events"
```
