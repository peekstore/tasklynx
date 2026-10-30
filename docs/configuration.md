# Configuration

Tasklynx reads a single TOML file for the web server and theme.

## Config file location

```
~/.config/peekstore/tasklynx/config.toml
```

| Platform | Path |
| --- | --- |
| Linux | `~/.config/peekstore/tasklynx/config.toml` |
| macOS | `~/.config/peekstore/tasklynx/config.toml` |
| Windows | `%USERPROFILE%\.config\peekstore\tasklynx\config.toml` |
| Termux | `~/.config/peekstore/tasklynx/config.toml` |

`XDG_CONFIG_HOME` is respected on Unix-like systems; the file is then placed at `$XDG_CONFIG_HOME/peekstore/tasklynx/config.toml`.

## Managing the file

```bash
tx config init    # write defaults and print the path
tx config show    # print the active configuration
```

Edit the file with any text editor. Restart `tx web` to apply changes.

## Default configuration

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

[ai]
provider = "none"
model = "llama3.1"
endpoint = "http://127.0.0.1:11434"
max_context_tasks = 50
include_descriptions = true
redact = true
```

| Key | Type | Purpose |
| --- | --- | --- |
| `host` | string | Bind address for `tx web` |
| `port` | integer | Bind port for `tx web` |
| `accent` | string | Accent color |
| `background` | string | Page background |
| `panel` | string | Card and panel background |
| `foreground` | string | Primary text |
| `muted` | string | Secondary text |
| `font_family` | string | CSS font stack for the dashboard |
| `default_project` | string | Default project for new tasks |
| `ai.provider` | string | `none`, `ollama`, or `openai-compatible` |
| `ai.model` | string | Model name sent to the provider |
| `ai.endpoint` | string | Provider base URL |
| `ai.max_context_tasks` | integer | Maximum open tasks included in prompt context |
| `ai.include_descriptions` | boolean | Include task descriptions in prompt context |
| `ai.redact` | boolean | Redact common secrets before sending prompts |

## AI providers

AI is disabled by default. The `ollama` and `openai-compatible` providers require a build with the optional `ai` feature:

```bash
cargo build --features ai
```

Ollama example:

```toml
[ai]
provider = "ollama"
model = "llama3.1"
endpoint = "http://127.0.0.1:11434"
```

OpenAI-compatible example:

```toml
[ai]
provider = "openai-compatible"
model = "gpt-4.1-mini"
endpoint = "https://api.openai.com/v1"
```

Google Gemini through AI Studio:

```toml
[ai]
provider = "openai-compatible"
model = "gemini-2.5-flash"
endpoint = "https://generativelanguage.googleapis.com/v1beta/openai"
```

The endpoint must include the API version path, such as `/v1` or `/v1beta/openai`. For hosted endpoints, put the key in `TASKLYNX_AI_API_KEY`; it is never stored in `config.toml`. Run `tx ai doctor` to check provider reachability and model availability. AI commands build prompt context from the current branch only and require `--apply` before creating tasks.

Command-line flags override the file for that run:

```bash
tx web --host 0.0.0.0 --port 9000
```

## Data storage

Tasks and branches are stored in `tasks.json`.

| Platform | Path |
| --- | --- |
| Linux | `~/.local/share/tasklynx/tasks.json` |
| macOS | `~/Library/Application Support/tasklynx/tasks.json` |
| Windows | `%LOCALAPPDATA%\tasklynx\tasks.json` |

Override the location with `TASKLYNX_HOME`:

```bash
export TASKLYNX_HOME="$HOME/tasklynx-data"
# store is now $HOME/tasklynx-data/tasklynx/tasks.json
```

Writes are atomic: Tasklynx writes a temporary file and renames it over `tasks.json`.

## Environment variables

| Variable | Used by | Effect |
| --- | --- | --- |
| `XDG_CONFIG_HOME` | Tasklynx | Base directory for `config.toml` |
| `TASKLYNX_HOME` | Tasklynx | Relocates the data store |
| `TASKLYNX_AI_API_KEY` | `tx ai` | API key for OpenAI-compatible providers |
| `TASKLYNX_VERSION` | Installers | Pin the release to install |
| `DRY_RUN` | `installer.sh` | Print the resolved artifact without installing |
| `TERMUX_VERSION` | `installer.sh` | Force Termux/Android detection |

## Backups and portability

Because the store is a single JSON file, backups are simple:

```bash
cp ~/.local/share/tasklynx/tasks.json ~/tasks-backup-$(date +%F).json
```

To move to another machine, copy `tasks.json` to the same data path (or point `TASKLYNX_HOME` at the copied directory) and install Tasklynx there.

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
