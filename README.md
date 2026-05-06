# apimock

A developer tool for defining HTTP endpoints that return canned responses. Supports a terminal UI (TUI) and a web dashboard, sharing the same backend.

## Installation

### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/duwei0227/apimock/main/install.sh | bash
```

Installs to `/usr/local/bin` (if writable) or `~/.local/bin`. After installation, `mock` is available directly in your terminal.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/duwei0227/apimock/main/install.ps1 | iex
```

Installs to `%USERPROFILE%\.local\bin` and adds it to your user PATH automatically.

### Manual download

Download the latest binary for your platform from the [Releases](../../releases) page.

| Platform | File |
|----------|------|
| Linux (static) | `mock-linux-x86_64.tar.gz` |
| Windows | `mock-windows-x86_64.zip` |

### Build from source

Prerequisites: Rust (stable), Node.js 20+

```bash
git clone <repo-url>
cd apimock
cargo build --release
# Binary: target/release/mock
```

The frontend is built automatically by `build.rs` during `cargo build`.

## Usage

### TUI mode (default)

```bash
mock
mock --db my-project.db   # custom database file
```

Launches a terminal UI with tabs for Ports, Mocks, Logs, and Functions.

### Web dashboard mode

```bash
mock --dashboard
mock --dashboard --port 8888   # custom management port (default: 9999)
```

Opens a web dashboard at `http://<local-ip>:9999`. The URL is printed on startup.

### Background daemon mode

```bash
mock start                        # start server in background
mock start --port 8888            # custom management port
mock stop                         # stop background server
mock status                       # check if server is running
mock serve                        # foreground server (no TUI), useful for scripts
```

`mock start` spawns the server as a background daemon and prints its PID. The web dashboard is available at `http://localhost:9999`. Use `mock stop` to shut it down cleanly.

A PID file (`apimock.pid` by default, alongside the database) tracks the running process. `mock stop` reads this file to find and terminate the daemon.

## Concepts

### Ports

A **port** is an HTTP server instance listening on a specific port number. Ports must be started before they accept requests. You can have multiple ports running simultaneously, each with its own set of mocks.

### Mocks

A **mock** defines how a port responds to a specific request:

| Field | Description |
|-------|-------------|
| Port | Which port server handles this mock |
| Method | HTTP method (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`, `ANY`) |
| Path | URL path, supports `{param}` placeholders and `*` wildcards |
| Response Status | HTTP status code to return (default: 200) |
| Response Headers | Key-value pairs added to the response |
| Response Body | Body text, file path, or template with built-in functions |
| Delay (ms) | Artificial delay before responding |

**Path matching rules:**
- `{param}` matches a single path segment (e.g. `/users/{id}` matches `/users/42`)
- `*` matches a single segment
- Trailing `*` matches any remaining segments
- Exact method match takes priority over `ANY`
- The `(port, method, path)` combination must be unique

**Body source:**
- **Inline** — type the response body directly
- **File** — enter an absolute path to a `.json` or `.txt` file on the server; the file is read on each request

### Logs

Incoming requests and system events are logged in real time. Click any request row to see full headers and body detail. Use the Port and Path filters to narrow results.

## Template Functions

Use `{{function}}` or `{{function:arg}}` placeholders in the **Response Body** field. They are evaluated fresh on every request.

| Function | Syntax | Default | Example output |
|----------|--------|---------|----------------|
| `date` | `{{date}}` or `{{date:format}}` | `yyyyMMdd` | `20260503` |
| `time` | `{{time}}` or `{{time:format}}` | `HHmmss` | `143025` |
| `datetime` | `{{datetime}}` or `{{datetime:format}}` | `yyyyMMddHHmmss` | `20260503143025` |
| `randomInt` | `{{randomInt}}` or `{{randomInt:min:max}}` | 0–100 | `42` |
| `randomFloat` | `{{randomFloat}}` or `{{randomFloat:min:max:decimals}}` | 0.0–1.0, 2dp | `0.73` |
| `randomString` | `{{randomString}}` or `{{randomString:length}}` | 10 chars | `aB3kFz9Qmw` |
| `randomChinese` | `{{randomChinese}}` or `{{randomChinese:length}}` | 15 chars | `的一是在人有我` |
| `uuid` | `{{uuid}}` | — | `550e8400-e29b-41d4-a716-446655440000` |

**Date/time format tokens:** `yyyy` year · `MM` month · `dd` day · `HH` hour · `mm` minute · `ss` second

**Example response body:**

```json
{
  "id": "{{uuid}}",
  "name": "{{randomString:8}}",
  "score": {{randomInt:0:100}},
  "createdAt": "{{datetime:yyyy-MM-dd HH:mm:ss}}"
}
```

## TUI Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `1` / `2` / `3` / `4` | Switch to Ports / Mocks / Logs / Functions tab |
| `Tab` | Next tab |
| `?` | Toggle built-in functions help overlay (outside modals) |
| `F1` | Toggle built-in functions help overlay (anywhere, including inside modals) |
| `q` | Quit |

### Ports tab

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Navigate list |
| `n` | New port |
| `e` | Edit selected port |
| `d` | Delete selected port |
| `Space` | Toggle port on / off |

### Mocks tab

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Navigate list |
| `n` | New mock |
| `e` | Edit selected mock |
| `d` | Delete selected mock |
| `Space` | Toggle mock enabled / disabled |

### Logs tab

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Navigate list |
| `Enter` | Open / close request detail |
| `Esc` | Close detail |
| `f` | Toggle follow mode (auto-scroll to newest) |
| `r` | Switch to Request logs |
| `s` | Switch to System logs |

## CLI Reference

```
mock [OPTIONS] [COMMAND]

Commands:
  start   Start the mock server as a background daemon
  stop    Stop the background mock server
  status  Show status of the background mock server
  serve   Run the mock server in the foreground (ports + web dashboard)

Options:
      --dashboard       Launch the web dashboard instead of the TUI (no subcommand only)
      --port <PORT>     Management port for dashboard/serve mode [default: 9999]
      --db <DB>         Path to the SQLite database file [default: apimock.db]
  -h, --help            Print help
```

## Data Storage

All configuration and logs are stored in a SQLite database file (`apimock.db` by default). The `.db-shm` and `.db-wal` files alongside it are normal SQLite WAL-mode auxiliary files and can be ignored in version control.
