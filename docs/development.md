# Development

## Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- Git

## Build

```bash
git clone https://github.com/peekstore/tasklynx.git
cd tasklynx
cargo build
```

## Release Build

```bash
cargo build --release
```

## Run

```bash
cargo run -- --help
cargo run -- board
cargo run -- web
```

## Testing

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Project Structure

```
src/
├── main.rs   # CLI parsing and command handling
├── model.rs  # Data structures (Task, ProjectData, Config)
├── store.rs  # JSON persistence with atomic writes
├── tui.rs    # Ratatui terminal Kanban board
├── web.rs    # Axum web server
└── config.rs # Configuration management
assets/       # Embedded web assets and icons
```

## Release Process

Releases are automated via GitHub Actions. Push a version tag:
```bash
git tag v0.0.1-beta-1
git push origin v0.0.1-beta-1
```

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.
