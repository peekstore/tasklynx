# Deployment

Tasklynx ships binaries, documentation, and a container from GitHub Actions. This page describes what each workflow produces.

## Release assets

Pushing a tag matching `v*.*.*` triggers the release workflow, which builds and uploads:

| Platform | Architecture | Artifact |
| --- | --- | --- |
| Linux | x86_64 | `tasklynx-linux-amd64` |
| Linux | aarch64 | `tasklynx-linux-arm64` |
| macOS | x86_64 | `tasklynx-macos-amd64` |
| macOS | Apple Silicon | `tasklynx-macos-arm64` |
| Windows | x86_64 | `tasklynx-windows-amd64.exe` |
| Windows | ARM64 | `tasklynx-windows-arm64.exe` |
| Android | `arm64-v8a` | `tasklynx-android-arm64-v8a` |
| Android | `armeabi-v7a` | `tasklynx-android-armeabi-v7a` |
| Android | `x86_64` | `tasklynx-android-x86_64` |

Each release also publishes `SHA256SUMS.txt`. Android artifacts are built with NDK r25c against API level 21 and use Android ABI names deliberately.

## Workflows

| Workflow | Trigger | Purpose |
| --- | --- | --- |
| `ci.yml` | push, pull request | `fmt`, `clippy`, and tests |
| `release.yml` | tag `v*.*.*` | Build all targets and publish a release |
| `installer-test-all.yml` | push to `main`, release completed, PR, manual | Test installers on Linux x64/arm64, macOS, Windows, Android ABIs, and Nix |
| `docs.yml` | push to `main` | Build and deploy the MkDocs site |
| `container.yml` | tag `v*` | Build and push the container image |

Installer tests resolve the newest **published** release rather than the version in `.version`. This prevents a race where a release commit is tested before its assets exist. Use `TASKLYNX_VERSION` to pin a tag when running them locally or manually.

## Container

The image is published to GitHub Container Registry on version tags:

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
