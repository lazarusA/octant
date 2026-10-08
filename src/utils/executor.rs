#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, OnceLock};
#[cfg(not(target_arch = "wasm32"))]
use zarrs::storage::storage_adapter::async_to_sync::AsyncToSyncBlockOn;

#[cfg(not(target_arch = "wasm32"))]
pub struct TokioBlockOn(pub Arc<tokio::runtime::Runtime>);

#[cfg(not(target_arch = "wasm32"))]
impl AsyncToSyncBlockOn for TokioBlockOn {
    fn block_on<F: core::future::Future>(&self, future: F) -> F::Output {
        self.0.block_on(future)
    }
}

#[cfg(not(target_arch = "wasm32"))]
static SHARED_TOKIO_RT: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
/// Returns a shared thread-safe Tokio runtime instance.
pub fn get_shared_tokio_rt() -> Arc<tokio::runtime::Runtime> {
    SHARED_TOKIO_RT
        .get_or_init(|| {
            Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create shared Tokio runtime"),
            )
        })
        .clone()
}

/// Runs `f`, turning a panic into an error: a backend, codec or decoder that
/// panics on a file it cannot read (an unsupported format, a malformed
/// header) then fails like any other load, and the error reaches the UI,
/// instead of a background thread panicking and aborting the app. Panics
/// still print through the panic hook; under `panic = "abort"` (the browser)
/// nothing can be caught, so readers must also reject such files up front.
pub fn catch_panic<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".to_string());
        Err(format!("unreadable or unsupported data: {message}"))
    })
}

/// Helper struct for spawning background worker tasks.
pub struct TaskExecutor;

impl TaskExecutor {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn<F, T>(f: F) -> std::thread::JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        std::thread::spawn(f)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn_background<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        rayon::spawn(f);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn spawn_background<F>(f: F)
    where
        F: FnOnce() + 'static,
    {
        f();
    }
}
