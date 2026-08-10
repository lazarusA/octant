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

/// Returns a shared thread-safe Tokio runtime instance.
#[cfg(not(target_arch = "wasm32"))]
pub fn get_shared_tokio_rt() -> Arc<tokio::runtime::Runtime> {
    SHARED_TOKIO_RT
        .get_or_init(|| {
            Arc::new(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create shared Tokio runtime"),
            )
        })
        .clone()
}

/// Helper struct for spawning background worker tasks on both native OS threads and WASM event loop.
pub struct TaskExecutor;

impl TaskExecutor {
    pub fn spawn<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::spawn(f);
        }
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                f();
            });
        }
    }
}
