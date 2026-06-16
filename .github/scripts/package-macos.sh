#!/usr/bin/env bash
set -euo pipefail

arch="${ARCH:-$(uname -m)}"
APP_DIR="gitnapse-dmg-staging"

mkdir -p "${APP_DIR}"
cp target/release/gitnapse "${APP_DIR}/gitnapse"

asset="gitnapse-${RELEASE_TAG}-macos-${arch}.dmg"
hdiutil create \
  -volname "gitnapse ${RELEASE_TAG}" \
  -srcfolder "${APP_DIR}" \
  -ov -format UDZO \
  "${asset}"

echo "ASSET=${asset}" >> "$GITHUB_ENV"
