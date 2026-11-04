//! Icechunk backend implementation for `BlockStore`.

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use native::{IcechunkBlockStore, build_sync_icechunk_store};

#[cfg(target_arch = "wasm32")]
pub use wasm_wrapper::IcechunkBlockStore;

#[cfg(target_arch = "wasm32")]
mod wasm_wrapper {
    use super::wasm::WasmIcechunkBlockStore;
    use crate::data::DatasetMetadata;
    use crate::data::blocks::{BlockStore, BlockStoreError};
    use crate::data::octant_block::OctantBlock;
    use crate::data::slice_request::SliceRequest;
    use std::sync::Arc;

    pub struct IcechunkBlockStore {
        inner: Arc<WasmIcechunkBlockStore>,
    }

    impl IcechunkBlockStore {
        pub fn open(location: &str) -> Result<Self, BlockStoreError> {
            let inner = WasmIcechunkBlockStore::get_or_create(location);
            Ok(Self { inner })
        }
    }

    impl BlockStore for IcechunkBlockStore {
        fn backend_name(&self) -> &str {
            self.inner.backend_name()
        }

        fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
            self.inner.variables()
        }

        fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
            self.inner.inspect()
        }

        fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
            self.inner.fetch_block(request)
        }
    }
}
