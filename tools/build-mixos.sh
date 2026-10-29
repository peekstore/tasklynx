#!/usr/bin/env bash
# build-mixos.sh - Build script for MiXOS
set -e
echo "Building tasklynx for MiXOS..."
cargo build --release
cp target/release/tasklynx ./tasklynx-mixos-$(uname -m)
echo "Done: tasklynx-mixos-$(uname -m)"
