use crate::plots::PlotType;

use super::OctantApp;
use super::state::StoreKind;

#[derive(Debug, Clone)]
pub enum AppAction {
    InspectActiveStore,
    SelectStore { kind: StoreKind, target: String },
    SelectVariable(usize),
    SetTimestep(usize),
    SetPlotType(PlotType),
    SetColormap(u32),
    SetLineProfileDim(usize),
    SetLineProfileSlice(usize),
    ToggleLineAllSeries,
    TogglePlayback,
    UpdateColorBounds { min: f32, max: f32 },
}

impl OctantApp {
    /// Action dispatch handler for event-driven app state mutations.
    pub fn dispatch(&mut self, action: AppAction) {
        match action {
            AppAction::InspectActiveStore => self.inspect_active_store(),
            AppAction::SelectStore { kind, target } => {
                self.selected_store_kind = kind;
                self.store_target_input = target;
                self.inspect_active_store();
            }
            AppAction::SelectVariable(idx) => {
                self.selected_variable_idx = idx;
                if let Some(meta) = self.active_dataset_metadata.clone()
                    && let Some(var_info) = meta.variables.get(idx).cloned()
                {
                    crate::ui::variables_panel::init_variable_dimension_defaults(self, &var_info);
                    self.plotted_store_kind = self.selected_store_kind;
                    self.plotted_store_target_input = self.store_target_input.clone();
                    self.plotted_dataset_metadata = Some(meta);
                    self.plotted_variable_idx = idx;
                    self.plotted_dim_config = self.dim_config.clone();
                    self.plotted_selected_dim_indices = self.selected_dim_indices.clone();
                    self.plotted_selected_dim_ranges = self.selected_dim_ranges.clone();
                    self.plotted_spatial_dims = self.spatial_dims.clone();
                    self.plotted_animated_dim = self.animated_dim;
                    self.reset_variable_bounds();
                }
                self.load_selected_variable_block();
            }
            AppAction::SetTimestep(step) => {
                self.current_timestep = step;
                self.load_selected_variable_block();
            }
            AppAction::SetPlotType(plot_type) => {
                if plot_type == PlotType::Volume || plot_type == PlotType::PointCloud {
                    self.is_playing = false;
                }
                self.active_plot_type = plot_type;
            }
            AppAction::SetColormap(cmap) => {
                self.active_colormap = cmap;
            }
            AppAction::SetLineProfileDim(dim_idx) => {
                self.line_profile_dim_idx = dim_idx;
            }
            AppAction::SetLineProfileSlice(slice_idx) => {
                self.line_profile_slice_idx = slice_idx;
            }
            AppAction::ToggleLineAllSeries => {
                self.line_plot_all_series = !self.line_plot_all_series;
            }
            AppAction::TogglePlayback => {
                let is_3d = self.active_plot_type == PlotType::Volume
                    || self.active_plot_type == PlotType::PointCloud;
                if is_3d {
                    self.is_playing = false;
                } else {
                    self.is_playing = !self.is_playing;
                }
            }
            AppAction::UpdateColorBounds { min, max } => {
                self.color_range_min = min;
                self.color_range_max = max;
            }
        }
    }
}
