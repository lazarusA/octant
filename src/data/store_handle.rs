use std::error::Error;
use std::sync::Arc;

use zarrs::storage::ReadableWritableListableStorage;

use crate::app::StoreKind;

/// Shared handle to an opened dataset/store.
///
/// The important idea is that the store is opened once and then reused
/// by many BlockRequests.
///
/// One StoreHandle can therefore service:
///
///     temperature
///     pressure
///     humidity
///     wind_u
///     wind_v
///
/// without reopening the underlying dataset for every variable.
#[derive(Clone)]
pub struct StoreHandle {
    pub kind: StoreKind,
    pub target: String,
    pub storage: ReadableWritableListableStorage,
}

impl StoreHandle {
    pub fn new(
        kind: StoreKind,
        target: impl Into<String>,
        storage: ReadableWritableListableStorage,
    ) -> Self {
        Self {
            kind,
            target: target.into(),
            storage,
        }
    }

    pub fn kind(&self) -> StoreKind {
        self.kind
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn storage(&self) -> ReadableWritableListableStorage {
        self.storage.clone()
    }
}