//! Metadata finalization and per-variable coordinate preloading for WebAssembly Zarr.

use std::collections::HashMap;

use super::WasmZarrBlockStore;
use crate::data::backends::coord_bounds::{
    CoordPreload, fetch_all_dimension_coordinates_for_variables, settle_preloaded_coordinates,
};
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};

use zarrs::array::ArraySubset;

impl WasmZarrBlockStore {
    /// Finalizes and caches dataset metadata. Coordinates are left out, so the dataset opens
    /// with its metadata alone; they are read per variable ([`Self::load_variable_coordinates`]).
    pub async fn finalize_metadata(
        &self,
        variables: Vec<VariableInfo>,
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
        };

        let mut guard = self.metadata.write().unwrap_or_else(|p| p.into_inner());
        *guard = Some(dataset_metadata.clone());
        dataset_metadata
    }

    /// Fetches the coordinate arrays of `variable`'s dimensions (spatial first, at the root
    /// or in its group) and reads them, keyed like `DatasetMetadata::dimension_coordinates`.
    pub async fn load_variable_coordinates(
        &self,
        variable: &VariableInfo,
    ) -> HashMap<String, CoordValues> {
        let mut incomplete = Vec::new();
        for (path, len) in
            super::coord_paths::variable_coordinate_arrays(&self.memory_store, variable)
        {
            let state = self.preload_coordinate_chunks_1d(&path, len).await;
            if state != CoordPreload::Complete {
                incomplete.push((path, state));
            }
        }
        let mut coords = fetch_all_dimension_coordinates_for_variables(
            self.memory_store.clone(),
            std::slice::from_ref(variable),
            Some(&self.base_url),
        );
        settle_preloaded_coordinates(&mut coords, &incomplete);
        coords
    }

    /// Preloads every chunk of a 1D coordinate, which is read whole (`zarr::coords`): the
    /// first and last chunks first, as the endpoints, then the rest.
    #[allow(clippy::single_range_in_vec_init)]
    pub async fn preload_coordinate_chunks_1d(&self, coord_name: &str, count: u64) -> CoordPreload {
        let Some(last) = count.checked_sub(1) else {
            return CoordPreload::Complete;
        };
        for ends in [0..1, last..count] {
            let subset = ArraySubset::new_with_ranges(&[ends]);
            if Box::pin(self.preload_chunks_for_subset(coord_name, &subset, None))
                .await
                .is_err()
            {
                log::error!("[WASM Zarr] Coordinate '{coord_name}' endpoints failed to load");
                return CoordPreload::Failed;
            }
        }
        let subset = ArraySubset::new_with_ranges(&[0..count]);
        match Box::pin(self.preload_chunks_for_subset(coord_name, &subset, None)).await {
            Ok(()) => CoordPreload::Complete,
            Err(e) => {
                log::error!("[WASM Zarr] Coordinate '{coord_name}' partly loaded: {e}");
                CoordPreload::Partial
            }
        }
    }
}
