//! The whole window for keyboard tests: drawn in the app's order over a
//! plotted, animated heatmap, with a stand-in for the canvas (painting it
//! needs a GPU).

use super::regions::{self, Region};
use super::{dispatch, focus};
use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::panel_layout::Panel;
use crate::ui::panel_width_tests::plotted;
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Rect, Ui, pos2};

pub(super) const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1600.0, 1000.0));

pub(super) fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        let ctx = ui.ctx().clone();
        dispatch(app, &ctx, true);
        crate::ui::top_bar::show_top_bar(app, ui);
        crate::ui::bottom_bar::show_bottom_bar(app, ui);
        let canvas = ui.available_rect_before_wrap();
        crate::ui::dataset::show_dataset_window(app, &ctx, canvas);
        crate::ui::variables_overlay::show_variables_overlay(app, &ctx, canvas);
        crate::ui::settings::show_settings_window(app, &ctx, canvas);
        crate::ui::variables_panel::show_variable_controls(app, &ctx, canvas);
        crate::ui::colorbar::show_colorbar_overlay(app, &ctx, canvas);
        let id = crate::app::canvas::canvas_id();
        let response = ui.interact(canvas, id, egui::Sense::drag());
        regions::register(&ctx, Region::Canvas, id, response.layer_id, canvas);
        focus::end_frame(&ctx);
    }
}

/// The plotted heatmap of `t2m` shown (no landing screen), its panels open
/// or not, and the window settled over it.
pub(super) fn window(panels_open: bool) -> (OctantApp, Harness) {
    let mut app = plotted(PlotType::Heatmap);
    app.layout.show_hero = false;
    for panel in Panel::ALL {
        *app.layout.panel_shown_mut(panel) = panels_open;
    }
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    (app, h)
}

/// Presses `key` with `modifiers`; where focus ended up.
pub(super) fn press(
    h: &mut Harness,
    app: &mut OctantApp,
    key: Key,
    modifiers: Modifiers,
) -> Option<focus::LastFocus> {
    h.press(key, modifiers, &mut run(app))
}
