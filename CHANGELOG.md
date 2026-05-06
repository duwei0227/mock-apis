# Changelog

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
- `--db` flag to set SQLite database path (default: `mock-apis.db`)
- Background daemon mode — `mock start` spawns the server as a background process (calls `setsid` on Unix to survive terminal close); `mock stop` terminates it via a PID file; `mock status` reports whether the daemon is running
- `mock serve` subcommand — runs the server in the foreground (ports + web dashboard) without a TUI; handles `SIGTERM` and `Ctrl+C` for clean shutdown
- PID file written alongside the database (`<db-stem>.pid`); stale files are cleaned up automatically on next `mock start`
- GitHub Actions release workflow for Linux (musl static) and Windows binaries

### Fixed

- `start_all_enabled` logs a warning and continues if a port address is already in use, rather than aborting
