//! Closing datasets, and what a closed dataset lets go: its blocks and its
//! stores' memory, once no other dataset, plotted layer or staged selection
//! reads its source.

use super::app_state::OctantApp;
use super::dataset_activation::same_target;

impl OctantApp {
    /// Removes a dataset from `dataset_manager` by ID.
    /// If it is currently active, resets active dataset metadata and variable tree.
    pub fn remove_dataset(&mut self, dataset_id: &str) {
        if let Some(removed) = self.dataset_manager.remove(dataset_id) {
            crate::data::backends::coord_bounds::evict_coord_values(Some(&removed.source.uri));
            self.coordinate_loader.forget(&removed.id);
            let is_active = self.selected.metadata.is_some()
                && same_target(&self.selected.store_target, &removed.source.uri);
            if is_active {
                self.clear_active_metadata();
            }
            // The plate browser goes with its plate, not with one of its fields.
            if self
                .plate
                .as_ref()
                .is_some_and(|p| p.url == removed.source.uri.trim_end_matches('/'))
            {
                self.plate = None;
            }
            self.release_dataset_memory(&removed.source.id, &removed.source.uri);
            self.status_message = format!("Removed dataset '{}'", removed.source.display_name);
        }
    }

    /// Clears all datasets from `dataset_manager` and resets active dataset state.
    pub fn clear_all_datasets(&mut self) {
        let sources: Vec<(String, String)> = self
            .dataset_manager
            .iter()
            .map(|d| (d.source.id.clone(), d.source.uri.clone()))
            .collect();
        self.dataset_manager.clear();
        crate::data::backends::coord_bounds::evict_coord_values(None);
        self.coordinate_loader = crate::data::blocks::CoordinateLoader::default();
        self.clear_active_metadata();
        self.plate = None;
        for (id, uri) in &sources {
            self.release_dataset_memory(id, uri);
        }
        self.status_message = "Cleared all datasets from Dataset Manager".to_string();
    }

    /// Drops the blocks of `source_id` and the stores of `uri` once nothing
    /// reads them.
    pub(super) fn release_dataset_memory(&mut self, source_id: &str, uri: &str) {
        let reads = |target: &str| same_target(target, uri);
        let in_use = self.dataset_manager.iter().any(|d| reads(&d.source.uri))
            || self
                .layers
                .iter()
                .any(|l| reads(&l.selection().store_target))
            || (self.selected.metadata.is_some() && reads(&self.selected.store_target));
        if in_use {
            return;
        }
        self.block_cache.remove_source(source_id);
        crate::data::backends::release_stores(uri);
    }
}
