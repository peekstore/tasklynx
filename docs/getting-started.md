# Getting Started

This guide takes you from a clean machine to a working Tasklynx setup in a few minutes.

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

For every install option, including Nix, manual downloads, and building from source, see [Installation](installation.md).

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
tx add "Implement authentication" \
  --project api \
  --tag backend,auth \
  --priority high \
  --due 2026-12-31
```

`tx add` prints the new task ID. IDs are UUIDs, but every command accepts any unique prefix, so the first four to eight characters are usually enough.

## 4. Move work forward

```bash
tx doing <id>      # mark in progress
tx complete <id>   # mark done
tx reopen <id>     # send back to todo
```

## 5. See your work

```bash
tx list            # compact list for the current branch
tx today           # due today and overdue
tx stats           # totals
tx board           # interactive terminal Kanban board
tx web             # dashboard at http://127.0.0.1:8787
```

Press `q` to leave the board and `Ctrl+C` to stop the web server.

## 6. Organize with branches

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

## 7. Keep it current

```bash
tx self update
```

To remove Tasklynx later, use `tx self uninstall`, or see the [uninstall section](installation.md#uninstall).

## Next steps

- [CLI Reference](cli.md) — every command and flag.
- [Configuration](configuration.md) — port, theme, and default project.
- [Troubleshooting](troubleshooting.md) — if something does not work.

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
