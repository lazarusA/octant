//! Opening and closing the floating panels, by their toolbar buttons, close
//! buttons or shortcuts alike: keyboard focus enters a panel opened by keys
//! and returns to what opened it when it closes. Also the landing screen
//! shown in the plot's place.

use super::OctantApp;
use crate::ui::keyboard::regions::{self, Region};
use crate::ui::keyboard::{focus, windows};
use crate::ui::panel_layout::Panel;

impl OctantApp {
    /// Opens or closes `panel`. Closing the last of Variables and
    /// Dimensions drops the staged, unplotted selection.
    pub fn set_panel_open(&mut self, ctx: &egui::Context, panel: Panel, open: bool) {
        let shown = self.layout.panel_shown_mut(panel);
        if *shown == open {
            return;
        }
        *shown = open;
        if open {
            windows::remember_opener(ctx, panel.area_id());
            if focus::visible(ctx) {
                regions::enter_when_drawn(ctx, Region::of(panel));
            }
            return;
        }
        let exploring = matches!(panel, Panel::Variables | Panel::Dimensions);
        if exploring && !self.layout.show_variables_overlay && !self.layout.show_variable_controls {
            self.revert_selected_state_to_plotted();
        }
        windows::return_to_opener(ctx, panel.area_id());
    }

    /// Whether a plot has data to draw.
    pub fn has_plot(&self) -> bool {
        let data = &self.layers.base.data;
        data.matrix.is_some() || data.volume.is_some()
    }

    /// Whether the landing screen fills the canvas: asked for, or nothing
    /// is plotted yet.
    pub fn hero_shown(&self) -> bool {
        self.layout.show_hero || !self.has_plot()
    }

    /// Shows the landing screen, or the plot again.
    pub fn toggle_hero(&mut self) {
        self.layout.show_hero = !self.layout.show_hero;
    }

    pub fn toggle_panel(&mut self, ctx: &egui::Context, panel: Panel) {
        let open = !self.layout.panel_shown(panel);
        self.set_panel_open(ctx, panel, open);
    }
}
