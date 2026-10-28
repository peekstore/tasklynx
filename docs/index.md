<p align="center">
  <img src="logo.svg" alt="Tasklynx logo" width="120" height="120">
</p>

# Tasklynx Documentation

Tasklynx is a local-first task manager for developers. It combines a scriptable CLI, an interactive terminal Kanban board, and a small web dashboard on top of a single JSON store — with Git-style branches for parallel work.

- **Version:** 0.0.1-beta-2
- **License:** [MIT](https://github.com/peekstore/tasklynx/blob/main/LICENSE)
- **Repository:** [github.com/peekstore/tasklynx](https://github.com/peekstore/tasklynx)
- **Releases:** [github.com/peekstore/tasklynx/releases](https://github.com/peekstore/tasklynx/releases)

## Start here

| Guide | What it covers |
| --- | --- |
| [Getting Started](getting-started.md) | Install, initialize, and complete your first task |
| [Installation](installation.md) | Every install method, updates, and uninstall |
| [CLI Reference](cli.md) | Commands, flags, and examples |
| [Configuration](configuration.md) | Config file, data paths, and theming |

## Understand the project

| Guide | What it covers |
| --- | --- |
| [Architecture](architecture.md) | Modules, data flow, and storage design |
| [Development](development.md) | Build, test, and release workflow |
| [Deployment](deployment.md) | Release artifacts, Pages, and containers |

## Help

| Guide | What it covers |
| --- | --- |
| [Troubleshooting](troubleshooting.md) | Common problems and fixes |
| [FAQ](faq.md) | Frequently asked questions |
| [Screenshots](screenshots.md) | Interface previews |

## At a glance

```bash
# Install (Linux, macOS, Termux)
curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh

# Create your first task
tx init
tx add "Ship the beta" --priority high --due 2026-12-31
tx list
tx board
```

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
