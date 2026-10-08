//! A variable's dimension coordinates, read in the background when it is chosen (datasets
//! open with their metadata alone) and merged into every copy of its dataset's metadata.

use std::collections::HashMap;

use crate::app::OctantApp;
use crate::data::CoordValues;

impl OctantApp {
    /// Reads the coordinates of variable `idx` of the selected dataset in the background,
    /// once per variable.
    pub fn request_variable_coordinates(&mut self, idx: usize) {
        let Some(variable) = self
            .selected
            .metadata
            .as_ref()
            .and_then(|m| m.variables.get(idx))
            .cloned()
        else {
            return;
        };
        let Some(store) = self.selected_store_handle() else {
            return;
        };
        let source_id = self.selected_source_id();
        self.coordinate_loader.request(&source_id, store, variable);
    }

    /// Merges coordinates that arrived since the last frame.
    pub fn poll_coordinate_results(&mut self) {
        for result in self.coordinate_loader.poll() {
            match result.coords {
                Ok(coords) => self.merge_variable_coordinates(&result.source_id, coords),
                Err(e) => log::warn!("Coordinates of '{}' failed to load: {e}", result.variable),
            }
        }
    }

    /// Adds `coords` to the metadata of the dataset `source_id`: the selected, the plotted
    /// and the stored copy, whichever belong to it.
    pub(crate) fn merge_variable_coordinates(
        &mut self,
        source_id: &str,
        coords: HashMap<String, CoordValues>,
    ) {
        if coords.is_empty() {
            return;
        }
        let selected = self.selected_source_id() == source_id;
        let plotted = self.plotted_source_id() == source_id;
        let stored = self
            .dataset_manager
            .get_mut(source_id)
            .and_then(|d| d.metadata.as_mut());
        let copies = [
            selected
                .then_some(self.selected.metadata.as_mut())
                .flatten(),
            plotted
                .then_some(self.layers.base.selection_mut().metadata.as_mut())
                .flatten(),
            stored,
        ];
        for meta in copies.into_iter().flatten() {
            meta.dimension_coordinates
                .extend(coords.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        self.coordinates_revision = self.coordinates_revision.wrapping_add(1);
    }
}
