# CLI Reference

Tasklynx installs two command names that behave identically:

- `tasklynx`
- `tx` (short alias)

Examples below use `tx`. Run `tx <command> --help` for the authoritative flag list.

## Global options

```bash
tx --help
tx --version
```

Running `tx` with no subcommand opens the terminal board.

## Commands

| Command | Description |
| --- | --- |
| `tx init` | Initialize the data store and config |
| `tx add <title>` | Add a task |
| `tx list` | List tasks on the current branch |
| `tx doing <id>` | Mark a task in progress |
| `tx complete <id>` | Mark a task done |
| `tx reopen <id>` | Reopen a completed task |
| `tx next` | Show the best next task to work on |
| `tx delete <id>` | Move a task to trash |
| `tx undo` | Undo the most recent CLI mutation |
| `tx trash list` | List trashed tasks |
| `tx trash restore <id>` | Restore a trashed task |
| `tx trash purge` | Permanently remove trashed tasks |
| `tx edit <id>` | Edit task fields |
| `tx duplicate <id>` | Duplicate a task |
| `tx search <query>` | Search titles, descriptions, and tags |
| `tx today` | Show tasks due today or overdue |
| `tx agenda [--days N]` | Show upcoming, overdue, and recurring work |
| `tx block <id> --by <blocker>` | Mark a task as blocked by another |
| `tx unblock <id> [--by <blocker>]` | Remove one or all blockers |
| `tx subtask <parent> <title>` | Create a subtask under a parent |
| `tx tree [id]` | Show parent/child task trees |
| `tx stats` | Show task and branch totals |
| `tx branch` | Manage branches |
| `tx git status` | Show Git and Tasklynx branch context |
| `tx git link <id>` | Link a task to a commit or PR |
| `tx git sync` | Link tasks from commit trailers |
| `tx git scan` | Find TODO/FIXME comments in tracked files |
| `tx git open <id>` | Open a task commit or PR in the browser |
| `tx git hook` | Install or remove the Tasklynx Git hook |
| `tx git activate <id>` | Mark a task active for commit trailers |
| `tx git deactivate` | Clear the active commit task |
| `tx export` | Export tasks as JSON or Markdown |
| `tx web` | Start the web dashboard |
| `tx board` | Open the terminal Kanban board |
| `tx config` | Manage configuration |
| `tx self update` | Update to the latest release |
| `tx self uninstall` | Remove the installation |

## Task IDs

Every task has a UUID. Any unique prefix is accepted wherever an `<id>` is expected:

```bash
tx complete 3f9c
```

If the prefix is ambiguous or missing, the command exits with an error.

## Adding tasks

```bash
tx add "Task title" \
  --project default \
  --tag bug,ui \
  --priority normal \
  --due 2026-12-31
```

| Flag | Short | Default | Notes |
| --- | --- | --- | --- |
| `--project <name>` | `-p` | `default` | Project name |
| `--tag <tags>` | `-t` | none | Comma-separated list |
| `--priority <level>` | `-P` | `normal` | `low`, `normal`, `high`, `urgent` |
| `--due <YYYY-MM-DD>` | — | none | Due at 23:59:59 local time |

`tx add` prints the full task UUID.

## Listing and filtering

```bash
tx list
tx list --project api
tx list --tag backend
tx list --status todo
tx list --status doing
tx list --status done
tx list --search auth
tx list --all
```

| Flag | Description |
| --- | --- |
| `--project <name>` | Only tasks in a project |
| `--tag <tag>` | Only tasks with a tag |
| `--status <status>` | `todo`, `doing`, or `done` |
| `--search <text>` | Case-insensitive title/description match |
| `--all` | Include tasks from every branch |

Output is one task per line:

```
3f9c1a2b  todo    high    api          Implement authentication
```

Columns are: short ID, status, priority, project, title.

## Editing tasks

```bash
tx edit <id> --title "New title"
tx edit <id> --description "More detail"
tx edit <id> --priority high
tx edit <id> --project backend
tx edit <id> --due 2026-12-31
tx edit <id> --add-tag urgent
tx edit <id> --remove-tag stale
```

`--add-tag` and `--remove-tag` can be repeated or passed comma-separated for `--add-tag`.

## Status changes

```bash
tx doing <id>
tx complete <id>
tx reopen <id>
```

Completing a task records `completed_at`; reopening clears it and returns the task to `todo`.

## Searching, today, and stats

```bash
tx search auth
tx today
tx stats
```

- `search` matches title, description, and tags (case-insensitive).
- `today` prints `OVERDUE` or `TODAY` followed by the short ID and title.
- `stats` prints `tasks=… done=… open=… overdue=… branches=… projects=…`.

## Choosing the next task

```bash
tx next
tx next --project api
tx next --explain
tx next --include-blocked
tx next --all
```

`tx next` ranks open tasks by priority, due date, and status. Blocked tasks are skipped unless `--include-blocked` is passed. `--all` includes every branch, and `--explain` prints the ranking inputs.

## Dependencies and subtasks

```bash
tx block <id> --by <blocker-id>
tx unblock <id> --by <blocker-id>
tx unblock <id>
tx subtask <parent-id> "Write migration"
tx tree
tx tree <parent-id>
```

A task is blocked while any task in `blocked_by` is still open. Completing a parent with open subtasks requires `--force`:

```bash
tx complete <parent-id> --force
```

Subtasks inherit the parent's project, branch, and priority unless you pass `--priority` or `--due`.

## Undo and trash

```bash
tx delete <id>
tx undo
tx trash list
tx trash restore <id>
tx trash purge
```

Deleting a task moves it to trash instead of removing it immediately. `tx undo` restores the store to its state before the most recent successful mutation, including CLI commands, web edits, and board sessions. The trash can be emptied permanently with `tx trash purge`.

## Git integration

```bash
tx git status
tx git link <id>
tx git link <id> --commit <sha>
tx git link <id> --pr https://github.com/owner/repo/pull/42
tx git open <id>
tx git open <id> --print
```

`tx git status` reports the repository root, Git branch, HEAD, remote, and open tasks on the current Tasklynx branch. `tx git link` stores the commit, Git branch, and remote URL on the task. When `--commit` is omitted, `HEAD` is used. `--pr` stores a pull request or merge request URL.

`tx git open` opens the task's PR URL when present, otherwise its commit URL. Use `--print` to print the URL without launching a browser.

### Scanning code comments

```bash
tx git scan
tx git scan --apply
tx git scan --apply --project backend --limit 50
```

`tx git scan` searches tracked files for `TODO`, `FIXME`, and `HACK` comments. Preview mode prints file paths and line numbers without changing the store. `--apply` creates tasks with the source path, line number, Git commit, and branch attached. Already-scanned locations are skipped, so rerunning the command is safe.

### Commit trailers and hooks

```bash
tx git activate <id>
tx git hook install
git commit -m "Implement authentication"
tx git sync
git commit -m "More work"
tx git deactivate
tx git hook uninstall
```

- `tx git activate` stores the task ID in the repository's local Git config.
- `tx git hook install` installs an opt-in `prepare-commit-msg` hook that appends a `Tasklynx-Id: <uuid>` trailer when a task is active.
- `tx git sync` scans recent commits for that trailer and links matching tasks to the commit.
- `tx git deactivate` clears the active task.
- `tx git hook uninstall` removes only a hook installed by Tasklynx.

Tasklynx refuses to overwrite or remove hooks it did not install. Git integration only reads local repository metadata and updates local task data. It does not push, rewrite history, or contact a remote.

## Branches

```bash
tx branch list
tx branch create feature-auth
tx branch switch feature-auth
tx branch delete feature-auth
```

A branch can only be deleted when it is not current and has no tasks. The current branch is marked with `*` in `tx branch list`.

## Exporting

```bash
tx export --format json
tx export --format md
tx export > tasks.json
```

`--format` defaults to `json`. Any value other than `md` produces JSON.

## Importing / Backup / Restore

```bash
tx import backup.json
tx import tasks.md
tx import --replace backup.json
cat tasks.json | tx import
```

- Supports JSON (either a full `ProjectData` export or an array of `Task` objects) and Markdown checklists.
- Merges with existing tasks by default (updating matching IDs and appending new ones).
- Pass `--replace` to overwrite all existing tasks.

## Web dashboard

```bash
tx web
tx web --host 127.0.0.1 --port 8787
```

Defaults come from `config.toml`. The server exposes:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Dashboard page |
| `GET` | `/favicon.svg` | Icon |
| `GET` | `/api/tasks` | Full project data as JSON |
| `POST` | `/api/tasks` | Create a task from a JSON body |
| `PUT` | `/api/tasks/{id}` | Replace a task |
| `DELETE` | `/api/tasks/{id}` | Delete a task |

The server automatically sets security headers (`Content-Security-Policy`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`) and enforces a 2MB request body limit. There is no authentication; keep the bind address on `127.0.0.1` unless behind a secure reverse proxy.

## Terminal board

```bash
tx board
```

Opens an interactive terminal Kanban board:
- `h` / `l` or `←` / `→` — switch between columns (Todo, Doing, Done).
- `j` / `k` or `↓` / `↑` — navigate tasks within the active column.
- `Space` / `Enter` — advance task status (`Todo` → `Doing` → `Done`).
- `x` — toggle complete / reopen.
- `b` — cycle through active branches.
- `q` / `Esc` — save changes and quit.

## Shell completions

```bash
tx completions bash >> ~/.bashrc
tx completions zsh > "${fpath[1]}/_tasklynx"
tx completions fish > ~/.config/fish/completions/tasklynx.fish
tx completions powershell >> $PROFILE
```

Generates completion scripts for `bash`, `elvish`, `fish`, `powershell`, or `zsh`.

## Man page

```bash
tx man | man -l -
tx man > /usr/local/share/man/man1/tasklynx.1
```

Renders a standard roff-formatted UNIX manual page for Tasklynx.

## Configuration commands

```bash
tx config init
tx config show
```

`init` writes the config file and prints its path. `show` prints the active configuration as TOML.

## Self-management

```bash
tx self update
tx self uninstall
```

`update` re-runs the official installer. `uninstall` removes the binary and the `tx` alias.

## Exit codes

- `0` — success.
- non-zero — invalid arguments, task not found, branch errors, I/O failures, or invalid dates.

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
