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
                self.load_selected_variable_block();
            }
            AppAction::SetTimestep(step) => {
                self.current_timestep = step;
                self.load_selected_variable_block();
            }
            AppAction::SetPlotType(plot_type) => {
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
                self.is_playing = !self.is_playing;
            }
            AppAction::UpdateColorBounds { min, max } => {
                self.color_range_min = min;
                self.color_range_max = max;
            }
        }
    }
}
