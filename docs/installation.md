# Installation

## Requirements

- **Linux / macOS:** a POSIX shell (`sh`, `bash`, `zsh`) and `curl` or `wget`.
- **Windows:** PowerShell 5.1 or newer.
- **Termux:** the Termux app from F-Droid, with `curl` installed.
- **From source:** Rust 1.70+ and Git.

No runtime dependencies are required; Tasklynx ships as a single binary.

## One-line installer

### Linux, macOS, and Termux

```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
```

The installer:

1. Detects the operating system and CPU architecture (including Termux ABIs).
2. Downloads the matching release binary.
3. Installs `tasklynx` and the `tx` alias to `${XDG_CONFIG_HOME:-$HOME/.config}/peekstore/tasklynx/bin`.
4. Adds that directory to your shell `PATH`.

Restart your shell, or run `source ~/.bashrc` (or your shell's rc file), then verify:

```bash
tx --version
```

### Windows

```powershell
irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1 | iex
```

The installer writes to `%USERPROFILE%\.config\peekstore\tasklynx\bin`, updates the user `PATH`, and creates a Start Menu shortcut. Open a new terminal afterwards.

### Pinning a version

Both installers honor `TASKLYNX_VERSION`:

```bash
TASKLYNX_VERSION=v0.0.1-beta-2 \
  curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
```

```powershell
$env:TASKLYNX_VERSION = "v0.0.1-beta-2"
irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1 | iex
```

### Dry run

Preview the resolved artifact without installing:

```bash
DRY_RUN=1 sh installer.sh
```

On Windows, run `.\installer.ps1 -DryRun` from a checkout.

## Nix and NixOS

```bash
# Run without installing
nix run github:peekstore/tasklynx

# Install into your profile
nix profile install github:peekstore/tasklynx
```

## Android (Termux)

Termux uses Android's `bionic` libc, so it needs a dedicated Android build. Android artifacts use Android ABI names, not desktop names.

The installer detects Termux automatically (`TERMUX_VERSION` or `/data/data/com.termux`) and selects the artifact matching `ro.product.cpu.abi`:

| Android ABI | Devices | Artifact |
| --- | --- | --- |
| `arm64-v8a` | Modern 64-bit ARM phones | `tasklynx-android-arm64-v8a` |
| `armeabi-v7a` | Older 32-bit ARM devices | `tasklynx-android-armeabi-v7a` |
| `x86_64` | Emulators and x86_64 tablets | `tasklynx-android-x86_64` |

```bash
pkg install curl
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
source ~/.bashrc
tx --version
```

To build inside Termux:

```bash
pkg install rust clang
cargo build --release
```

To cross-compile from a desktop checkout, use [`build-android-termux.sh`](https://github.com/peekstore/tasklynx/blob/main/build-android-termux.sh).

## Manual download

Download the archive for your platform from [Releases](https://github.com/peekstore/tasklynx/releases), verify it against `SHA256SUMS.txt`, and place the binary on your `PATH`.

```bash
sha256sum -c SHA256SUMS.txt
```

## From source

```bash
git clone https://github.com/peekstore/tasklynx.git
cd tasklynx
cargo build --release
install -Dm755 target/release/tasklynx ~/.local/bin/tasklynx
```

## Update

```bash
tx self update
```

This re-runs the official installer and replaces the binary in place.

## Uninstall

### Self command

```bash
tx self uninstall
```

This removes the binary and the `tx` alias.

### Installer script (Linux, macOS, Termux)

```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh -s -- --selfuninstall
```

### Installer script (Windows PowerShell)

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1))) --selfuninstall
```

### Nix and NixOS

```bash
nix profile remove tasklynx
```

### Manual removal

Linux, macOS, and Termux:

```bash
rm -rf ~/.config/peekstore/tasklynx
```

Then remove the matching `export PATH=...` line from your shell rc file (`.bashrc`, `.zshrc`, `.profile`, and so on).

Windows:

```powershell
Remove-Item -Recurse -Force "$env:USERPROFILE\.config\peekstore\tasklynx"
```

Also remove the entry from your user `PATH` in **System Properties → Environment Variables**.

Uninstalling removes the program, not your data. Delete the data store separately if you want a clean slate:

```bash
rm -rf ~/.local/share/tasklynx        # Linux
rm -rf ~/Library/Application\ Support/tasklynx  # macOS
```

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
