# Frequently Asked Questions

## Is my data stored locally?

Yes. Tasklynx writes a single `tasks.json` file on your machine. There is no account, sync service, or telemetry. See [Configuration](configuration.md#data-storage) for the exact path.

## What is `tx`?

`tx` is a short alias for the `tasklynx` binary. Both names run the same program.

## Does it work offline?

Yes. Tasks are local, and the web dashboard is served from the binary. Only `tx self update` and the installers use the network.

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

The installers also accept `--selfuninstall`. See [Installation](installation.md#uninstall) for manual and Nix removal. Uninstalling does not delete your data.

## How do I change the port or theme?

Edit `~/.config/peekstore/tasklynx/config.toml`, or run `tx config init` to create it. Restart `tx web` afterwards. Command-line flags such as `--port` override the file for one run.

## What is a branch for?

Branches are independent task lists in the same store. Use them to separate features, releases, or workstreams. `tx list` shows the current branch; `tx list --all` shows everything.

## Can I delete a branch with tasks in it?

No. Move or delete its tasks first, or switch branches and clear them. This prevents accidental loss.

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
