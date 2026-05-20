# Changelog

## [0.3.0] - 2026-05-20

### Added

- **Request param filtering** — configure one or more param names on a mock; incoming
  requests whose query string (or POST body) contains a matching key-value pair will have
  the JSON response filtered to only return items where that field equals the given value.
  Filtering is recursive: works on top-level arrays, nested object arrays, and any depth
  of wrapping. Primitive arrays (e.g. enum lists) are never filtered. Only applies to
  JSON responses; non-JSON bodies pass through unchanged. Empty param values are ignored
  (no filtering applied). PUT and DELETE mocks do not support param filtering.
- **Pagination** — enable pagination on GET / POST mocks with configurable query param
  names, data field path, and total field path:
  - **Page param** — query param for the page number (default `page`)
  - **Page size param** — query param for items per page (default `pageSize`); defaults
    to 10 items when not supplied in the request
  - **Data field** — field path containing the array to paginate; supports dot-notation
    for nested fields (e.g. `body.list`); empty = top-level array, wrapped in a
    `{items, total, page, page_size}` envelope
  - **Total field** — field path to overwrite with the computed total count; supports
    dot-notation (e.g. `body.total`); empty = skip
- **Address column in Mocks list** — both the web dashboard and TUI now show `IP:Port`
  in the mock list. The dashboard column includes a copy button.
- **TUI full pagination config** — the mock create/edit modal exposes all four pagination
  fields (Page param, Page size param, Data field, Total field) when Pagination is
  enabled, matching the web dashboard.
- **TUI Request Params chips display** — inactive Request Params field renders each param
  as a `[name]` chip; shows a placeholder hint when empty; `+` appends a `|` separator
  to add another entry.
- **TUI Response Headers chips display** — same chip / placeholder / `+` behaviour as
  Request Params.
- **System IP column in TUI Ports tab** — the local machine IP is resolved at startup
  and shown left of the Port column so you can see the full access address at a glance.
- **Log clear key in TUI** — press `c` on the Logs tab to clear the active log tab
  (Requests or System) from both memory and the database.
- **Scrollable request detail overlay** — when the detail popup is open, `↑`/`↓` scroll
  through long header lists or large bodies; scroll resets automatically on close.
- **Body field scroll and clear in TUI modal** — `↑`/`↓` scroll the Response Body field
  while editing; `Ctrl+U` clears the entire field instantly. The field title shows the
  current line count. Long lines wrap inside the field.
- **Live log polling for daemon-owned ports** — the TUI now polls the database every
  second for new log entries, so request logs appear in real time even when ports are
  managed by a background daemon (the broadcast channel only works within one process).

### Changed

- **`--dashboard` flag removed** — the web dashboard is always served at
  `http://localhost:9999` whenever the server is running (`mock start` or `mock serve`).
  There is no longer a separate flag to switch between TUI and dashboard modes.
- **TUI Logs tab always follows newest** — the follow-mode toggle (`f`) has been removed.
  The log view always shows the most recent entry at the top; `↑`/`↓` navigate for
  detail selection without leaving follow mode.
- **Port filter labels** — ports without a label now show only the port number (e.g.
  `8082`) in the Mocks page filter dropdown instead of `8082 — unnamed`.

### Fixed

- **Clipboard copy on plain HTTP (Windows LAN)** — `navigator.clipboard` is unavailable
  in non-secure contexts; copy buttons now fall back to `document.execCommand('copy')`
  so they work when the dashboard is accessed via a LAN IP on Windows.
- **Empty request param values ignored** — if a configured param is sent with an empty
  value (e.g. `?name=`), it is excluded from the filter set and the full dataset is
  returned instead of filtering against an empty string.
- **Windows CI build** — `build.rs` now skips the npm build step when `frontend/dist/`
  already exists (pre-built by CI) and uses `npm.cmd` on Windows to avoid
  `program not found` errors.
- **Dark mode splitter gutter** — the white divider line between the mock list and detail
  panel in the web dashboard is now styled to match the dark surface colour.
- **Log detail visual hierarchy** — sub-section labels (`Request Headers`, `Request Body`,
  `Response Headers`, `Response Body`) are now rendered in cyan+bold to clearly separate
  them from field labels; header keys are highlighted in yellow; empty/none markers are
  dimmed. Headers are sorted alphabetically.

---

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
- **Dashboard port conflict with daemon running** — launching a second server instance
  no longer crashes with "Address already in use"; the guard in `main.rs` detects a
  running daemon and skips the duplicate bind.
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
- Web dashboard always served at `http://localhost:9999` when the server is running
- `--port` flag to set management port (default: 9999)
- `--db` flag to set SQLite database path (default: `apimock.db`)
- Background daemon mode — `mock start` spawns the server as a background process (calls `setsid` on Unix to survive terminal close); `mock stop` terminates it via a PID file; `mock status` reports whether the daemon is running
- `mock serve` subcommand — runs the server in the foreground (ports + web dashboard) without a TUI; handles `SIGTERM` and `Ctrl+C` for clean shutdown
- PID file written alongside the database (`<db-stem>.pid`); stale files are cleaned up automatically on next `mock start`
- GitHub Actions release workflow for Linux (musl static) and Windows binaries

### Fixed

- `start_all_enabled` logs a warning and continues if a port address is already in use, rather than aborting
