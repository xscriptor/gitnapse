#!/usr/bin/env bash
set -euo pipefail

if [ -d "$HOME/.cargo/bin" ]; then
  echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
fi

cargo --version
rustc --version
