# Changelog

## [0.2.0] - 2026-05-07

### Added

- **`mock restart` subcommand** — stops the running daemon and starts a fresh one,
  reloading all port and API configuration from the database.
- **`mock status` detail output** — shows each running port with its label and lists
  every enabled API route (method, path, name) beneath it.
- **Live two-way sync between TUI, dashboard, and daemon** — changes made in any UI
  are reflected in the others in near-real-time via WebSocket `state_changed` events.
  - `LogEvent` gains a `StateChanged { resource }` variant (`ports` or `mocks`)
    broadcast after every mutating operation in `LivePortManager` and all dashboard
    route handlers.
  - Vue frontend reconnects to WebSocket on app mount (not only in the Logs view)
    and re-fetches the affected Pinia store on receipt of a `state_changed` event.
- **Port runtime state in SQLite** — `port_configs` table tracks `running` and
  `owner_pid` so any process can read accurate port status without a TCP probe
  (migration `0007_port_runtime_status`).
- **Daemon-aware TUI** — when a daemon is running the TUI delegates start, stop, and
  restart operations to the daemon's HTTP API instead of trying to bind ports locally.
- **`POST /api/v1/ports/:id/restart`** — new dashboard API endpoint for restarting a
  port and reloading its mock snapshot from the database.
- **Action button labels and tooltips in the web dashboard** — port action buttons now
  show icon + text label ("Start", "Stop", "Edit", "Delete") with a hover tooltip.
  PrimeVue `Tooltip` directive registered globally in `main.ts`.

### Fixed

- **Startup conflict** — TUI or dashboard launched alongside a running daemon no longer
  calls `start_all_enabled`, preventing races for port ownership and duplicate-bind
  errors (`is_external_daemon_running` guard in `main.rs`).
- **`mock --dashboard` with daemon running** — instead of crashing with "Address already
  in use", the command now prints the daemon's existing dashboard URL and attempts to
  open it in the system browser.
- **TUI mock operations with daemon running** — enabling/disabling, creating, editing,
  and deleting mocks now delegate the port restart to the daemon so changes take effect
  immediately, instead of being silently dropped by the local port manager.
- **Port delete with daemon running** — the TUI now stops the daemon-owned server before
  deleting the database record, instead of leaving the port running indefinitely.
- **WebSocket lifetime regression** — `LogsView` was disconnecting the shared WebSocket
  on unmount; ownership moved to `App.vue` so the connection survives view navigation.
- **Standalone TUI stop** — stopping a port in standalone TUI mode now also sets
  `enabled = false` so the port does not auto-restart on the next launch.

---

## [0.1.0] - 2026-05-05

### Added

#### Core
- SQLite-backed storage for ports, mocks, and request/system logs
- `LivePortManager` — manages independent Axum HTTP server per port, with start/stop/restart
- Request handler with glob-style path matching (`{param}`, `*`, trailing `*`)
- `ANY` method fallback; exact method match takes priority
- Artificial response delay support per mock
- File-based response body (`file://` prefix reads from local path on each request)
- Real-time event broadcast via `tokio::broadcast` for live log streaming

#### Template functions
- `{{date}}` / `{{date:format}}` — current local date
- `{{time}}` / `{{time:format}}` — current local time
- `{{datetime}}` / `{{datetime:format}}` — current local date and time
- `{{randomInt}}` / `{{randomInt:min:max}}` — random integer
- `{{randomFloat}}` / `{{randomFloat:min:max:decimals}}` — random float
- `{{randomString}}` / `{{randomString:length}}` — random alphanumeric string
- `{{randomChinese}}` / `{{randomChinese:length}}` — random common simplified Chinese characters
- `{{uuid}}` — random UUID v4

#### TUI (terminal UI)
- Ports tab — list, create, edit, delete ports; toggle on/off with Space (persists `enabled` state to database)
- Mocks tab — list, create, edit, delete, enable/disable mocks; port display
- Logs tab — request and system log viewer with detail panel; follow mode
- Functions tab — built-in template function reference table
- `?` overlay for quick function help from any tab (outside modals); `F1` opens the same overlay from anywhere including inside modals
- Quit confirmation dialog when ports are running, with suggestion to use `mock start` for persistent background use
- Port conflict and validation error messages in modals
- Cursor rendering in modal input fields

#### Web dashboard
- REST API under `/api/v1`: full CRUD for ports, mocks, and logs
- WebSocket endpoint `/ws/logs` for real-time log streaming
- Vue 3 + PrimeVue frontend embedded into binary via `rust-embed`
- Ports page — create, edit, delete, start/stop ports
- Mocks page — create, edit, delete, duplicate, enable/disable mocks; split detail panel
- Logs page — request/system log tables with Port and Path filters, detail dialog
- Functions page — built-in template function reference
- Dark mode toggle
- Pagination for log tables

#### Infrastructure
- `--dashboard` flag to launch web dashboard instead of TUI
- `--port` flag to set management port (default: 9999)
- `--db` flag to set SQLite database path (default: `apimock.db`)
- Background daemon mode — `mock start` spawns the server as a background process (calls `setsid` on Unix to survive terminal close); `mock stop` terminates it via a PID file; `mock status` reports whether the daemon is running
- `mock serve` subcommand — runs the server in the foreground (ports + web dashboard) without a TUI; handles `SIGTERM` and `Ctrl+C` for clean shutdown
- PID file written alongside the database (`<db-stem>.pid`); stale files are cleaned up automatically on next `mock start`
- GitHub Actions release workflow for Linux (musl static) and Windows binaries

### Fixed

- `start_all_enabled` logs a warning and continues if a port address is already in use, rather than aborting
