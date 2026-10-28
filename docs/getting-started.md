# Getting Started

## Installation

### One-line installer (Recommended)

**Linux / macOS**
```bash
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh
```

**Windows (PowerShell)**
```powershell
irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1 | iex
```

### From source
```bash
git clone https://github.com/peekstore/tasklynx.git
cd tasklynx
cargo build --release
cp target/release/tasklynx ~/.local/bin/
```

### Manual download
Download from [Releases](https://github.com/peekstore/tasklynx/releases).

## Initial Setup

Initialize Tasklynx (creates data directory and default config):
```bash
tx init
```

## Basic Workflow

```bash
# Add a task
tx add "Implement authentication" --project api --tag backend,auth --priority high --due 2026-12-31

# List tasks on current branch
tx list

# View Kanban board
tx board

# Mark as in progress
tx doing <id>

# Complete task
tx complete <id>

# Start web dashboard
tx web
```

## Self Management

```bash
# Update to latest version
tx self update

# Uninstall
tx self uninstall
```
