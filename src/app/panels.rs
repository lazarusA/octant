//! Opening and closing the floating panels, by their toolbar buttons, close
//! buttons or shortcuts alike: keyboard focus enters a panel opened by keys
//! and returns to what opened it when it closes.

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

    /// Takes keyboard focus to `panel`: opens it (focus entering it), moves
    /// focus into it when it is open, and closes it when focus already is.
    pub fn go_to_panel(&mut self, ctx: &egui::Context, panel: Panel) {
        if !self.layout.panel_shown(panel) {
            return self.set_panel_open(ctx, panel, true);
        }
        if focus::in_area(ctx, panel.area_id()) {
            return self.set_panel_open(ctx, panel, false);
        }
        windows::remember_opener(ctx, panel.area_id());
        regions::enter(ctx, Region::of(panel));
    }

    pub fn toggle_panel(&mut self, ctx: &egui::Context, panel: Panel) {
        let open = !self.layout.panel_shown(panel);
        self.set_panel_open(ctx, panel, open);
    }
}
