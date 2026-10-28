# CLI Reference

Tasklynx provides both `tasklynx` and `tx` as command aliases.

## Global Options

```bash
tx --help
tx --version
```

## Commands

| Command | Description |
| --- | --- |
| `tx init` | Initialize Tasklynx data and config |
| `tx add <title>` | Add a new task |
| `tx list` | List tasks on current branch |
| `tx complete <id>` | Mark task as done |
| `tx reopen <id>` | Reopen a completed task |
| `tx doing <id>` | Mark task as in progress |
| `tx delete <id>` | Delete a task |
| `tx edit <id>` | Edit task properties |
| `tx duplicate <id>` | Duplicate a task |
| `tx search <query>` | Search tasks by title/description/tags |
| `tx today` | Show tasks due today or overdue |
| `tx stats` | Show task statistics |
| `tx branch` | Manage task branches |
| `tx export` | Export tasks as JSON or Markdown |
| `tx web` | Start web dashboard |
| `tx board` | Open terminal Kanban board (TUI) |
| `tx config` | Manage configuration |
| `tx self update` | Update to latest version |
| `tx self uninstall` | Uninstall Tasklynx |

## Add Options

```bash
tx add "Task title" \
  --project default \
  --tag bug,ui \
  --priority normal \
  --due 2026-12-31
```

Priorities: `low`, `normal`, `high`, `urgent`

## List Options

```bash
tx list --project website
tx list --tag bug
tx list --status todo
tx list --search auth
tx list --all  # Show tasks from all branches
```

## Edit Options

```bash
tx edit <id> --title "New title"
tx edit <id> --description "Details"
tx edit <id> --priority high
tx edit <id> --project backend
tx edit <id> --due 2026-12-31
tx edit <id> --add-tag urgent
tx edit <id> --remove-tag old
```

## Branch Commands

```bash
tx branch list
tx branch create feature-x
tx branch switch feature-x
tx branch delete feature-x
```

## Export

```bash
tx export --format json
tx export --format md
```

## Web Server

```bash
tx web --host 127.0.0.1 --port 8787
```
