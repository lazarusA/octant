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
    pub fn set_panel_open(&mut self, panel: Panel, open: bool) {
        if !self.show_panel(panel, open) || open {
            return;
        }
        let exploring = matches!(panel, Panel::Variables | Panel::Dimensions);
        if exploring && !self.layout.show_variables_overlay && !self.layout.show_variable_controls {
            self.revert_selected_state_to_plotted();
        }
    }

    /// Opens or closes `panel`, keeping a staged selection (plotting closes
    /// the panels while its block loads); whether it changed. Keyboard focus
    /// follows when the chrome next draws (`follow_panel_focus`).
    pub fn show_panel(&mut self, panel: Panel, open: bool) -> bool {
        let shown = self.layout.panel_shown_mut(panel);
        if *shown == open {
            return false;
        }
        *shown = open;
        self.layout.panel_focus.push((panel, open));
        true
    }

    /// Focus enters each panel opened since the last call (when keys move
    /// it) and returns to what opened each one closed, in order.
    pub(crate) fn follow_panel_focus(&mut self, ctx: &egui::Context) {
        for (panel, open) in std::mem::take(&mut self.layout.panel_focus) {
            if !open {
                windows::return_to_opener(ctx, panel.area_id());
                continue;
            }
            windows::remember_opener(ctx, panel.area_id());
            if focus::visible(ctx) {
                regions::enter_when_drawn(ctx, Region::of(panel));
            }
        }
    }

    /// Takes keyboard focus to `panel`: opens it (focus entering it), moves
    /// focus into it when it is open, and closes it when focus already is.
    pub fn go_to_panel(&mut self, ctx: &egui::Context, panel: Panel) {
        if !self.layout.panel_shown(panel) {
            return self.set_panel_open(panel, true);
        }
        if focus::in_area(ctx, panel.area_id()) {
            return self.set_panel_open(panel, false);
        }
        windows::remember_opener(ctx, panel.area_id());
        regions::enter(ctx, Region::of(panel));
    }

    pub fn toggle_panel(&mut self, panel: Panel) {
        let open = !self.layout.panel_shown(panel);
        self.set_panel_open(panel, open);
    }
}
