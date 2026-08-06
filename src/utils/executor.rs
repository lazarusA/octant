use std::sync::{Arc, OnceLock};
use zarrs::storage::storage_adapter::async_to_sync::AsyncToSyncBlockOn;

pub struct TokioBlockOn(pub Arc<tokio::runtime::Runtime>);

impl AsyncToSyncBlockOn for TokioBlockOn {
    fn block_on<F: core::future::Future>(&self, future: F) -> F::Output {
        self.0.block_on(future)
    }
}

static SHARED_TOKIO_RT: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();

/// Returns a shared thread-safe Tokio runtime instance.
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

/// Helper struct for spawning background worker threads.
pub struct TaskExecutor;

impl TaskExecutor {
    pub fn spawn<F, T>(f: F) -> std::thread::JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        std::thread::spawn(f)
    }
}
