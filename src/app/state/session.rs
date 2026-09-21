//! Session lifecycle, view resets, color ranges, and plotted synchronization.

use super::app_state::OctantApp;
use super::layer_state::PlottedVariableState;
use super::store_kind::StoreKind;

impl OctantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
        let mut app = Self {
            wgpu_render_state: cc.wgpu_render_state.clone(),
            ..Default::default()
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(target) = std::env::args().nth(1) {
            app.submit_or_activate_source(&target, None);
        }

        app
    }

    pub fn reset_heatmap_view(&mut self) {
        self.heatmap_zoom = 1.0;
        self.heatmap_pan = egui::Vec2::ZERO;
    }

    pub fn reset_line_view(&mut self) {
        self.line_zoom = 1.0;
        self.line_pan = egui::Vec2::ZERO;
    }

    /// Returns the default automatic label for the active plotted variable (including unit if available).
    pub fn default_colorbar_label(&self) -> String {
        if let Some(meta) = &self.plotted_dataset_metadata {
            meta.variables
                .get(self.plotted_variable_idx)
                .map(|v| {
                    if let Some(unit) = v.attributes.get("units").or(v.units.as_ref()) {
                        format!("{} ({})", v.name, unit)
                    } else {
                        v.name.clone()
                    }
                })
                .unwrap_or_else(|| "Scalar Field".to_string())
        } else {
            "Scalar Field".to_string()
        }
    }

    /// Returns the effective colorbar label (custom overridden label if set, otherwise default).
    pub fn colorbar_label(&self) -> String {
        if let Some(ref custom) = self.custom_colorbar_label {
            custom.clone()
        } else {
            self.default_colorbar_label()
        }
    }

    /// Resets custom colorbar label back to default.
    pub fn reset_colorbar_label(&mut self) {
        self.custom_colorbar_label = None;
    }

    /// Resets color range min and max to the current dataset/matrix slice bounds and unlocks bounds.
    pub fn reset_color_range(&mut self) {
        let is_3d = self.active_plot_type == crate::plots::PlotType::Volume
            || self.active_plot_type == crate::plots::PlotType::PointCloud;

        if is_3d && let Some(vdata) = &self.volume_data {
            self.color_range_min = vdata.min_val;
            self.color_range_max = vdata.max_val;
            self.volume_cmin = vdata.min_val;
            self.volume_cmax = vdata.max_val;
        } else if let Some(mdata) = &self.matrix_data {
            self.color_range_min = mdata.min_val;
            self.color_range_max = mdata.max_val;
            self.volume_cmin = mdata.min_val;
            self.volume_cmax = mdata.max_val;
        } else if let Some(vdata) = &self.volume_data {
            self.color_range_min = vdata.min_val;
            self.color_range_max = vdata.max_val;
            self.volume_cmin = vdata.min_val;
            self.volume_cmax = vdata.max_val;
        } else {
            self.color_range_min = 0.0;
            self.color_range_max = 100.0;
            self.volume_cmin = 0.0;
            self.volume_cmax = 100.0;
        }
        self.lock_color_bounds = false;
    }

    /// Placeholder method to add a secondary dimensionally-compatible variable layer
    /// for multi-variable plotting (e.g., vector fields, RGB composites, dual-curves).
    pub fn add_plotted_layer(&mut self, layer: PlottedVariableState) -> Result<(), String> {
        if let (Some(existing_meta), Some(new_meta)) =
            (&self.plotted_dataset_metadata, &layer.dataset_metadata)
        {
            let existing_var = existing_meta.variables.get(self.plotted_variable_idx);
            let new_var = new_meta.variables.get(layer.variable_idx);

            if let (Some(v_a), Some(v_b)) = (existing_var, new_var) {
                super::dataset_activation::check_dimensional_compatibility(v_a, v_b)?;
            }
        }
        self.multi_plotted_layers.push(layer);
        Ok(())
    }

    /// Clears secondary multi-variable layers.
    pub fn clear_plotted_layers(&mut self) {
        self.multi_plotted_layers.clear();
    }

    /// Returns the source_id string for the currently plotted store.
    pub fn plotted_source_id(&self) -> String {
        StoreKind::make_source_id(self.plotted_store_kind, &self.plotted_store_target_input)
    }

    /// Returns the source_id string for the currently selected (UI active) store.
    pub fn selected_source_id(&self) -> String {
        StoreKind::make_source_id(self.selected_store_kind, &self.store_target_input)
    }

    /// Synchronizes all plotted configuration fields from the current UI selection.
    pub fn sync_plotted_state_from_selected(&mut self) {
        self.plotted_store_kind = self.selected_store_kind;
        self.plotted_store_target_input = self.store_target_input.clone();
        self.plotted_dataset_metadata = self.active_dataset_metadata.clone();
        self.plotted_variable_idx = self.selected_variable_idx;
        self.plotted_dim_config = self.dim_config.clone();
        self.plotted_selected_dim_indices = self.selected_dim_indices.clone();
        self.plotted_selected_dim_ranges = self.selected_dim_ranges.clone();
        self.plotted_spatial_dims = self.spatial_dims.clone();
        self.plotted_animated_dim = self.animated_dim;
        if !self.has_rgb_bands() {
            self.rgb_composite_mode = false;
            if self.active_colormap == 1000 {
                self.active_colormap = 0;
            }
        }
        self.reset_variable_bounds();
    }

    /// Returns VariableInfo for the currently plotted variable, if available.
    pub fn plotted_variable_info(&self) -> Option<&crate::data::VariableInfo> {
        self.plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.plotted_variable_idx))
    }

    /// Returns VariableInfo for the currently selected variable, if available.
    pub fn selected_variable_info(&self) -> Option<&crate::data::VariableInfo> {
        self.active_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.selected_variable_idx))
    }

    /// Returns the chunk size along `dim` for the currently plotted variable (defaults to 1).
    pub fn plotted_chunk_size(&self, dim: usize) -> usize {
        self.plotted_variable_info()
            .and_then(|v| v.chunk_shape.get(dim))
            .copied()
            .unwrap_or(1) as usize
    }

    /// Returns the total extent along `dim` for the currently plotted variable (defaults to 1).
    pub fn plotted_dim_size(&self, dim: usize) -> usize {
        self.plotted_variable_info()
            .and_then(|v| v.shape.get(dim))
            .copied()
            .unwrap_or(1) as usize
    }
}
