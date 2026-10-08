//! The dataset listed in the Variables overlay: one place sets or clears it,
//! keeping the variable tree cache, the search and the load counter in step.

use super::app_state::OctantApp;
use crate::data::DatasetMetadata;

impl OctantApp {
    /// Make `meta` the active dataset: rebuild the variable tree and count a
    /// new load, so per-dataset caches and focus reset.
    pub fn set_active_metadata(&mut self, meta: DatasetMetadata) {
        self.cached_variable_tree = Some(meta.build_variable_tree());
        self.selected.metadata = Some(meta);
        self.bump_metadata_generation();
    }

    /// Activate a freshly loaded or re-activated dataset: clear the search,
    /// apply the first variable's dimension defaults and select it.
    pub fn load_new_metadata(&mut self, meta: DatasetMetadata) {
        self.variable_search.clear();
        if let Some(var_info) = meta.variables.first().cloned() {
            crate::ui::variables_panel::init_variable_dimension_defaults(self, &var_info);
        }
        self.set_active_metadata(meta);
        self.selected.variable_idx = 0;
    }

    /// Drop the active dataset (loading, failed, removed or cleared).
    pub fn clear_active_metadata(&mut self) {
        self.selected.metadata = None;
        self.cached_variable_tree = None;
        self.variable_search.clear();
        self.bump_metadata_generation();
    }

    fn bump_metadata_generation(&mut self) {
        self.selected.metadata_generation = self.selected.metadata_generation.wrapping_add(1);
        self.cached_search = None;
    }
}
