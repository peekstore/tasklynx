# Frequently Asked Questions

## Is my data stored locally?

Yes. Tasklynx writes a single `tasks.json` file on your machine. There is no account, sync service, or telemetry. See [Configuration](configuration.md#data-storage) for the exact path.

## What is `tx`?

`tx` is a short alias for the `tasklynx` binary. Both names run the same program.

## Does it work offline?

Yes. Tasks are local, and the web dashboard is served from the binary. Only `tx self update`, the installers, and AI commands with a configured provider use the network. Offline triage (`tx ai triage --offline`) sends nothing.

## Can I use Tasklynx on multiple machines?

Copy `tasks.json` between machines, or point `TASKLYNX_HOME` at a synchronized directory. There is no automatic merge; avoid editing the same store concurrently on two machines.

## How do I update?

```bash
tx self update
```

This re-runs the official installer. Pin a version with `TASKLYNX_VERSION` if needed.

## How do I uninstall?

```bash
tx self uninstall
```

The installers also accept `--selfuninstall`. Use `tx self uninstall --purge` to remove data and config as well. See [Installation](installation.md#uninstall) for manual removal. A plain uninstall does not delete your data.

## How do I change the port or theme?

Edit `~/.config/peekstore/tasklynx/config.toml`, or run `tx config init` to create it. Restart `tx web` afterwards. Command-line flags such as `--port` override the file for one run.

## What is a branch for?

Branches are independent task lists in the same store. Use them to separate features, releases, or workstreams. `tx list` shows the current branch; `tx list --all` shows everything.

## Can I delete a branch with tasks in it?

No. Move or delete its tasks first, or switch branches and clear them. This prevents accidental loss.

## How do recurring tasks work?

Add `--repeat daily`, `weekly`, `monthly`, or `yearly`, optionally with `--repeat-until YYYY-MM-DD`:

```bash
tx add "Weekly review" --due 2026-11-06 --repeat weekly
tx add "Pay invoices" --due 2026-12-01 --repeat monthly --repeat-until 2027-12-01
```

Completing an occurrence creates the next one with its due date advanced. `tx edit <id> --repeat none` stops recurrence.

## How do tasks connect to Git?

`tx git status` shows branch and commit context, `tx git scan --apply` turns TODO comments into tasks, and `tx git link <id> --commit <sha>` links a task to a commit. The commit hook and trailers are opt-in. See [Git Integration](git-integration.md).

## Does AI send my task data anywhere?

Only when you configure and use a provider. The default build has networking disabled, official release binaries exclude provider support, and `tx ai triage --offline` is fully local. Plans and breakdowns preview before writing. See [AI Setup](ai-setup.md).

## Which Android file should I download?

Use the installer, which selects the right ABI automatically. If downloading manually: `arm64-v8a` for modern phones, `armeabi-v7a` for older 32-bit devices, and `x86_64` for emulators.

## Is the web server safe to expose?

No. `tx web` has no authentication and binds to `127.0.0.1` by default. Keep it local or put it behind an authenticated reverse proxy. See [SECURITY.md](https://github.com/peekstore/tasklynx/blob/main/SECURITY.md).

## Does it support Windows ARM64?

Yes. The release includes `tasklynx-windows-arm64.exe`, and the PowerShell installer detects ARM64.

## Where are the docs source and releases?

- Docs source: <https://github.com/peekstore/tasklynx/tree/main/docs>
- Published docs: <https://peekstore.github.io/tasklynx/>
- Releases: <https://github.com/peekstore/tasklynx/releases>

## How do I contribute?

Read [CONTRIBUTING.md](https://github.com/peekstore/tasklynx/blob/main/CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](https://github.com/peekstore/tasklynx/blob/main/CODE_OF_CONDUCT.md). Security issues should follow [SECURITY.md](https://github.com/peekstore/tasklynx/blob/main/SECURITY.md).

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
