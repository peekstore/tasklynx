# Installation

## One-line Installers

### Linux / macOS
```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1 | iex
```

## Self Uninstall

### Linux / macOS
```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh -s -- --selfuninstall
```

### Windows (PowerShell)
```powershell
Invoke-RestMethod -Uri "https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1" | Invoke-Expression -ArgumentList "--selfuninstall"
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

[Back to README](../README.md)
