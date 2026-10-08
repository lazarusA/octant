//! Session lifecycle, view resets, color ranges, and plotted synchronization.

use super::app_state::OctantApp;
use super::store_kind::StoreKind;

impl OctantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
        let mut app = Self {
            wgpu_render_state: cc.wgpu_render_state.clone(),
            ..Default::default()
        };
        app.load_colormap_prefs(cc.storage);

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
        self.layers.base.default_colorbar_label()
    }

    /// Returns the effective colorbar label (custom overridden label if set, otherwise default).
    pub fn colorbar_label(&self) -> String {
        self.layers.base.colorbar_label()
    }

    /// Resets custom colorbar label back to default.
    pub fn reset_colorbar_label(&mut self) {
        self.layers.base.color.custom_label = None;
    }

    /// Resets color range min and max to the current dataset/matrix slice bounds and unlocks bounds.
    pub fn reset_color_range(&mut self) {
        let canvas_plot_type = self.effective_canvas_plot_type();
        let is_3d = canvas_plot_type == crate::plots::PlotType::Volume
            || canvas_plot_type == crate::plots::PlotType::PointCloud;

        let cur_var = self
            .plotted_variable_info()
            .or_else(|| self.selected_variable_info());
        let cur_name = cur_var.map(|v| v.name.as_str());

        let mdata_valid = self
            .layers
            .base
            .data
            .matrix
            .as_ref()
            .is_some_and(|m| cur_name.is_none_or(|n| m.dataset_name.contains(n)));
        let vdata_valid = self
            .layers
            .base
            .data
            .volume
            .as_ref()
            .is_some_and(|v| cur_name.is_none_or(|n| v.dataset_name.contains(n)));

        if is_3d
            && vdata_valid
            && let Some(vdata) = &self.layers.base.data.volume
        {
            self.layers.base.color.range_min = vdata.min_val;
            self.layers.base.color.range_max = vdata.max_val;
        } else if mdata_valid && let Some(mdata) = &self.layers.base.data.matrix {
            self.layers.base.color.range_min = mdata.min_val;
            self.layers.base.color.range_max = mdata.max_val;
        } else if vdata_valid && let Some(vdata) = &self.layers.base.data.volume {
            self.layers.base.color.range_min = vdata.min_val;
            self.layers.base.color.range_max = vdata.max_val;
        } else {
            self.layers.base.color.range_min = 0.0;
            self.layers.base.color.range_max = 100.0;
        }
        self.layers.base.color.lock_bounds = false;
    }

    /// Returns the source_id string for the currently plotted store.
    pub fn plotted_source_id(&self) -> String {
        if !self.plotted().store_target.is_empty() {
            StoreKind::make_source_id(self.plotted().store_kind, &self.plotted().store_target)
        } else {
            self.selected_source_id()
        }
    }

    /// Returns the source_id string for the currently selected (UI active) store.
    pub fn selected_source_id(&self) -> String {
        StoreKind::make_source_id(self.selected.store_kind, &self.selected.store_target)
    }

    /// Returns the effective plot type that is currently plotted and rendered on canvas.
    #[inline]
    pub fn effective_canvas_plot_type(&self) -> crate::plots::PlotType {
        if self.plotted().metadata.is_some() {
            self.plotted().plot_type
        } else {
            self.selected.plot_type
        }
    }

    /// Returns true if the user is currently browsing/configuring a variable or dataset that has not been plotted yet.
    #[inline]
    pub fn is_exploring_unplotted_variable(&self) -> bool {
        self.plotted().metadata.is_none()
            || self.plotted().variable_idx != self.selected.variable_idx
            || self.plotted().store_target != self.selected.store_target
    }

    /// Reverts current staged/selected UI configuration back to the plotted dataset and variable.
    pub fn revert_selected_state_to_plotted(&mut self) {
        if let Some(meta) = self.plotted().metadata.clone() {
            self.set_active_metadata(meta);
            self.copy_plotted_to_selected();
        }
    }

    /// Synchronizes all plotted configuration fields from the current UI selection.
    pub fn sync_plotted_state_from_selected(&mut self) {
        let is_new_var = self.is_exploring_unplotted_variable();
        self.copy_selected_to_plotted();

        if is_new_var {
            self.enable_pyramid_resampling = false;
            self.layers.base.data.pyramid = None;
            let is_vol = self.plotted().plot_type == crate::plots::PlotType::Volume
                || self.plotted().plot_type == crate::plots::PlotType::PointCloud;
            if is_vol {
                self.layers.base.clear_2d();
            } else {
                self.layers.base.clear_3d();
            }
            if let Some(var_info) = self.plotted_variable_info().cloned() {
                let rank = var_info.shape.len();
                crate::ui::variables_panel::dimension_slider::init_composite_defaults(
                    self, &var_info, rank,
                );
            }
        }

        if !self.has_rgb_bands() {
            self.layers.base.composite.enabled = false;
        }
        self.reset_variable_bounds();
    }

    /// Returns VariableInfo for the currently plotted variable, if available.
    pub fn plotted_variable_info(&self) -> Option<&crate::data::VariableInfo> {
        self.layer_variable_info(crate::app::layers::LayerId::BASE)
    }

    /// Returns VariableInfo for the currently selected variable, if available.
    pub fn selected_variable_info(&self) -> Option<&crate::data::VariableInfo> {
        self.selected.variable_info()
    }
}
