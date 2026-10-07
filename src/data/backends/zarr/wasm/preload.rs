//! Coordinate preloading and metadata finalization for WebAssembly Zarr.

use super::WasmZarrBlockStore;
use crate::data::backends::coord_bounds::{CoordPreload, settle_preloaded_coordinates};
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
        let preloads = self.preload_coordinate_variables(&variables).await;
        let mut dimension_coordinates =
            crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables(
                self.memory_store.clone(),
                &variables,
                Some(clean_url),
            );
        settle_preloaded_coordinates(&mut dimension_coordinates, &preloads);

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

    /// Preloads every chunk of the 1D coordinate arrays (e.g. lat, lon, time, depth) of the
    /// dataset variables, at the root and in each variable's group. Returns the arrays whose
    /// chunks did not all arrive.
    pub async fn preload_coordinate_variables(
        &self,
        variables: &[VariableInfo],
    ) -> Vec<(String, CoordPreload)> {
        let coord_candidates =
            crate::data::backends::coord_bounds::collect_coordinate_candidates(variables);
        let mut incomplete = Vec::new();
        for coord_name in &coord_candidates {
            let clean = coord_name.trim().trim_start_matches('/').to_string();
            let mut paths_to_try = vec![format!("/{clean}")];
            for gp in variables.iter().filter_map(VariableInfo::group_path) {
                let path = format!("/{gp}/{clean}");
                if !paths_to_try.contains(&path) {
                    paths_to_try.push(path);
                }
            }
            for cp in paths_to_try {
                let target_name = cp.trim_start_matches('/').to_string();
                if let Ok(coord_array) =
                    open_or_instantiate_array_normalized(self.memory_store.clone(), &cp)
                    && coord_array.shape().len() == 1
                {
                    let count = coord_array.shape().first().copied().unwrap_or(0);
                    let state = self.preload_coordinate_chunks_1d(&target_name, count).await;
                    if state != CoordPreload::Complete {
                        incomplete.push((target_name, state));
                    }
                }
            }
        }
        incomplete
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
