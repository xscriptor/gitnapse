#!/usr/bin/env bash
set -euo pipefail

VERSION="${RELEASE_TAG#v}"

cargo deb --no-build --deb-version "${VERSION}"

asset="gitnapse-${RELEASE_TAG}-linux-ubuntu-amd64.deb"
cp "target/debian/gitnapse_${VERSION}_amd64.deb" "${asset}"

echo "ASSET=${asset}" >> "$GITHUB_ENV"
