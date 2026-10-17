//! The Layers menu's colormap button shows a hidden colorbar again.

use super::show_settings_window;
use crate::app::OctantApp;
use crate::i18n::{SettingsKey, tr};
use crate::ui::icons::{IconSize, ToolbarButton};
use crate::ui::test_input::Harness;
use egui::epaint::ClippedShape;
use egui::{Id, Pos2, Rect, Ui, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 1400.0));

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| show_settings_window(app, ui.ctx(), SCREEN)
}

/// The center of the painted text `label` in `shapes`.
fn text_center(shapes: &[ClippedShape], label: &str) -> Pos2 {
    shapes
        .iter()
        .find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) if text.galley.text() == label => {
                Some(text.pos + text.galley.size() * 0.5)
            }
            _ => None,
        })
        .expect("the Layers menu names the base layer")
}

/// The base layer's colormap button: right-aligned in its header row.
fn button_at(h: &Harness, shapes: &[ClippedShape]) -> Pos2 {
    let row_y = text_center(shapes, tr(SettingsKey::NoVariable)).y;
    let margin = egui::Frame::popup(&egui::Style::default())
        .total_margin()
        .right;
    let panel = h.area(Id::unique("octant_settings_area"));
    pos2(
        panel.max.x - margin - ToolbarButton::side(IconSize::Sm) / 2.0,
        row_y,
    )
}

#[test]
fn the_colormap_button_shows_a_hidden_colorbar() {
    let mut app = OctantApp::default();
    app.layout.show_settings_panel = true;
    app.layers.base.colorbar.shown = false;
    let mut h = Harness::new(SCREEN);
    let shapes = h.settle(4, &mut run(&mut app));
    h.click(button_at(&h, &shapes), &mut run(&mut app));
    assert!(app.layers.base.colorbar.shown);
}
