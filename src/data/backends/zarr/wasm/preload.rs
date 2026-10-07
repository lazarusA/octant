//! Coordinate preloading and metadata finalization for WebAssembly Zarr.

use super::WasmZarrBlockStore;
use crate::data::backends::zarr::strings::coordinate_preload_ranges;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::utils::metadata::open_or_instantiate_array_normalized;

use zarrs::array::ArraySubset;

impl WasmZarrBlockStore {
    /// Finalizes metadata by preloading coordinates, resolving dimension coordinates, and caching dataset metadata.
    pub async fn finalize_metadata(
        &self,
        variables: Vec<VariableInfo>,
        clean_url: &str,
    ) -> DatasetMetadata {
        self.preload_coordinate_variables(&variables).await;
        let dimension_coordinates =
            crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables(
                self.memory_store.clone(),
                &variables,
                Some(clean_url),
            );

        let dataset_name = clean_url
            .split('/')
            .next_back()
            .unwrap_or(clean_url)
            .to_string();

        let dataset_metadata = DatasetMetadata {
            name: dataset_name,
            store_type: "zarr".to_string(),
            variables,
            dimension_coordinates,
        };

        let mut guard = self.metadata.write().unwrap_or_else(|p| p.into_inner());
        *guard = Some(dataset_metadata.clone());
        dataset_metadata
    }

    /// Preloads boundary chunks for 1D coordinate arrays (e.g. lat, lon, time, depth) associated with the dataset variables.
    pub async fn preload_coordinate_variables(&self, variables: &[VariableInfo]) {
        let coord_candidates =
            crate::data::backends::coord_bounds::collect_coordinate_candidates(variables);

        for coord_name in &coord_candidates {
            let clean = coord_name.trim().trim_start_matches('/').to_string();
            let mut paths_to_try = vec![format!("/{clean}")];
            for var in variables {
                if let Some(gp) = var.group_path() {
                    paths_to_try.push(format!("/{gp}/{clean}"));
                }
            }

            for cp in paths_to_try {
                let target_name = cp.trim_start_matches('/').to_string();
                if let Ok(coord_array) =
                    open_or_instantiate_array_normalized(self.memory_store.clone(), &cp)
                    && coord_array.shape().len() == 1
                {
                    let count = coord_array.shape().first().copied().unwrap_or(0);
                    self.preload_coordinate_chunks_1d(&target_name, count).await;
                    break;
                }
            }
        }
    }

    /// Preloads the chunks a 1D coordinate needs: every label of a text coordinate, else its
    /// first and last value.
    #[allow(clippy::single_range_in_vec_init)]
    pub async fn preload_coordinate_chunks_1d(&self, coord_name: &str, count: u64) {
        let path = format!("/{}", coord_name.trim_start_matches('/'));
        let array = open_or_instantiate_array_normalized(self.memory_store.clone(), &path).ok();
        for range in coordinate_preload_ranges(array.as_ref(), count) {
            let subset = ArraySubset::new_with_ranges(&[range]);
            let _ = Box::pin(self.preload_chunks_for_subset(coord_name, &subset, None)).await;
        }
    }
}
