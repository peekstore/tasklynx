# Git Integration

Tasklynx reads local Git metadata and attaches repository context to tasks. It never pushes, rewrites history, or contacts a remote.

## Configuration

Add a `[git]` table to `config.toml`:

```toml
[git]
auto_branch = true
branch_prefix = "tx/"
link_commits = true
hooks = false
```

| Key | Default | Purpose |
| --- | --- | --- |
| `auto_branch` | `true` | Map `tx add` to the current Git branch |
| `branch_prefix` | `"tx/"` | Git branch prefix stripped during mapping |
| `link_commits` | `true` | Store the current Git commit when creating a task |
| `hooks` | `false` | Reserved for hook automation; hooks are installed explicitly |

With `auto_branch` enabled, Git branch `tx/auth` maps to Tasklynx branch `auth`. Tasklynx creates the branch if needed and switches to it before saving the task. Unprefixed branches such as `main` map unchanged, and a detached `HEAD` does not trigger a branch switch.

`link_commits` stores the current branch, remote, and `HEAD` commit on new tasks. Pass `auto_branch = false` or `link_commits = false` to keep manual branch or commit linking.

## Status and branch context

```bash
tx git status
```

The output includes the repository root, Git branch, mapped Tasklynx branch, `HEAD`, remote, workspace clean/dirty state, recent commits, and open tasks on the current Tasklynx branch. Dirty state reports changed and untracked file counts. It works inside and outside a Git repository.

## Linking commits, pull requests, and issues

```bash
tx git link <id> --commit <sha>
tx git link <id> --pr https://github.com/owner/repo/pull/42
tx git link <id> --issue https://github.com/owner/repo/issues/7
tx complete <id> --commit <sha>
```

`tx git link` uses `HEAD` when `--commit` is omitted. PR and issue URLs must be HTTP(S). `tx complete --commit` links the commit in the same atomic update that completes the task.

Open the stored link with:

```bash
tx git open <id>
tx git open <id> --print
```

`--print` writes the URL without launching a browser. Opening prefers the PR URL, then the issue URL, then the commit URL.

## Commit trailers and subject matching

Install the optional hook:

```bash
tx git activate <id>
tx git hook install
git commit -m "Implement authentication"
tx git sync
tx git sync --limit 25
git commit -m "More work <short-id>"
tx git deactivate
tx git hook uninstall
```

The `prepare-commit-msg` hook appends a `Tasklynx-Id` trailer while a task is active. `tx git hook install` prints the script for review, and reinstalling refreshes only a hook managed by Tasklynx. Tasklynx refuses to overwrite or remove unrelated hooks.

`tx git sync` scans the most recent 100 commits by default and links tasks by:

- A `Tasklynx-Id: <id>` trailer.
- A full task UUID or unique ID prefix in the commit subject.

Pass `--limit N` to change the scan window. Sync accepts full IDs and short prefixes. It does not rewrite commits.

## Scanning code comments

```bash
tx git scan
tx git scan --apply
tx git scan --apply --project backend --limit 50
```

`tx git scan` searches tracked files for `TODO`, `FIXME`, and `HACK` markers. Preview mode never writes to the task store. `--apply` creates tasks with the source path, line number, commit, and branch.

Scanned locations are deduplicated, so rerunning the command is safe. Binary files and vendored or generated directories are skipped, including `vendor/`, `node_modules/`, `target/`, `build/`, `dist/`, `out/`, `coverage/`, `third_party/`, and `.venv/`.

## Safety

- Git commands use argument arrays, never shell strings.
- Linking and scanning only update local Tasklynx data.
- No API tokens or remote credentials are required.
- Hook installation is explicit and reversible.
- Tests use temporary Git repositories rather than the host repository.
