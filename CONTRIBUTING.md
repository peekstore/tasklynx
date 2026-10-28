# Contributing to Tasklynx

Thanks for taking the time to contribute. This guide covers the local setup, coding standards, and review process.

## Code of Conduct

By participating, you agree to follow our [Code of Conduct](CODE_OF_CONDUCT.md). Please report unacceptable behavior to [peekstore585@gmail.com](mailto:peekstore585@gmail.com).

## Ways to contribute

- Report bugs and request features through [GitHub Issues](https://github.com/peekstore/tasklynx/issues).
- Improve the documentation in [`docs/`](docs).
- Fix bugs or implement features with a pull request.
- Share reproducible platform reports (especially Termux and Windows).

## Development setup

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or newer (stable toolchain).
- Git.
- `clippy` and `rustfmt` components (`rustup component add clippy rustfmt`).
- For Android cross-builds: Android NDK r25c and the targets listed in [`build-android-termux.sh`](build-android-termux.sh).

### Clone and build

```bash
git clone https://github.com/peekstore/tasklynx.git
cd tasklynx
cargo build
```

### Run from source

```bash
cargo run -- --help
cargo run -- list
cargo run -- board
cargo run -- web
```

Use a temporary store while experimenting:

```bash
TASKLYNX_HOME=/tmp/tasklynx-dev cargo run -- add "Test task"
```

## Before you open a pull request

Run the same checks as CI:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

If you changed installer logic, also exercise it in dry-run mode:

```bash
DRY_RUN=1 sh installer.sh
```

## Coding standards

- Match the existing style; run `cargo fmt` before committing.
- Keep `cargo clippy` warning-free with `-D warnings`.
- Prefer small, focused changes with a clear purpose.
- Add or update tests for behavior changes.
- Do not add dependencies unless they are necessary; explain the trade-off in the PR.
- Never commit secrets, tokens, or machine-specific paths.
- Keep user-facing text (CLI help, errors, docs) clear and concise.

## Commit messages

Use a short imperative subject line, optionally with a conventional prefix:

```
feat: add due-date filters to tx list
fix: remove dangling tx symlink on uninstall
docs: rewrite installation guide
ci: pin installer tests to the latest published release
```

Keep the subject under ~72 characters. Use the body to explain *why* when the change is not obvious.

## Branch naming

Create a branch from `main`:

```
feat/<short-description>
fix/<short-description>
docs/<short-description>
```

## Pull request process

1. Fork the repository and create your branch from `main`.
2. Make your change and add tests where practical.
3. Run the checks listed above.
4. Open a pull request with:
   - a clear title and description,
   - the motivation and approach,
   - linked issues (`Closes #123`),
   - screenshots for UI changes.
5. Address review feedback with additional commits; avoid force-pushing after review starts.
6. A maintainer will merge once CI is green and the change is approved.

## Documentation

- User documentation lives in [`docs/`](docs) and is published with MkDocs.
- Keep command examples runnable and consistent with the current CLI.
- Update the [CHANGELOG](CHANGELOG.md) for user-visible changes.

## Reporting bugs

Include:

- the Tasklynx version (`tx --version`),
- operating system and architecture (and Termux ABI if applicable),
- the exact command and full output,
- what you expected to happen,
- whether the data store was created with `tx init`.

## Security

Do **not** open a public issue for a security vulnerability. Follow the process in [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
