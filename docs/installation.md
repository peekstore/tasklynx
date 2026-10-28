# Installation

## One-line Installers

### Linux / macOS / Android (Termux)
```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
```

The installer detects your OS and architecture, downloads the matching release
binary to `~/.config/peekstore/tasklynx/bin`, creates a `tx` alias, and adds the
directory to your shell `PATH`.

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1 | iex
```

Installs to `%USERPROFILE%\.config\peekstore\tasklynx\bin` and updates the user
`PATH`.

### Nix / NixOS
```bash
# Run without installing
nix run github:peekstore/tasklynx

# Install into your profile
nix profile install github:peekstore/tasklynx
```

## Android (Termux)

Termux uses Android's `bionic` libc, so it needs the dedicated `tasklynx-android-*`
build. Android binaries are published using Android ABI names (not `amd64`/`arm64`).
The installer detects Termux automatically (`TERMUX_VERSION` env var or
`/data/data/com.termux`) and picks the artifact matching `ro.product.cpu.abi`.

| Android ABI | Device | Artifact |
| --- | --- | --- |
| `arm64-v8a` | Modern 64-bit ARM phones | `tasklynx-android-arm64-v8a` |
| `armeabi-v7a` | Older 32-bit ARM devices | `tasklynx-android-armeabi-v7a` |
| `x86_64` | Emulators / x86_64 tablets | `tasklynx-android-x86_64` |

```bash
pkg install curl
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
source ~/.bashrc
tx --version
```

To build it yourself inside Termux:

```bash
pkg install rust clang
cargo build --release
```

Or use the helper script from a desktop checkout:

```bash
./build-android-termux.sh
```

## From Source
```bash
git clone https://github.com/peekstore/tasklynx.git
cd tasklynx
cargo build --release
```

## Update
```bash
tx self update
```

## Uninstall

### Self command
```bash
tx self uninstall
```

### Installer script (Linux / macOS / Termux)
```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh -s -- --selfuninstall
```

### Installer script (Windows PowerShell)
```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1))) --selfuninstall
```

### Nix / NixOS
```bash
nix profile remove tasklynx
```

### Manual removal
```bash
# Linux / macOS / Termux
rm -rf ~/.config/peekstore/tasklynx
```
Then remove the line `export PATH="$HOME/.config/peekstore/tasklynx/bin:$PATH"`
from your shell rc file (`.bashrc`, `.zshrc`, `.profile`, ...).

On Windows, remove the folder and delete the matching entry from your user
`PATH`:

```powershell
Remove-Item -Recurse -Force "$env:USERPROFILE\.config\peekstore\tasklynx"
```

[Back to README](../README.md)
