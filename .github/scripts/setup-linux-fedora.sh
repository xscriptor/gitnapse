#!/usr/bin/env bash
set -euo pipefail

# Fedora base image does not include Rust; install it from the distribution
# along with the build dependencies and cargo-generate-rpm.
dnf -y install \
  curl gcc gcc-c++ make pkgconf-pkg-config openssl-devel \
  ca-certificates git tar gzip rpm-build rust cargo

cargo install cargo-generate-rpm --locked
