# Deployment

Tasklynx ships binaries, packages, documentation, and a container from GitHub Actions. This page describes what each workflow produces.

## Release assets

Pushing a tag matching `v*` triggers the `Auto Build & Release` workflow, which builds and uploads:

| Platform | Architecture | Artifact |
| --- | --- | --- |
| Linux | x86_64 | `tasklynx-linux-amd64` |
| Linux | aarch64 | `tasklynx-linux-arm64` |
| macOS | x86_64 | `tasklynx-darwin-amd64` |
| macOS | Apple Silicon | `tasklynx-darwin-arm64` |
| Windows | x86_64 | `tasklynx-windows-amd64.exe` |
| Windows | ARM64 | `tasklynx-windows-arm64.exe` |
| Android | `arm64-v8a` | `tasklynx-android-arm64-v8a` |
| Android | `armeabi-v7a` | `tasklynx-android-armeabi-v7a` |
| Android | `x86_64` | `tasklynx-android-x86_64` |

Each release also publishes `SHA256SUMS.txt`, archives, Debian packages, and per-artifact `.sha256` files. Android artifacts are built with NDK r25c against API level 21 and use Android ABI names deliberately. Official release binaries are built without the `ai` feature.

## Workflows

| Workflow | Trigger | Purpose |
| --- | --- | --- |
| `ci.yml` | push to `main`, pull request | `fmt`, `clippy`, and tests |
| `release.yml` (`Auto Build & Release`) | tag `v*`, manual | Build all targets and publish a release |
| `package-publish.yml` (`Package & Distribute`) | push to `main`, tag `v*`, PR, manual | Test pipx installs, build Debian/Chocolatey/npm packages, and publish registry packages on tags |
| `installer-test-all.yml` | push to `main`, release completed, PR, manual | Test installers on Linux x64/arm64, macOS, Windows, and Android ABIs |
| `docs.yml` | push to `main`, manual | Build and deploy the MkDocs site |
| `container.yml` (`Publish container`) | push to `main`, tag `v*`, manual | Build and push the container image |
| `screenshots.yml` (`Capture Screenshots`) | push to `main` when web assets change, manual | Regenerate screenshots with Chromium and Puppeteer |

Installer tests resolve the newest **published** release rather than the version in `.version`. This prevents a race where a release commit is tested before its assets exist. Use `TASKLYNX_VERSION` to pin a tag when running them locally or manually.

## Container

The image is published to GitHub Container Registry on version tags and from `main`:

```
ghcr.io/peekstore/tasklynx
```

```bash
docker run --rm ghcr.io/peekstore/tasklynx --version
```

Mount a host directory and set `TASKLYNX_HOME` to persist tasks:

```bash
docker run --rm -v "$PWD/tasklynx:/data" -e TASKLYNX_HOME=/data \
  ghcr.io/peekstore/tasklynx list
```

## Documentation site

Docs are published with MkDocs and the `mkdocs-void` theme:

- Site: <https://peekstore.github.io/tasklynx/>
- Source: [`docs/`](https://github.com/peekstore/tasklynx/tree/main/docs)
- Config: [`mkdocs.yml`](https://github.com/peekstore/tasklynx/blob/main/mkdocs.yml)

Build locally with `mkdocs serve`.

## Reproducing a release locally

```bash
cargo build --release
```

For Android, use `./build-android-termux.sh`. Compare your artifact's hash with `SHA256SUMS.txt` from the release to confirm a match.

[Back to README](https://github.com/peekstore/tasklynx/blob/main/README.md)
