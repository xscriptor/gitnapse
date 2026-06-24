use std::sync::OnceLock;
use tokio::runtime::Runtime;

pub fn get_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Cannot create global tokio runtime")
    })
}

/// Install the rustls crypto provider (required before any TLS connection).
/// Safe to call multiple times – subsequent calls are no-ops.
pub fn ensure_crypto_provider() {
    if rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider())
        .is_err()
    {
        log::warn!("could not install rustls crypto provider (may already be set)");
    }
}
