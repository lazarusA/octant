//! A window of the plotted quad, turned into the heatmap's zoom and pan,
//! shows that window again (contained on a canvas of another shape).

use super::heatmap_view_for;
use crate::data::ViewportResampler;

/// What `zoom` and `pan` (points) show on `canvas`, as `render_canvas` reads it.
fn shown(zoom: f32, pan: egui::Vec2, canvas: egui::Vec2, aspect: [f32; 2]) -> super::QuadWindow {
    let gpu = [pan.x / (0.5 * canvas.x), -pan.y / (0.5 * canvas.y)];
    ViewportResampler::compute_visible_data_bounds(gpu, zoom, aspect)
}

fn close(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4
}

#[test]
fn a_window_turns_into_the_view_that_shows_it() {
    let canvas = egui::vec2(800.0, 600.0);
    let aspect = [0.75, 1.0];
    // A window of the drawn quad's shape (`du * ax == dv * ay`): shown exactly.
    let window = ((0.2, 0.2 + 0.2 / 0.75), (0.5, 0.7));
    let (zoom, pan) = heatmap_view_for(window, canvas, aspect, (0.1, 1e4));
    let (u, v) = shown(zoom, pan, canvas, aspect);
    assert!(close(u, window.0) && close(v, window.1), "{u:?} {v:?}");
}

#[test]
fn another_shape_contains_the_window_about_its_centre() {
    let canvas = egui::vec2(800.0, 400.0);
    let aspect = [0.5, 1.0];
    let window = ((0.1, 0.3), (0.6, 0.8));
    let (zoom, pan) = heatmap_view_for(window, canvas, aspect, (0.1, 1e4));
    let (u, v) = shown(zoom, pan, canvas, aspect);
    assert!(u.0 <= 0.1 + 1e-6 && u.1 >= 0.3 - 1e-6, "{u:?}");
    assert!(v.0 <= 0.6 + 1e-6 && v.1 >= 0.8 - 1e-6, "{v:?}");
    assert!(((u.0 + u.1) / 2.0 - 0.2).abs() < 1e-4 && ((v.0 + v.1) / 2.0 - 0.7).abs() < 1e-4);
}

#[test]
fn the_zoom_stays_within_its_limits() {
    let canvas = egui::vec2(100.0, 100.0);
    let tiny = ((0.5, 0.5001), (0.5, 0.5001));
    assert_eq!(
        heatmap_view_for(tiny, canvas, [1.0, 1.0], (0.1, 50.0)).0,
        50.0
    );
    let whole = ((-10.0, 10.0), (-10.0, 10.0));
    assert_eq!(
        heatmap_view_for(whole, canvas, [1.0, 1.0], (0.1, 50.0)).0,
        0.1
    );
}
