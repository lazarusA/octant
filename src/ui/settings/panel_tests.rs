//! The settings panel grows with its body up to the canvas height.

use super::show_settings_window;
use crate::app::OctantApp;
use crate::plots::PlotType;
use egui::{Id, RawInput, Rect, pos2, vec2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 1400.0));

fn app(plot_type: PlotType) -> OctantApp {
    let mut app = OctantApp {
        show_settings_panel: true,
        ..Default::default()
    };
    app.selected.plot_type = plot_type;
    app
}

/// Panel height after a few frames (areas settle over two passes).
fn settled_height(ctx: &egui::Context, app: &mut OctantApp, canvas: Rect) -> f32 {
    for _ in 0..4 {
        let input = RawInput {
            screen_rect: Some(SCREEN),
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| show_settings_window(app, ui.ctx(), canvas));
        output.textures_delta.clear();
    }
    ctx.memory(|m| m.area_rect(Id::new("octant_settings_area")))
        .map_or(0.0, |r| r.height())
}

#[test]
fn panel_grows_when_its_body_gets_taller() {
    let canvas = SCREEN;
    let fresh = settled_height(
        &egui::Context::default(),
        &mut app(PlotType::Heatmap),
        canvas,
    );

    let ctx = egui::Context::default();
    let mut app = app(PlotType::Volume);
    let short = settled_height(&ctx, &mut app, canvas);
    app.selected.plot_type = PlotType::Heatmap;
    let grown = settled_height(&ctx, &mut app, canvas);

    assert!(
        fresh > short,
        "heatmap settings are taller: {fresh} vs {short}"
    );
    assert!(
        (grown - fresh).abs() < 1.0,
        "panel stuck at {grown}, expected {fresh}"
    );
}

#[test]
fn panel_stops_at_the_canvas_bottom() {
    let canvas = Rect::from_min_size(pos2(0.0, 0.0), vec2(1200.0, 200.0));
    let height = settled_height(
        &egui::Context::default(),
        &mut app(PlotType::Volume),
        canvas,
    );
    assert!(height <= canvas.height() - 8.0, "panel {height} overflows");
}

#[test]
fn a_plot_reveals_the_layers_menu_once() {
    let mut app = app(PlotType::Heatmap);
    app.plot_from_panel();
    assert!(app.reveal_layers_menu, "plotting asks for the Layers menu");
    settled_height(&egui::Context::default(), &mut app, SCREEN);
    assert!(!app.reveal_layers_menu, "the settings panel opened it");
}
