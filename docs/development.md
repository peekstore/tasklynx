# Development

```bash
cargo fmt
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Release builds are triggered by pushing a semantic version tag such as `v1.0.0`.
