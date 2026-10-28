# Configuration

Configuration is stored in `~/.config/tasklynx/config.toml` (or platform equivalent). Web server settings and theme colors can be customized.

## View Config

```bash
tx config show
```

## Initialize Config

```bash
tx config init
```

## Default Configuration

```toml
host = "127.0.0.1"
port = 8787
accent = "#ff453a"
background = "#070809"
panel = "#101216"
foreground = "#f5f5f5"
muted = "#8f949e"
font_family = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace"
default_project = "default"
```

## Data Storage

Tasks are stored as `tasks.json` in the platform data directory:
- Linux: `~/.local/share/tasklynx/`
- macOS: `~/Library/Application Support/tasklynx/`
- Windows: `%LOCALAPPDATA%\tasklynx\`

Override with `TASKLYNX_HOME` environment variable.

## Theme Customization

Edit `~/.config/tasklynx/config.toml` to customize the web dashboard colors. Changes take effect on next `tx web` start.
