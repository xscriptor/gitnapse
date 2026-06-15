$ErrorActionPreference = "Stop"

# The runner has Rust installed via actions-rust-lang/setup-rust-toolchain.
# Just verify the toolchain is available.
cargo --version
rustc --version
