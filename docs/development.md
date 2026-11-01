# Development

## Prerequisites

- [Rust](https://rustup.rs/) 1.75 or newer (stable).
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
cargo clippy --all-targets --features ai -- -D warnings
cargo test --all-targets --features ai
```

## Project layout

```
src/
├── main.rs       # CLI parsing and command dispatch
├── model.rs      # Task, ProjectData, AppConfig, dates, recurrence
├── store.rs      # JSON persistence, import/export, undo
├── config.rs     # Config load/save and path resolution
├── templates.rs  # Built-in task templates
├── duplicates.rs # Title similarity and duplicate detection
├── reminders.rs  # Due work, notifications, and iCalendar export
├── git.rs        # Repository discovery, branch mapping, scan helpers
├── tui.rs        # Ratatui terminal board
├── web.rs        # Axum server, REST API, embedded UI
└── ai.rs         # Optional AI providers and prompt handling
assets/           # Embedded web UI and icons
docs/             # MkDocs documentation
Screenshots/      # README and docs images
tests/            # CLI integration tests
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

Release builds are automated by GitHub Actions, but version bumps and tags are explicit. The old auto-version workflow has been removed.

1. Synchronize every manifest and documentation example:

```bash
./tools/sync_version.sh v0.0.1-b4
```

2. Add the matching `CHANGELOG.md` section and update `CITATION.cff`'s `date-released` if needed.
3. Run the quality gates above, then `cargo build --release --locked` and `mkdocs build --strict`.
4. Commit, then create and push an annotated tag:

```bash
git add -A
git commit -m "chore(release): bump version to v0.0.1-b4"
git tag -a v0.0.1-b4 -m "Tasklynx v0.0.1-b4"
git push origin main
git push origin v0.0.1-b4
```

The release workflow (`Auto Build & Release`) builds every target, publishes the assets, and generates checksums. `Package & Distribute` publishes package artifacts on the tag, and `Installer Test All` runs after the release completes against the newest published release.

## Getting help

Open an issue at [github.com/peekstore/tasklynx/issues](https://github.com/peekstore/tasklynx/issues) and read [CONTRIBUTING.md](https://github.com/peekstore/tasklynx/blob/main/CONTRIBUTING.md) first.

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
