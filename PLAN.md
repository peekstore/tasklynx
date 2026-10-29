# Tasklynx Improvement Plan

Living checklist from the repository audit. Items are grouped by priority and updated as work lands.

Legend: `[ ]` pending · `[~]` in progress · `[x]` done

## Phase 1 — Correctness and credibility

- [x] **Fix `tx self update` on Windows.** `src/main.rs` hardcodes `sh` + `installer.sh`; Windows has no `sh` and the error is swallowed. Branch on the platform: PowerShell + `installer.ps1` on Windows, `sh` elsewhere. Propagate failures.
- [x] **Add tests.** Added unit tests in `src/model.rs`, `src/store.rs`, and `src/config.rs`, and CLI integration tests in `tests/cli.rs`.
  - Unit tests: `store::find` prefix matching, ambiguous prefix rejection, `export_json`/`export_md`, `import_json`/`import_md`, `local_due_date`, `overdue`, config defaults/partial config.
  - CLI integration tests in `tests/cli.rs` using `CARGO_BIN_EXE_tasklynx` and an isolated `TASKLYNX_HOME`/`XDG_CONFIG_HOME`.
  - Cover add/list/doing/complete/reopen/delete/edit/duplicate/search/today/stats/branch/export/import/completions/man/config, invalid dates, and ambiguous IDs.
- [x] **Add `flake.nix`.** Added Nix flake and configured CI to run `nix run . -- --version` without masking.
- [x] **Fix "archive" wording.** Updated README and installation documentation to clarify binary and archive downloads.
- [x] **Make `AppConfig` partial-config safe.** Added `#[serde(default)]` and `validate()` checks for host and port.

## Phase 2 — Robustness

- [x] **Uninstall consistency.** Aligned `tx self uninstall`, `installer.sh --selfuninstall`, and `installer.ps1 --selfuninstall`. Preserves config/data by default, and supports `--purge` for full removal.
- [x] **Windows self-delete.** Spawn delayed helper (`cmd /C timeout ... & del`) on Windows when uninstalling.
- [x] **Single version source.** Added integration test and CI check verifying `.version` matches `Cargo.toml`.
- [x] **Import / backup / restore.** Added `tx import` supporting both JSON and Markdown formats with merge and `--replace` options.
- [x] **Shell completions and man page.** Added `clap_complete` (`tx completions <shell>`) and `clap_mangen` (`tx man`).
- [x] **Cross-process safety.** Added `store::StoreLock` advisory file lock with timeout and stale lock recovery for concurrent safety.
- [x] **Web hardening.** Added security headers (CSP, nosniff, frame deny, referrer policy), 2MB request body limit, and `DELETE /api/tasks/{id}`.
- [x] **Durability.** Added `file.sync_all()` (`fsync`) to atomic store saves.
- [x] **Interactive TUI.** Upgraded `src/tui.rs` into an active 3-column Kanban board with keyboard navigation (`h`/`l` columns, `j`/`k` task selection, `Space` advance, `x` toggle done, `b` branch cycle, `q` quit).

## Phase 3 — Repository hygiene

- [x] **GitHub templates.** Added `.github/ISSUE_TEMPLATE/bug_report.md`, `feature_request.md`, `config.yml`, and `PULL_REQUEST_TEMPLATE.md`.
- [x] **Automation.** Added `.github/dependabot.yml` for cargo and github-actions updates.
- [x] **Community files.** Added `SUPPORT.md` and `CITATION.cff`.
- [x] **Toolchain pinning.** Added `rust-toolchain.toml` and MSRV (Rust 1.75.0) CI job in `.github/workflows/ci.yml`.
- [x] **Cargo metadata.** Enriched `Cargo.toml` with `authors`, `homepage`, `documentation`, `readme`, `keywords`, `categories`, `rust-version`, and `exclude`.
- [x] **Clean stray files.** Removed redundant root scripts, moved `build-mixos.sh` to `tools/`, cleaned `.gitkeep`/`.gitmodules`.
- [x] **CI matrix.** Multi-OS matrix testing on `ubuntu-latest`, `macos-latest`, and `windows-latest` on every PR and push.

## Phase 4 — Distribution

- [x] **Release archives.** Updated `.github/workflows/release.yml` to package `.tar.gz` for Unix and `.zip` for Windows alongside raw binaries.
- [x] **Update manifest.** Generates `latest.json` release manifest with tag, version, commit, and timestamp.
- [x] **Fix `SHA256SUMS.txt`.** Excluded individual `.sha256` files from aggregate checksum file.
- [x] **Signing guidance.** Documented macOS Gatekeeper (`xattr -d com.apple.quarantine`) and Windows SmartScreen unblock instructions in docs.
- [x] **Packaging & Distribution.** Populated `packaging/` and distribution pipelines:
  - **Debian / APT**: `packaging/debian/control` and `build_deb.sh` producing `.deb` packages.
  - **Arch Linux / Pacman**: `packaging/pacman/PKGBUILD` and `.SRCINFO`.
  - **Homebrew**: `Formula/tasklynx.rb` and `packaging/homebrew/tasklynx.rb`.
  - **Mise / asdf**: `packaging/mise/` plugin scripts (`list-all`, `download`, `install`).
  - **Pipx**: `packaging/pipx/` PEP-621 / `pyproject.toml` Python package with binary runner.
  - **Scoop**: `packaging/scoop/tasklynx.json` and `bucket/tasklynx.json`.
  - **Winget**: `packaging/winget/manifests/p/peekstore/tasklynx/0.0.1-beta-2/` manifests.
  - **Chocolatey**: `packaging/chocolatey/tasklynx.nuspec` and install/uninstall scripts with `CHOCO_SECRET_KEY` automation.
  - **NPM**: `packaging/npm/` package with multi-platform binary fetcher and `NPM_SCERET` publication workflow.
  - **GitHub Actions workflows**: `.github/workflows/package-publish.yml` testing and publishing packages across ecosystems.
  - **Real screenshots automation**: `.github/workflows/screenshots.yml` and `tools/capture_real_screenshots.js` supporting Chromium, Puppeteer, Puppeteer-Core, Playwright, Playwright-Core, and Chromedriver.

## Verification

Every phase passes:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
mkdocs build
```
