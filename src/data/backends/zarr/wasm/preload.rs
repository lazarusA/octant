//! Metadata finalization for WebAssembly Zarr.

use std::collections::HashMap;

use super::WasmZarrBlockStore;
use crate::data::{DatasetMetadata, VariableInfo};

impl WasmZarrBlockStore {
    /// Finalizes and caches dataset metadata. Coordinates are left out, so the dataset opens
    /// with its metadata alone; they are read per variable (`chunk_source::load_variable_coordinates`).
    pub async fn finalize_metadata(
        &self,
        variables: Vec<VariableInfo>,
        ome: Option<std::sync::Arc<crate::data::ome::OmeDataset>>,
        clean_url: &str,
    ) -> DatasetMetadata {
        let dataset_name = clean_url
            .split('/')
            .next_back()
            .unwrap_or(clean_url)
            .to_string();

        let dataset_metadata = DatasetMetadata {
            name: dataset_name,
            store_type: "zarr".to_string(),
            variables,
            dimension_coordinates: HashMap::new(),
            ome,
        };

        let mut guard = self.metadata.write().unwrap_or_else(|p| p.into_inner());
        *guard = Some(dataset_metadata.clone());
        dataset_metadata
    }
}
