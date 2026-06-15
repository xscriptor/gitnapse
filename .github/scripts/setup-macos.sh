#!/usr/bin/env bash
set -euo pipefail

# Ensure cargo is available in subsequent steps.
if [ -d "$HOME/.cargo/bin" ]; then
  echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
fi

# The runner has Rust installed via actions-rust-lang/setup-rust-toolchain.
# Just verify the toolchain is available.
cargo --version
rustc --version
