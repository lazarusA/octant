//! Session lifecycle, view resets, color ranges, and plotted synchronization.

use super::app_state::OctantApp;
use super::store_kind::StoreKind;
use crate::app::layers::LayerId;

const LANGUAGE_STORAGE_KEY: &str = "octant_language";

impl OctantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
        let mut app = Self {
            wgpu_render_state: cc.wgpu_render_state.clone(),
            ..Default::default()
        };
        if let Some(rs) = &cc.wgpu_render_state {
            crate::plots::limits::set_from_device(&rs.device.limits());
        }
        app.load_language_pref(cc.storage);
        crate::ui::fonts::FontEngine::init(&cc.egui_ctx);
        crate::ui::number_input::install(&cc.egui_ctx);
        crate::ui::theme::install(&cc.egui_ctx);
        // A CRS definition fetched on the web repaints the frame waiting for it.
        #[cfg(target_arch = "wasm32")]
        {
            let ctx = cc.egui_ctx.clone();
            crate::data::crs::set_wake(move || ctx.request_repaint());
        }
        app.load_colormap_prefs(cc.storage);

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(target) = std::env::args().nth(1) {
            app.submit_or_activate_source(&target, None);
        }

        app
    }

    /// Restores the user's language preference from eframe storage; on the
    /// first run (nothing saved yet), starts in the system's language when
    /// Octant has it, else English.
    pub fn load_language_pref(&mut self, storage: Option<&dyn eframe::Storage>) {
        let saved = storage.and_then(|storage| {
            eframe::get_value::<crate::i18n::Language>(storage, LANGUAGE_STORAGE_KEY)
        });
        if let Some(lang) = saved.or_else(crate::i18n::locale::system_language) {
            crate::i18n::set_language(lang);
        }
    }

    /// Persists the user's language preference to eframe storage.
    pub fn save_language_pref(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, LANGUAGE_STORAGE_KEY, &crate::i18n::language());
    }

    pub fn reset_heatmap_view(&mut self) {
        self.nav.reset_heatmap();
    }

    pub fn reset_line_view(&mut self) {
        self.nav.reset_line();
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

    /// Resets the base layer's color range to its data extent and unlocks it.
    pub fn reset_color_range(&mut self) {
        self.reset_layer_color_range(LayerId::BASE);
    }

    /// Resets layer `id`'s color range min and max to its current slice or
    /// volume extent (0..100 without data of its variable) and unlocks it.
    pub fn reset_layer_color_range(&mut self, id: LayerId) {
        // Only the base layer can be a 3D plot; overlays are heatmaps.
        let is_3d = id == LayerId::BASE
            && matches!(
                self.effective_canvas_plot_type(),
                crate::plots::PlotType::Volume | crate::plots::PlotType::PointCloud
            );
        let cur_name = self.layer_variable_info(id).map(|v| v.name.clone());
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let of_var = |name: &str| cur_name.as_deref().is_none_or(|n| name.contains(n));
        let matrix = layer
            .data
            .matrix
            .as_ref()
            .filter(|m| of_var(&m.dataset_name))
            .map(|m| (m.min_val, m.max_val));
        let volume = layer
            .data
            .volume
            .as_ref()
            .filter(|v| of_var(&v.dataset_name))
            .map(|v| (v.min_val, v.max_val));
        let extent = if is_3d {
            volume.or(matrix)
        } else {
            matrix.or(volume)
        };
        let (min, max) = extent.unwrap_or((0.0, 100.0));
        layer.color.range_min = min;
        layer.color.range_max = max;
        layer.color.lock_bounds = false;
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
        let Some(meta) = self.plotted().metadata.as_ref() else {
            return true;
        };
        let (shown, staged) = (self.plotted().variable_idx, self.selected.variable_idx);
        !crate::app::multiscale::same_variable(meta, shown, staged)
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
        self.sync_plotted_state_for(None);
    }

    /// [`Self::sync_plotted_state_from_selected`] for the block of variable
    /// `shown` (a level of a multiscale stack) that arrived.
    pub(crate) fn sync_plotted_state_for(&mut self, shown: Option<&str>) {
        let is_new_var = self.is_exploring_unplotted_variable();
        self.copy_selected_to_plotted(shown);

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
                crate::ui::variables_panel::dimension_slider::init_composite_defaults(
                    self, &var_info,
                );
            }
        }

        if !self.has_rgb_bands() {
            self.layers.base.composite.enabled = false;
        }
        self.reset_variable_bounds();
        self.sync_overlays_to_base();
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
