#!/usr/bin/env bash
set -euo pipefail

# Ensure cargo is available in subsequent steps.
if [ -d "$HOME/.cargo/bin" ]; then
  echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
fi

# rnestler/archlinux-rust already ships Rust via rustup.
# Install the remaining build tools required by the project and packaging.
pacman -Syu --noconfirm --needed \
  base-devel curl ca-certificates git openssl pkgconf \
  fakeroot binutils libarchive
