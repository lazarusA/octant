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
                self.submit_or_activate_source(&target, Some(kind));
            }
            AppAction::SelectVariable(idx) => {
                self.layout.show_hero = false;
                self.selected.variable_idx = idx;
                self.request_variable_coordinates(idx);
                if let Some(meta) = self.selected.metadata.clone()
                    && let Some(var_info) = meta.variables.get(idx).cloned()
                {
                    crate::ui::variables_panel::init_variable_dimension_defaults(self, &var_info);
                }
                self.load_selected_variable_block();
            }
            AppAction::SetTimestep(step) => {
                self.playback.current_timestep = step;
                self.load_step_blocks();
            }
            AppAction::SetPlotType(plot_type) => {
                self.switch_plot_type(plot_type);
            }
            AppAction::SetColormap(cmap) => {
                self.layers.base.color.colormap = cmap;
            }
            AppAction::SetLineProfileDim(dim_idx) => {
                self.plot_configs.line.profile_dim_idx = dim_idx;
            }
            AppAction::SetLineProfileSlice(slice_idx) => {
                self.plot_configs.line.profile_slice_idx = slice_idx;
            }
            AppAction::ToggleLineAllSeries => {
                self.plot_configs.line.all_series = !self.plot_configs.line.all_series;
            }
            AppAction::TogglePlayback => {
                self.playback.is_playing = !self.playback.is_playing;
                if self.playback.is_playing {
                    self.prefetch_animated_ranges();
                }
            }

            AppAction::UpdateColorBounds { min, max } => {
                self.layers.base.color.range_min = min;
                self.layers.base.color.range_max = max;
            }
        }
    }
}
