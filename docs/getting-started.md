# Getting Started

This guide takes you from a clean machine to a working Tasklynx setup in a few minutes. Examples target the current `0.0.1-b4` prerelease.

## 1. Install

The fastest path is the one-line installer.

**Linux, macOS, and Termux**

```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1 | iex
```

The installer downloads the correct binary, installs `tasklynx` with the `tx` alias, and adds the install directory to your `PATH`. Restart your shell (or `source` your rc file) before continuing.

For package managers, manual downloads, and building from source, see [Installation](installation.md).

## 2. Initialize

```bash
tx init
```

This creates the local data store and a default config file. It is safe to run again; existing tasks are not overwritten.

Verify the install:

```bash
tx --version
tx --help
```

## 3. Create tasks

```bash
TASK_ID=$(tx add "Implement authentication" \
  --project api \
  --tag backend,auth \
  --priority high \
  --due 2026-12-31 \
  --description "Password login, sessions, and logout" \
  --estimate 90)
```

`tx add` prints the new task UUID, so assigning it to a shell variable makes the next examples copy-pasteable. IDs are UUIDs, but every command accepts any unique prefix, so the first four to eight characters are usually enough.

Templates fill in common task fields:

```bash
tx add --template review
tx add "Review authentication" --template review --priority high
```

## 4. Move work forward

```bash
tx doing "$TASK_ID"      # mark in progress
tx start "$TASK_ID"      # start a local timer
tx stop                  # stop the running timer
tx time "$TASK_ID"       # estimate versus tracked time
tx next --explain        # show the best next task and ranking inputs
tx complete "$TASK_ID"
tx reopen "$TASK_ID"     # send back to todo
```

Completing or deleting a task stops its active timer automatically.

## 5. Plan dependencies and recurring work

```bash
API_ID=$(tx add "Draft API schema" --project api --priority high)
tx block "$TASK_ID" --by "$API_ID"
tx next                  # blocked tasks are skipped automatically
tx unblock "$TASK_ID" --by "$API_ID"

PARENT_ID=$(tx add "Ship billing" \
  --project api --due 2026-11-15 \
  --repeat monthly --repeat-until 2027-11-15)
tx subtask "$PARENT_ID" "Add invoice export"
tx tree "$PARENT_ID"
```

`--repeat` accepts `daily`, `weekly`, `monthly`, `yearly`, or `none`. `--repeat-until` requires a repeat value. Completing a recurring task creates the next occurrence with the due date advanced.

## 6. See your work

```bash
tx list            # compact list for the current branch
tx today           # due today and overdue
tx agenda --days 14 # upcoming, overdue, and recurring work
tx stats           # totals
tx board           # interactive terminal Kanban board
tx web             # dashboard at http://127.0.0.1:8787
```

Press `q` to leave the board and `Ctrl+C` to stop the web server.

## 7. Organize with branches

Branches let you keep separate streams of work in one file.

```bash
tx branch create feature-auth
tx branch switch feature-auth
tx add "Add login form" --project api

tx list            # only feature-auth tasks
tx list --all      # tasks on every branch

tx branch switch main
tx branch delete feature-auth   # must be empty first
```

With the default Git integration settings, `tx add` can create and switch to a Tasklynx branch derived from the current Git branch. See [Git Integration](git-integration.md).

## 8. Connect Git (optional)

```bash
tx git status
tx git scan --apply --project backend
GIT_TASK=$(tx add "Wire authentication" --project api)
tx git activate "$GIT_TASK"
tx git hook install
git commit -m "Wire authentication"
tx git sync
tx git link "$GIT_TASK" --commit "$(git rev-parse --short HEAD)"
tx git deactivate
```

The hook is opt-in. Tasklynx only reads local repository metadata and never pushes or rewrites history.

## 9. Enable AI providers (optional)

Official release binaries are built without the `ai` feature. Offline AI commands such as `tx ai triage --offline` still work, but provider-backed commands require a source build:

```bash
cargo install --git https://github.com/peekstore/tasklynx.git --locked --features ai
```

Then configure Ollama or an OpenAI-compatible endpoint in `config.toml` and run:

```bash
tx ai doctor
tx ai plan "Ship the billing API"
tx ai breakdown "$TASK_ID"
```

Nothing is written by `tx ai plan` or `tx ai breakdown` until you pass `--apply`. See [AI Setup](ai-setup.md).

## 10. Keep it current

```bash
tx self update
```

To remove Tasklynx later, use `tx self uninstall`, or `tx self uninstall --purge` to remove data and config as well. See the [uninstall section](installation.md#uninstall).

## Next steps

- [CLI Reference](cli.md) — every command, flag, and current example.
- [Git Integration](git-integration.md) — branch mapping, commit links, scanning, and hooks.
- [AI Setup](ai-setup.md) — providers, privacy controls, and the optional `ai` build feature.
- [Configuration](configuration.md) — port, theme, saved views, and default project.
- [Troubleshooting](troubleshooting.md) — if something does not work.

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
