//! The floating panels over the canvas, in the order the frame needs.

use super::OctantApp;

impl OctantApp {
    /// Draws the Variables, Settings and Dimensions panels, then (unless
    /// `colorbars` is off, on the landing screen) the colorbars: after the
    /// panels, so a colormap hovered in their pickers this frame previews on
    /// its layer's colorbar as on the plot. Colorbars sit in the middle
    /// order, under the panels, whatever the call order.
    pub(super) fn show_floating_panels(
        &mut self,
        ctx: &egui::Context,
        canvas_rect: egui::Rect,
        colorbars: bool,
    ) {
        crate::ui::variables_overlay::show_variables_overlay(self, ctx, canvas_rect);
        crate::ui::settings::show_settings_window(self, ctx, canvas_rect);
        crate::ui::variables_panel::show_variable_controls(self, ctx, canvas_rect);
        if colorbars {
            crate::ui::colorbar::show_colorbar_overlay(self, ctx, canvas_rect);
        }
    }
}
