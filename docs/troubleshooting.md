# Troubleshooting

## Binary not found
Ensure install directory is in PATH. Use `tx self update` or reinstall.

## Web port in use
Use `tx web --port <other_port>`.

## Data location
Check platform data dir or set `TASKLYNX_HOME`.

## Build issues
```bash
cargo clean
cargo build --release
```

[Back to README](../README.md)
