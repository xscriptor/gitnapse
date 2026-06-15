#!/usr/bin/env bash
set -euo pipefail

VERSION="${RELEASE_TAG#v}"

cargo generate-rpm --set-metadata="version='${VERSION}'"

asset="gitnapse-${RELEASE_TAG}-linux-fedora-x86_64.rpm"
cp "target/generate-rpm/gitnapse-${VERSION}-1.x86_64.rpm" "${asset}"

echo "ASSET=${asset}" >> "$GITHUB_ENV"
