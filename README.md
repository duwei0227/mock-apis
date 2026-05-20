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

### Web dashboard

The web dashboard is served automatically whenever the backend is running. There is no separate flag — just start the server and open a browser:

```bash
mock start                        # start as daemon → http://localhost:9999
mock serve                        # foreground → http://localhost:9999
```

### Background daemon mode

```bash
mock start                        # start server in background
mock start --port 8888            # custom management port
mock stop                         # stop background server
mock restart                      # stop + start (reloads all port and API config)
mock status                       # show daemon status and running ports
mock serve                        # foreground server (no TUI), useful for scripts
```

`mock start` spawns the server as a background daemon and prints its PID. The web dashboard is available at `http://localhost:9999`. Use `mock stop` to shut it down cleanly. Use `mock restart` to reload configuration after changing ports or mocks outside the UI.

`mock status` shows whether the daemon is running and, if it is, lists each active port with its label and all enabled API routes:

```
Mock server is running (PID: 12345).

  ● :8080  payments-api
      DELETE  /payments/{id}  (cancel payment)
      GET     /payments       (list payments)
      POST    /payments       (create payment)

  ● :3000  auth-service
      (no enabled APIs)
```

A PID file (`apimock.pid` by default, alongside the database) tracks the running process. `mock stop` reads this file to find and terminate the daemon.

### Multi-client workflow

Run the daemon once and connect multiple UIs to it simultaneously:

```bash
mock start          # start daemon (manages ports, serves dashboard on :9999)
mock                # open TUI — reads state from SQLite, delegates start/stop to daemon
# open http://localhost:9999 in any browser for the web dashboard
```

Changes made in the TUI are visible in the browser within one second; changes made in the browser (or via the API) are visible in the TUI within 500 ms. Both UIs share a single SQLite database as the source of truth.

## Concepts

### Ports

A **port** is an HTTP server instance listening on a specific port number. Ports must be started before they accept requests. You can have multiple ports running simultaneously, each with its own set of mocks.

### Mocks

A **mock** defines how a port responds to a specific request:

| Field | Description |
|-------|-------------|
| Port | Which port server handles this mock |
| Method | HTTP method (`GET`, `POST`, `PUT`, `DELETE`, `ANY`) |
| Path | URL path, supports `{param}` placeholders and `*` wildcards |
| Name | Short label shown in the mock list |
| Description | Optional longer description |
| Request Params | Param names to filter the JSON response by (see below) |
| Response Status | HTTP status code to return (default: 200) |
| Response Headers | Key-value pairs added to the response |
| Response Body | Body text, file path, or template with built-in functions |
| Delay (ms) | Artificial delay before responding |
| Pagination | Enable automatic JSON array pagination (see below) |

**Path matching rules:**
- `{param}` matches a single path segment (e.g. `/users/{id}` matches `/users/42`)
- `*` matches a single segment
- Trailing `*` matches any remaining segments
- Exact method match takes priority over `ANY`
- The `(port, method, path)` combination must be unique

**Body source:**
- **Inline** — type the response body directly
- **File** — enter an absolute path to a `.json` or `.txt` file on the server; the file is read on each request

**Request Params filtering:**

Add one or more param names to a mock to enable response filtering. When a request arrives with a matching query param (or POST body field), the JSON response is filtered to only return items where that field equals the given value.

- Works on `GET` and `POST` mocks only (hidden for `PUT` and `DELETE`)
- Filtering is recursive — finds and filters object arrays at any nesting depth
- Primitive arrays (e.g. `["GET","POST"]`) are never filtered
- Non-JSON responses pass through unchanged
- A param with an empty value (e.g. `?name=`) is ignored — the full dataset is returned

Example: a mock with param `name` and response body `[{"name":"alice"},{"name":"bob"}]` will return only `[{"name":"alice"}]` when called with `?name=alice`.

**Pagination:**

Enable pagination on `GET` / `POST` mocks to automatically slice a JSON array by page. Configure:

| Setting | Description | Default |
|---------|-------------|---------|
| Page param | Query param for the page number | `page` |
| Page size param | Query param for items per page | `pageSize` (10 if not sent) |
| Data field | Dot-notation path to the array (e.g. `body.list`); empty = top-level array | — |
| Total field | Dot-notation path to write the total count back (e.g. `body.total`); empty = skip | — |

When **Data field** is empty the response is replaced with a `{items, total, page, page_size}` envelope. When it is set, the original JSON structure is preserved and only the target array is replaced with the current page slice.

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
| `r` | Switch to Request logs |
| `s` | Switch to System logs |
| `c` | Clear the active log tab (also removes from database) |

## CLI Reference

```
mock [OPTIONS] [COMMAND]

Commands:
  start    Start the mock server as a background daemon
  stop     Stop the background mock server
  restart  Stop then start the daemon (reloads all port and API config)
  status   Show daemon status, running ports, and enabled API routes
  serve    Run the mock server in the foreground (ports + web dashboard)

Options:
      --port <PORT>     Management port for dashboard/serve mode [default: 9999]
      --db <DB>         Path to the SQLite database file [default: apimock.db]
  -h, --help            Print help
```

Running `mock` with no subcommand launches the TUI. The web dashboard is available at `http://localhost:9999` whenever `mock start` or `mock serve` is running.

## Data Storage

All configuration and logs are stored in a SQLite database file (`apimock.db` by default). The `.db-shm` and `.db-wal` files alongside it are normal SQLite WAL-mode auxiliary files and can be ignored in version control.
