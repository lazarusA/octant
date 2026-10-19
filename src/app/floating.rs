//! The bars, modal windows and floating panels around the canvas, in the
//! order the frame needs. The keyboard tests draw the window through
//! `show_chrome` too, so they walk what the app draws.

use super::OctantApp;

impl OctantApp {
    /// Draws the top bar, the bottom bar (on a plot with an animated
    /// dimension; `plot_shown`: no landing screen), the modal windows, then
    /// the floating panels over what is left; the canvas rect.
    pub(crate) fn show_chrome(&mut self, ui: &mut egui::Ui, plot_shown: bool) -> egui::Rect {
        let ctx = ui.ctx().clone();
        // The bars consume space from the remaining area.
        crate::ui::top_bar::show_top_bar(self, ui);
        if plot_shown && self.has_animated_dimension() {
            crate::ui::bottom_bar::show_bottom_bar(self, ui);
        }
        crate::ui::catalog::show_catalog_window(self, &ctx);
        crate::ui::about::show_about_window(self, &ctx);
        crate::ui::about::show_icon_gallery_window(self, &ctx);
        crate::ui::export_modal::show_export_modal(self, &ctx);
        crate::ui::keyboard::help::show(self, &ctx);
        crate::ui::plot_type::picker::show_plot_type_picker(self, &ctx);
        // Overlays anchor relative to the remaining canvas rect.
        let canvas_rect = ui.available_rect_before_wrap();
        self.show_floating_panels(&ctx, canvas_rect, plot_shown);
        canvas_rect
    }

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
        crate::ui::dataset::show_dataset_window(self, ctx, canvas_rect);
        crate::ui::variables_overlay::show_variables_overlay(self, ctx, canvas_rect);
        crate::ui::settings::show_settings_window(self, ctx, canvas_rect);
        crate::ui::variables_panel::show_variable_controls(self, ctx, canvas_rect);
        if colorbars {
            crate::ui::colorbar::show_colorbar_overlay(self, ctx, canvas_rect);
        }
    }
}
