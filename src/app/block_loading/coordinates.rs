//! A variable's dimension coordinates, read in the background when it is chosen (datasets
//! open with their metadata alone) and merged into every copy of its dataset's metadata.

use std::collections::HashMap;

use crate::app::OctantApp;
use crate::app::layers::{LayerId, VariableSelection};
use crate::data::{CoordValues, StoreHandle, VariableInfo};

impl OctantApp {
    /// Reads the coordinates of variable `idx` of the selected dataset in the background,
    /// once per variable.
    pub fn request_variable_coordinates(&mut self, idx: usize) {
        if let Some((source_id, store, variable)) = self.coordinate_request(&self.selected, idx) {
            self.coordinate_loader.request(&source_id, store, variable);
        }
    }

    /// Reads the coordinates of layer `id`'s staged variable in the background,
    /// once per variable (the base layer's is the selected one).
    pub(crate) fn request_layer_coordinates(&mut self, id: LayerId) {
        let request = self
            .layer_selections(id)
            .and_then(|(_, staged)| self.coordinate_request(staged, staged.variable_idx));
        if let Some((source_id, store, variable)) = request {
            self.coordinate_loader.request(&source_id, store, variable);
        }
    }

    /// The dataset, open store and variable `idx` of `selection`, whose
    /// coordinates a request reads.
    fn coordinate_request(
        &self,
        selection: &VariableSelection,
        idx: usize,
    ) -> Option<(String, StoreHandle, VariableInfo)> {
        let variable = selection.metadata.as_ref()?.variables.get(idx)?.clone();
        let source_id = self.selection_source_id(selection);
        let store =
            self.resolve_store_handle(&source_id, &selection.store_target, selection.store_kind)?;
        Some((source_id, store, variable))
    }

    /// Merges coordinates that arrived since the last frame.
    pub fn poll_coordinate_results(&mut self) {
        for result in self.coordinate_loader.poll() {
            match result.coords {
                Ok(coords) => self.merge_variable_coordinates(&result.source_id, coords),
                Err(e) => self.notify(
                    crate::ui::toast::Severity::Warning,
                    "Coordinates unavailable",
                    format!(
                        "Coordinates of '{}' failed to load ({e}); axes show indices",
                        result.variable
                    ),
                ),
            }
        }
    }

    /// Adds `coords` to the metadata of the dataset `source_id`: the selected copy, each
    /// layer's and the stored one, whichever belong to it.
    pub(crate) fn merge_variable_coordinates(
        &mut self,
        source_id: &str,
        coords: HashMap<String, CoordValues>,
    ) {
        if coords.is_empty() {
            return;
        }
        let fallback = self.selected_source_id();
        let selected = fallback == source_id;
        let mut copies: Vec<&mut crate::data::DatasetMetadata> = Vec::new();
        if selected && let Some(meta) = self.selected.metadata.as_mut() {
            copies.push(meta);
        }
        for layer in self.layers.iter_mut() {
            let selection = layer.selection_mut();
            if super::layer_request::source_id_or(selection, &fallback) == source_id
                && let Some(meta) = selection.metadata.as_mut()
            {
                copies.push(meta);
            }
        }
        if let Some(meta) = self
            .dataset_manager
            .get_mut(source_id)
            .and_then(|d| d.metadata.as_mut())
        {
            copies.push(meta);
        }
        for meta in copies {
            meta.dimension_coordinates
                .extend(coords.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        self.coordinates_revision = self.coordinates_revision.wrapping_add(1);
    }
}
