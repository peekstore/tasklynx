# Changelog

## Unreleased

- Added `tx next` with priority, due-date, and blocker-aware ranking.
- Added task dependencies with `tx block` and `tx unblock`.
- Added subtasks with `tx subtask` and tree rendering with `tx tree`.
- Added `tx undo` plus recoverable deletes via `tx trash list|restore|purge`.
- Added local Git integration with `tx git status` and `tx git link`.
- Added `[git]` configuration with automatic branch mapping, Git metadata on `tx add`, and recent commits plus workspace clean/dirty state in `tx git status`.
- Added `tx git sync` for linking tasks from `Tasklynx-Id` commit trailers or task IDs in commit subjects.
- Added `tx git scan` to turn TODO/FIXME/HACK comments into deduplicated tasks, skipping vendored and generated directories.
- Added opt-in Git hooks with `tx git hook install|uninstall`, task activation, and hook-script review output.
- Added `tx git open` for task commit, PR, and issue URLs, including `--print`.
- Added `tx git link --issue` and optional issue URLs on tasks.
- Added `tx complete --commit <sha>` as a shortcut for completing and linking work.
- Added dedicated Git integration and AI setup guides.
- Added Git metadata, source locations, and blocker fields to the task schema with backward-compatible defaults.
- Added opt-in AI provider configuration for `none`, Ollama, and OpenAI-compatible endpoints.
- Added `tx ai plan` and `tx ai breakdown` with preview-by-default output and `--apply`.
- Added `tx ai triage` with deterministic offline fallback and `--include-blocked`.
- Added `tx ai standup`, `tx ai review`, and `tx ai ask` with validated report and answer schemas.
- Added AI preview/apply endpoints and an AI copilot panel to the web dashboard.
- Added TUI detail-pane AI breakdown and triage previews.
- Added `tx ai doctor` for provider reachability and model checks.
- Added strict AI response validation, task dependency mapping, context limits, and secret redaction.
- Added an optional `ai` Cargo feature so default builds remain network-free.
- Added OpenAI-compatible endpoint support for Google Gemini AI Studio.
- Added `tx remind` for overdue, today, and upcoming tasks with opt-in desktop notifications.
- Added iCalendar export with `tx export --format ics`, including recurrence rules and stable event IDs.
- Added local time tracking with `tx start`, `tx stop`, and `tx time` using the existing estimate field.
- Active timers stop automatically when a task is completed or deleted.
- Added built-in templates for `tx add --template review|bug|feature|test|chore`.
- Added saved list filters and views with `tx view save|list|show|run|delete` and `tx list --view`.
- Added bulk completion with `tx complete --done-before <date>`, `--all`, and `--force`.
- Added title similarity detection with `tx duplicates` and warnings when `tx add` finds a similar open task.

## 0.0.1-beta-4

- Added dynamic release notes generation in GitHub Actions with changelog, installation, downloads, and AI reader links.
- Streamlined distribution matrix covering core supported installation methods: Linux x64/arm64, macOS, Windows PowerShell, Android Termux ABIs, APT (.deb), npm, pipx, Chocolatey, and Cargo.
- Added AI reader (`llms.txt` and `llms-full.txt`) links and badges to README and documentation.
- Dynamic auto-version synchronization across all 30 manifests and package repositories.

## 0.0.1-beta-3

- Support for package managers (apt, pipx, chocolatey, npm, and cargo).
- Automated package checksum syncing via GitHub Actions.
- Real high-DPI screenshots generated and updated via headless Chromium automation in CI.
- Dual dark/light mode banner support for Void MkDocs theme and GitHub README.
- Built-in Void theme dark/light palette switcher and AI reader (`llms.txt`, `llms-full.txt`).

## 0.0.1-beta-2

- Android builds now use real Android ABI artifacts: `arm64-v8a`, `armeabi-v7a`, `x86_64`.
- Installer detects Termux and selects the matching `tasklynx-android-<abi>` binary.
- Reliable POSIX installer with working `--selfuninstall` (removes binary and `tx` alias).
- Windows installer validates the downloaded PE binary and supports ARM64 detection.
- `tx self update` uses `installer.sh`; `tx self uninstall` removes the `tx` alias.
- Release workflow is idempotent (removes stale drafts/releases before publishing).
- CI installers tested on Linux x64/arm64, macOS, Windows, Termux ABIs and Nix.

## 0.0.1-beta-1

- Rust rewrite.
- Ratatui terminal interface.
- Responsive embedded web dashboard.
- Branches, projects, tags, priorities and deadlines.
- JSON/Markdown export.
- Windows icon embedding.
- Cross-platform release workflow.
- Termux Android targets.
- MkDocs Void documentation.
