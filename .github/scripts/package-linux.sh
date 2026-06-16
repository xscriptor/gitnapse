#!/usr/bin/env bash
set -euo pipefail

asset="gitnapse-${RELEASE_TAG}-linux-x86_64.tar.gz"
tar -czf "${asset}" -C target/x86_64-unknown-linux-musl/release gitnapse

echo "ASSET=${asset}" >> "$GITHUB_ENV"
