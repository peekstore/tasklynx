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
| `tx view <save\|list\|show\|run\|delete>` | Manage saved filters and views |
| `tx doing <id>` | Mark a task in progress |
| `tx start <id>` | Start a local timer and mark a task in progress |
| `tx stop` | Stop the running timer |
| `tx time [id]` | Show tracked time versus estimates |
| `tx complete <id> [--commit <sha>]` | Mark a task done and optionally link a commit |
| `tx reopen <id>` | Reopen a completed task |
| `tx next` | Show the best next task to work on |
| `tx delete <id>` | Move a task to trash |
| `tx undo` | Undo the most recent CLI mutation |
| `tx trash list` | List trashed tasks |
| `tx trash restore <id>` | Restore a trashed task |
| `tx trash purge` | Permanently remove trashed tasks |
| `tx edit <id>` | Edit task fields |
| `tx duplicate <id>` | Duplicate a task |
| `tx duplicates [--threshold N]` | Find tasks with similar titles |
| `tx search <query>` | Search titles, descriptions, and tags |
| `tx today` | Show tasks due today or overdue |
| `tx agenda [--days N]` | Show upcoming, overdue, and recurring work |
| `tx remind [--days N] [--notify]` | Show due and overdue work, optionally with a desktop notification |
| `tx block <id> --by <blocker>` | Mark a task as blocked by another |
| `tx unblock <id> [--by <blocker>]` | Remove one or all blockers |
| `tx subtask <parent> <title>` | Create a subtask under a parent |
| `tx tree [id]` | Show parent/child task trees |
| `tx stats` | Show task and branch totals |
| `tx branch` | Manage branches |
| `tx git status` | Show Git and Tasklynx branch context |
| `tx git link <id>` | Link a task to a commit, PR, or issue |
| `tx git sync` | Link tasks from commit trailers |
| `tx git scan` | Find TODO/FIXME comments in tracked files |
| `tx git open <id>` | Open a task commit, PR, or issue in the browser |
| `tx git hook` | Install or remove the Tasklynx Git hook |
| `tx git activate <id>` | Mark a task active for commit trailers |
| `tx git deactivate` | Clear the active commit task |
| `tx ai plan <goal>` | Draft a task plan with an AI provider |
| `tx ai breakdown <id>` | Draft subtasks for an existing task |
| `tx ai triage` | Rank the next tasks; offline fallback when no provider is configured |
| `tx ai standup` | Summarize recent work for a daily standup |
| `tx ai review [branch]` | Summarize open, blocked, and completed branch work |
| `tx ai ask <question>` | Ask a grounded question about task context |
| `tx ai doctor` | Check AI provider and model configuration |
| `tx export` | Export tasks as JSON, Markdown, or iCalendar |
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
| `--estimate <minutes>` | — | none | Estimated effort in minutes |
| `--template <name>` | — | none | Apply a built-in task template |

`tx add` prints the full task UUID.

### Templates

```bash
tx add --template review
tx add "Review authentication" --template review
tx add --template bug --priority low
```

Available templates are `review`, `bug`, `feature`, `test`, and `chore`. A template supplies a default title, priority, tags, description, and estimate. Explicit flags override template values, and explicit `--tag` values are merged with template tags. `--template review` can be used without a title; otherwise a title is required.

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
| `--view <name>` | Apply a saved view |
| `--save <name>` | Save the current filter as a view |

Output is one task per line:

```
3f9c1a2b  todo    high    api          Implement authentication
```

Columns are: short ID, status, priority, project, title.

### Saved views

```bash
tx view save api --project api --tag backend
tx view list
tx view show api
tx view run api
tx list --view api
tx list --save todo --status todo
tx view delete api
```

Saved views live in `config.toml` under `[views.<name>]`. `tx list --view <name>` runs a saved filter, and flags passed alongside it override the saved values. `tx list --save <name>` saves the current filter. `tx view run <name>` is equivalent to `tx list --view <name>`, while `tx view show <name>` prints the saved filter definition.

## Editing tasks

```bash
tx edit <id> --title "New title"
tx edit <id> --description "More detail"
tx edit <id> --priority high
tx edit <id> --project backend
tx edit <id> --due 2026-12-31
tx edit <id> --estimate 90
tx edit <id> --add-tag urgent
tx edit <id> --remove-tag stale
```

`--add-tag` and `--remove-tag` can be repeated or passed comma-separated for `--add-tag`.

## Status changes

```bash
tx doing <id>
tx complete <id>
tx complete <id> --commit <sha>
tx reopen <id>
```

Completing a task records `completed_at`; reopening clears it and returns the task to `todo`. Completing or deleting a task automatically stops its running timer. `tx complete --commit` links the supplied commit as part of the same atomic update.

Bulk completion closes every open task due on or before a date:

```bash
tx complete --done-before 2026-10-31
tx complete --done-before 2026-10-31 --all
tx complete --done-before 2026-10-31 --force
```

Bulk completion uses the current branch unless `--all` is passed. Parents with open subtasks are skipped unless `--force` is used. The whole batch is saved as one undo step.

## Time tracking

```bash
tx add "Write migration" --estimate 45
tx start <id>
tx stop
tx time
tx time <id>
```

`tx start` records a local session and moves the task to `doing`. Only one timer can run at a time; `tx stop` closes it. Completion and deletion also close an active session.

`tx time` lists tasks with tracked sessions and their estimate versus actual time. `tx time <id>` shows the estimate, actual total, and each session's start/end timestamps. Running sessions are marked `active` and count toward the total. Estimates and sessions stay local in `tasks.json`.

## Searching, today, and stats

```bash
tx search auth
tx today
tx stats
```

- `search` matches title, description, and tags (case-insensitive).
- `today` prints `OVERDUE` or `TODAY` followed by the short ID and title.
- `stats` prints `tasks=… done=… open=… overdue=… branches=… projects=…`.

## Duplicate detection

```bash
tx duplicates
tx duplicates --threshold 0.8
tx duplicates --all
```

`tx duplicates` compares open task titles and prints similar pairs with a score. The default threshold is `0.72`; use `--all` to compare across branches. `tx add` also prints a warning when a new title looks similar to an existing open task, but still creates the task.

## Reminders and calendar

```bash
tx remind
tx remind --days 7
tx remind --notify
tx export --format ics > tasklynx.ics
```

`tx remind` lists open tasks that are overdue or due within the next N days (default 3). Rows are labeled `OVERDUE`, `TODAY`, or `UPCOMING` and show the short ID, due date, priority, and title. Completed tasks and tasks without a due date are skipped.

`--notify` sends one desktop notification after printing the list. It is opt-in and daemon-free: Linux uses `notify-send`, macOS uses `osascript`, and other platforms report that notifications are unsupported. Use `tx remind` without `--notify` if a notification helper is unavailable.

`tx export --format ics` writes a standards-based iCalendar file for tasks with due dates. Each task becomes an all-day event, recurring tasks include an `RRULE`, and completed tasks are marked `STATUS:COMPLETED`.

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
tx git link <id> --issue https://github.com/owner/repo/issues/7
tx git open <id>
tx git open <id> --print
```

`tx git status` reports the repository root, Git branch, mapped Tasklynx branch, HEAD, remote, workspace clean/dirty state, recent commits, and open tasks on the current Tasklynx branch. `tx git link` stores the commit, Git branch, and remote URL on the task. When `--commit` is omitted, `HEAD` is used. `--pr` stores a pull request or merge request URL, and `--issue` stores an issue URL.

With `[git] auto_branch = true` (the default), `tx add` maps the current Git branch to a Tasklynx branch by stripping `branch_prefix` (`tx/auth` becomes `auth`) and switches to it. With `link_commits = true`, the new task also records the current Git branch, remote, and `HEAD` commit. Set either option to `false` to keep manual branch and commit linking.

`tx git open` opens the task's PR URL when present, then its issue URL, and otherwise its commit URL. Use `--print` to print the URL without launching a browser.

### Scanning code comments

```bash
tx git scan
tx git scan --apply
tx git scan --apply --project backend --limit 50
```

`tx git scan` searches tracked files for `TODO`, `FIXME`, and `HACK` comments. Preview mode prints file paths and line numbers without changing the store. `--apply` creates tasks with the source path, line number, Git commit, and branch attached. Already-scanned locations are skipped, so rerunning the command is safe. Vendored and generated directories such as `vendor/`, `node_modules/`, `target/`, `build/`, and `dist/` are skipped.

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
- `tx git hook install` installs an opt-in `prepare-commit-msg` hook that appends a `Tasklynx-Id: <uuid>` trailer when a task is active. The hook script is printed for review.
- `tx git sync` scans recent commits for that trailer or a task ID in the commit subject and links matching tasks to the commit.
- `tx git deactivate` clears the active task.
- `tx git hook uninstall` removes only a hook installed by Tasklynx.

Tasklynx refuses to overwrite or remove hooks it did not install. Git integration only reads local repository metadata and updates local task data. It does not push, rewrite history, or contact a remote.

## AI planning

AI commands are opt-in. The default build keeps networking disabled and the default provider is `none`:

```toml
[ai]
provider = "none"
model = "llama3.1"
endpoint = "http://127.0.0.1:11434"
max_context_tasks = 50
include_descriptions = true
redact = true
```

To use Ollama or an OpenAI-compatible endpoint, build with the `ai` feature and select a provider:

```bash
cargo build --features ai
tx ai doctor
```

Supported providers:

| Provider | Notes |
| --- | --- |
| `none` | Offline mode; AI commands explain how to configure a provider |
| `ollama` | Uses `POST /api/generate` and `GET /api/tags` on the configured endpoint |
| `openai-compatible` | Uses `<endpoint>/chat/completions` and `<endpoint>/models`; include the API version path in the endpoint |

Hosted endpoints should read the API key from `TASKLYNX_AI_API_KEY`; keys are never written to `config.toml`.

Google Gemini through AI Studio uses the OpenAI-compatible endpoint:

```toml
[ai]
provider = "openai-compatible"
model = "gemini-2.5-flash"
endpoint = "https://generativelanguage.googleapis.com/v1beta/openai"
```

```bash
export TASKLYNX_AI_API_KEY="your-ai-studio-key"
tx ai doctor
```

### Plan and breakdown

```bash
tx ai plan "Ship the billing API"
tx ai plan "Ship the billing API" --project api --limit 8
tx ai plan "Ship the billing API" --apply
tx ai breakdown <id>
tx ai breakdown <id> --apply --limit 5
```

- `tx ai plan` drafts new tasks from a goal.
- `tx ai breakdown` drafts subtasks under an existing task.
- Both commands preview by default. Nothing is written until `--apply` is passed.
- `--limit` caps the number of returned tasks.
- `--no-description` omits task descriptions from the prompt context.
- `depends_on` entries in the AI response become real blockers between new tasks.
- Existing open tasks on the current branch are included as context, up to `ai.max_context_tasks`.
- With `redact = true`, common API keys, tokens, passwords, and private keys are redacted before the prompt is sent.

### Triage, standup, review, and ask

```bash
tx ai triage
tx ai triage --limit 3 --project api
tx ai triage --include-blocked
tx ai triage --offline
tx ai standup --days 1
tx ai review feature-auth --days 14
tx ai ask "What is blocking the release?"
```

- `tx ai triage` ranks open work. With no provider configured, or with `--offline`, it uses the same deterministic ordering as `tx next`.
- `--include-blocked` lets triage consider blocked tasks and explains what blocks them.
- `tx ai standup` summarizes recent completed work plus current open and blocked work.
- `tx ai review [branch]` summarizes a branch, defaulting to the current branch.
- `tx ai ask "<question>"` answers from the selected open-task context and says when the answer is not present.
- `--days`, `--project`, `--limit`, and `--no-description` control context and output size.
- AI commands print the selected provider and model to stderr.
- `TASKLYNX_AI_LOG=1` logs prompt/response byte sizes and provider latency to stderr; prompt contents are never logged.

AI output is treated as untrusted: titles, priorities, dates, tags, dependency indexes, task IDs, report structure, and answer shape are validated before use. Tasklynx never auto-completes, deletes, or commits anything based on AI output.

### Doctor

```bash
tx ai doctor
```

Prints the configured provider, endpoint, and model, then checks connectivity. Ollama checks installed models via `/api/tags`; OpenAI-compatible providers check `/v1/models`. The command does not send task data.

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
tx export --format ics > tasklynx.ics
tx export > tasks.json
```

`--format` defaults to `json`. Supported values are `json`, `md`, and `ics`; any other value is rejected. ICS export includes every task with a due date, including completed tasks.

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
| `POST` | `/api/ai/triage` | Preview AI or offline task triage |
| `POST` | `/api/ai/plan` | Preview an AI plan |
| `POST` | `/api/ai/apply-plan` | Apply a previewed AI plan |
| `POST` | `/api/ai/breakdown` | Preview an AI subtask breakdown |
| `POST` | `/api/ai/apply-breakdown` | Apply a previewed breakdown |
| `POST` | `/api/ai/standup` | Generate a standup report |
| `POST` | `/api/ai/review` | Generate a branch review |
| `POST` | `/api/ai/ask` | Ask a question about task context |

The dashboard includes an **AI copilot** panel for triage, plan/breakdown previews with apply controls, standups, branch reviews, and grounded questions. Apply calls use the previewed draft rather than regenerating it.

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
- `a` — preview an AI breakdown for the selected task.
- `t` — preview AI or offline triage.
- `c` — clear the AI detail panel.
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
