#!/usr/bin/env bash
set -euo pipefail

VERSION="${RELEASE_TAG#v}"
PKGDIR="gitnapse-pkg"

mkdir -p "${PKGDIR}/usr/bin"
cp target/release/gitnapse "${PKGDIR}/usr/bin/gitnapse"

BINSIZE="$(du -sb "${PKGDIR}/usr/bin/gitnapse" | cut -f1)"
{
  echo "pkgname = gitnapse"
  echo "pkgver = ${VERSION}-1"
  echo "arch = x86_64"
  echo "size = ${BINSIZE}"
} > "${PKGDIR}/.PKGINFO"

asset="gitnapse-${RELEASE_TAG}-linux-arch-x86_64.pkg.tar.zst"
bsdtar --zstd -cf "${asset}" -C "${PKGDIR}" .

echo "ASSET=${asset}" >> "$GITHUB_ENV"
