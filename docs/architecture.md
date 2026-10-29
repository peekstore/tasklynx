# Architecture

Tasklynx is a small synchronous CLI with two optional front ends. This page describes the module layout, data flow, and storage guarantees.

## Modules

| File | Purpose |
| --- | --- |
| `src/main.rs` | Clap definitions, command dispatch, and user-facing output |
| `src/model.rs` | `Task`, `Status`, `Priority`, `ProjectData`, `AppConfig`, date parsing |
| `src/store.rs` | JSON load/save with atomic writes, lookup helpers, export |
| `src/config.rs` | Config load/save and path resolution |
| `src/tui.rs` | Ratatui terminal Kanban board |
| `src/web.rs` | Axum server, REST endpoints, embedded assets |
| `build.rs` | Windows icon embedding |

## Data flow

```mermaid
graph LR
  CLI[main.rs] --> Model[model.rs]
  CLI --> Store[store.rs]
  CLI --> TUI[tui.rs]
  CLI --> Web[web.rs]
  CLI --> Config[config.rs]
  Store --> JSON[(tasks.json)]
  Config --> TOML[(config.toml)]
```

`main.rs` loads `ProjectData` once, dispatches the subcommand, mutates the in-memory model, and saves it back. The TUI and web server receive a snapshot of the same data; the web server holds it behind a mutex and persists changes on each write.

## Data model

`ProjectData` is the root document:

```json
{
  "version": 1,
  "current_branch": "main",
  "branches": ["main"],
  "tasks": []
}
```

Each `Task` carries an ID, title, description, status (`todo`/`doing`/`done`), priority (`low`/`normal`/`high`/`urgent`), project, branch, tags, optional due date, and timestamps.

Serde deserialization uses defaults for missing fields, so older stores keep loading as the schema grows. `version` exists to support future migrations.

## Storage guarantees

- **One file** — `tasks.json` in the platform data directory, or under `TASKLYNX_HOME`.
- **Atomic writes & durability** — the store serializes to `tasks.json.tmp`, calls `sync_all()` for filesystem durability, and renames over the target. A crash cannot leave a partially written file.
- **Cross-process safety** — mutating operations acquire an advisory file lock (`.lock`) with timeout and stale-lock recovery, preventing race conditions between concurrent CLI or web server instances.

## Config

Config is separate from data:

- `~/.config/peekstore/tasklynx/config.toml` (or `$XDG_CONFIG_HOME/...`).
- Missing or partial config safely falls back to `AppConfig::default()`.

## Web server

- Built on `axum` and `tokio`.
- Serves an embedded single-page application and SVG favicon via `include_str!`, keeping the binary fully self-contained.
- JSON API: `GET /api/tasks`, `POST /api/tasks`, `PUT /api/tasks/{id}`, `DELETE /api/tasks/{id}`.
- Hardened with `DefaultBodyLimit` (2MB) and security headers (`CSP`, `nosniff`, `X-Frame-Options: DENY`, `no-referrer`).
- Binds to `127.0.0.1:8787` by default.

## TUI

- Built on `ratatui` and `crossterm`.
- Interactive 3-column Kanban workspace supporting cursor navigation (`j`/`k`), column switching (`h`/`l`), status advancement (`Space`), completion toggle (`x`), and branch cycling (`b`).

## Design constraints

- **Single binary** — no runtime assets or external dynamic libraries.
- **Local-first** — no network calls at runtime; `tx self update` shells out to the installer explicitly.
- **Small dependency surface** — clap, clap_complete, clap_mangen, serde, chrono, uuid, anyhow, dirs, ratatui, crossterm, axum, tokio, toml.

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
