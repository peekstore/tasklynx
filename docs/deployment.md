# Deployment

## GitHub Releases
CI workflow builds for multiple targets on tag push `v*.*.*`:
- linux-amd64, linux-arm64
- windows-amd64.exe
- darwin-amd64, darwin-arm64
- android-arm64, android-amd64

## GitHub Pages
Docs deployed via `mkdocs` with mkdocs-void theme to `https://peekstore.github.io/tasklynx/`.

## Build
```bash
cargo build --release
```

[Back to README](../README.md)
