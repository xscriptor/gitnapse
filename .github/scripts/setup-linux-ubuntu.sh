#!/usr/bin/env bash
set -euo pipefail

# Ensure cargo is available in subsequent steps.
if [ -d "$HOME/.cargo/bin" ]; then
  echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
fi

# cimg/rust already ships Rust. Install build dependencies and cargo-deb.
export DEBIAN_FRONTEND=noninteractive

if [ "$(id -u)" -eq 0 ]; then
  apt-get update
  apt-get install -y libssl-dev pkg-config dpkg-dev
else
  sudo apt-get update
  sudo apt-get install -y libssl-dev pkg-config dpkg-dev
fi

cargo install cargo-deb --locked
