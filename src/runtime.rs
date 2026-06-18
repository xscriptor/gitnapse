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
