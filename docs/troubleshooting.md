# Troubleshooting

## `tx: command not found`

The install directory is not on your `PATH`.

- Linux, macOS, Termux: add this to your shell rc file and restart the shell:

  ```sh
  export PATH="${XDG_CONFIG_HOME:-$HOME/.config}/peekstore/tasklynx/bin:$PATH"
  ```

- Windows: open a new terminal after installing, or confirm `%USERPROFILE%\.config\peekstore\tasklynx\bin` is in your user `PATH`.

Reinstalling also re-adds the path:

```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
```

## The wrong binary was installed (Android or ARM)

Check the platform and artifact the installer would choose:

```bash
DRY_RUN=1 sh installer.sh
```

On Termux, confirm the ABI:

```bash
getprop ro.product.cpu.abi
```

Expected artifacts are `tasklynx-android-arm64-v8a`, `tasklynx-android-armeabi-v7a`, or `tasklynx-android-x86_64`. If detection is wrong, force Termux mode with `TERMUX_VERSION=1`.

## Installer fails with 404

The version being requested has no published assets yet.

- Omit `TASKLYNX_VERSION` to install the latest published release.
- Or pick a tag that is fully published on the [Releases page](https://github.com/peekstore/tasklynx/releases).

## Port already in use

`tx web` prints a bind error when the port is taken. Use another port:

```bash
tx web --port 9000
```

Or change `port` in `~/.config/peekstore/tasklynx/config.toml`.

## The dashboard is reachable from other machines

The default bind address is `127.0.0.1`. If you set `host = "0.0.0.0"` or pass `--host 0.0.0.0`, the server is exposed without authentication. Put it behind an authenticated proxy or bind back to localhost.

## Tasks disappeared after changing `TASKLYNX_HOME`

`TASKLYNX_HOME` relocates the store; the file lives at `$TASKLYNX_HOME/tasklynx/tasks.json`. Point it at the same value you used before, or move `tasks.json` into the new location.

Find the active store path:

```bash
tx init
```

`tx init` prints the resolved `tasks.json` path.

## Data looks corrupted or a write was interrupted

Tasklynx writes atomically, so `tasks.json` should always be complete JSON. If it is not:

1. Keep a copy of the damaged file.
2. Validate it with `jq . tasks.json`.
3. Restore the most recent backup, or start fresh by moving the file aside and running `tx init`.

## `task not found` or ambiguous ID

IDs accept any unique prefix. Copy the short ID from `tx list`, or use the full UUID printed by `tx add`.

## `cannot delete current branch` or `branch is not empty`

- Switch to another branch before deleting it.
- Move or delete the branch's tasks first. `tx list --all` shows every task; `--status` filters help narrow them down.

## Build fails from source

```bash
rustup update stable
cargo clean
cargo build --release
```

Check that you are on Rust 1.70 or newer with `rustc --version`. For Android cross-builds, confirm the NDK path used by `build-android-termux.sh`.

## CI installer test failed on a push

Installer tests run against the newest **published** release. If a release was just tagged but has not finished uploading assets, the test will fail. Re-run the job after the release workflow completes, or dispatch **Installer Test All** manually.

## Still stuck?

Open an issue with the output of `tx --version`, your OS/architecture, the exact command, and the full error:

<https://github.com/peekstore/tasklynx/issues>

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
