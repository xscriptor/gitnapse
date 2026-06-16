#!/usr/bin/env bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive
sudo apt-get install -y musl-tools

rustup target add x86_64-unknown-linux-musl
