#!/usr/bin/env bash
set -euo pipefail

echo "=== cargo fmt --check ==="
cargo fmt --check

echo ""
echo "=== cargo clippy -- -D warnings ==="
cargo clippy -- -D warnings

echo ""
echo "=== cargo test ==="
cargo test

echo ""
echo "All CI checks passed."
