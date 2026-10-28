# Tasklynx

> Developer-first local task manager — Rust, terminal UI, responsive terminal-style web dashboard.

[![CI](https://github.com/peekstore/tasklynx/actions/workflows/ci.yml/badge.svg)](https://github.com/peekstore/tasklynx/actions/workflows/ci.yml)

## Features

- `tasklynx` / `tx` CLI
- Ratatui terminal Kanban
- Local atomic JSON persistence
- Git-style task branches
- Projects, tags, priorities and deadlines
- Search, agenda and statistics
- Task editing and duplication
- Git commit/reference fields in the data model
- JSON and Markdown export
- Responsive web dashboard
- Terminal-inspired red/black theme
- Configurable colors and typography
- Embedded web assets — single binary
- Windows executable icon embedded at build time
- Linux/macOS/Windows/Android Termux release targets
- GitHub Actions CI and automatic GitHub Releases
- MkDocs Void documentation

## Quick start

```bash
curl -sL https://raw.githubusercontent.com/peekstore/tasklynx/main/install.sh | bash
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/peekstore/tasklynx/main/install.ps1 | iex
```

Then:

```bash
tx init
tx add "Fix login bug" --project website --tag bug,auth --priority high
tx list
tx board
tx web
```

## Build

```bash
cargo build --release
```

## License

MIT — see [LICENSE](LICENSE).
