# Development

## Prerequisites

- [Rust](https://rustup.rs/) 1.70 or newer (stable).
- Git.
- `clippy` and `rustfmt` (`rustup component add clippy rustfmt`).
- Optional: Android NDK r25c for Android cross-builds.

## Build and run

```bash
git clone https://github.com/peekstore/tasklynx.git
cd tasklynx

cargo build
cargo run -- --help
cargo run -- list
cargo run -- board
cargo run -- web
```

Create an isolated store while developing:

```bash
TASKLYNX_HOME=/tmp/tasklynx-dev cargo run -- add "Test task"
```

## Quality gates

Run the same checks as CI before opening a pull request:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

## Project layout

```
src/
├── main.rs    # CLI parsing and command dispatch
├── model.rs   # Task, ProjectData, AppConfig, dates
├── store.rs   # JSON persistence and export
├── tui.rs     # Ratatui terminal board
├── web.rs     # Axum server and REST API
└── config.rs  # Config load/save
assets/        # Embedded web UI and icons
docs/          # MkDocs documentation
Screenshots/   # README and docs images
```

## Android builds

Cross-compile from a desktop checkout:

```bash
./build-android-termux.sh
```

The script configures the NDK toolchain and produces ABI-named artifacts:

- `tasklynx-android-arm64-v8a`
- `tasklynx-android-armeabi-v7a`
- `tasklynx-android-x86_64`

You can also build natively inside Termux with `pkg install rust clang && cargo build --release`.

## Installer changes

`installer.sh` and `installer.ps1` are tested in CI on Linux (x64 and arm64), macOS, Windows, and against the Android ABIs. When changing them:

```bash
sh -n installer.sh          # syntax check
DRY_RUN=1 sh installer.sh   # show the resolved artifact
```

## Documentation

Docs are built with MkDocs and the `mkdocs-void` theme:

```bash
pip install mkdocs mkdocs-void
mkdocs serve
```

Update the relevant page in [`docs/`](https://github.com/peekstore/tasklynx/tree/main/docs) and the site navigation in `mkdocs.yml`.

## Release process

Releases are automated by GitHub Actions.

1. Update the version in `Cargo.toml`, `Cargo.lock`, `.version`, the README, and the docs.
2. Add a `CHANGELOG.md` entry.
3. Commit, then create and push an annotated tag:

```bash
git tag -a v0.0.1-beta-2 -m "Tasklynx v0.0.1-beta-2"
git push origin v0.0.1-beta-2
```

The release workflow builds every target, publishes the assets, and generates checksums. Installer tests run afterward against the published release.

## Getting help

Open an issue at [github.com/peekstore/tasklynx/issues](https://github.com/peekstore/tasklynx/issues) and read [CONTRIBUTING.md](https://github.com/peekstore/tasklynx/blob/main/CONTRIBUTING.md) first.

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
