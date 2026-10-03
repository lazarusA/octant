//! Color-encoding icons: the heatmap plot type and the colormap brush.

use super::canvas::{IconCanvas, Weight, key};
use super::style::IconTone;
use egui::{Color32, Pos2};

/// Opacity of each heatmap cell, top-left to bottom-right, so the 2x2 grid
/// reads as a diagonal value gradient in the icon's own color.
const HEATMAP_CELL_ALPHA: [f32; 4] = [0.95, 0.62, 0.40, 0.18];

/// Paint stroke tones laid down by the colormap brush, left to right.
const BRUSH_TONES: [IconTone; 5] = [
    IconTone::Accent,
    IconTone::Info,
    IconTone::Success,
    IconTone::Warning,
    IconTone::Error,
];

/// PlotPlane (heatmap): rounded frame holding a 2x2 grid of shaded value cells (18-unit square).
pub fn draw_plot_plane(c: &IconCanvas) {
    let (lo, hi) = (key::SQ_MIN, key::SQ_MAX);
    c.rrect((lo, lo), (hi, hi), 2.5, c.body(), c.stroke(Weight::Base));

    let cells = [(7.0, 7.0), (13.0, 7.0), (7.0, 13.0), (13.0, 13.0)];
    for ((x, y), alpha) in cells.into_iter().zip(HEATMAP_CELL_ALPHA) {
        c.rrect_fill(
            (x, y),
            (x + 4.0, y + 4.0),
            0.75,
            c.color.gamma_multiply(alpha),
        );
    }
}

/// Colormap: diagonal paintbrush laying a five-tone semantic paint stroke (20-unit square).
pub fn draw_colormap(c: &IconCanvas) {
    // Follow the caller's opacity so dimmed buttons also dim the paint.
    let opacity = f32::from(c.color.a()) / 255.0;
    let tone = |t: IconTone| t.rgb(c.is_dark).unwrap_or(c.color).gamma_multiply(opacity);
    draw_paint_stroke(c, tone);
    draw_brush(c, tone(IconTone::Accent));
}

/// Gently arched paint band split into one segment per [`BRUSH_TONES`] entry.
fn draw_paint_stroke(c: &IconCanvas, tone: impl Fn(IconTone) -> Color32) {
    let (x_start, x_end, half) = (3.5_f32, 20.5_f32, 1.6_f32);
    let center_y = |x: f32| 19.8 - (std::f32::consts::PI * (x - x_start) / (x_end - x_start)).sin();
    let seg_w = (x_end - x_start) / BRUSH_TONES.len() as f32;

    for (i, t) in BRUSH_TONES.into_iter().enumerate() {
        let x0 = x_start + i as f32 * seg_w;
        let x1 = x0 + seg_w;
        let pts = vec![
            c.p(x0, center_y(x0) - half),
            c.p(x1, center_y(x1) - half),
            c.p(x1, center_y(x1) + half),
            c.p(x0, center_y(x0) + half),
        ];
        c.fill_pts(pts, tone(t));
    }

    // Rounded end caps in the first and last tones.
    c.dot((x_start, center_y(x_start)), half, tone(BRUSH_TONES[0]));
    c.dot((x_end, center_y(x_end)), half, tone(BRUSH_TONES[4]));
}

/// Brush from the handle end (top-right) to the bristle tip resting on the paint.
fn draw_brush(c: &IconCanvas, paint: Color32) {
    let (start, tip) = ((20.5_f32, 3.5_f32), (6.0_f32, 18.0_f32));
    // Point at fraction `t` along the brush axis, offset `off` units across it.
    let at = |t: f32, off: f32| -> Pos2 {
        let n = std::f32::consts::FRAC_1_SQRT_2;
        let x = start.0 + (tip.0 - start.0) * t + off * n;
        let y = start.1 + (tip.1 - start.1) * t + off * n;
        c.p(x, y)
    };

    // Tapered handle with a rounded end.
    c.fill_pts(
        vec![at(0.0, 1.2), at(0.44, 1.6), at(0.44, -1.6), at(0.0, -1.2)],
        c.color,
    );
    c.dot(start, 1.2, c.color);

    // Metal ferrule, set off from the handle by a hairline gap.
    let ferrule = vec![at(0.47, 2.4), at(0.60, 2.4), at(0.60, -2.4), at(0.47, -2.4)];
    c.fill_pts(ferrule, c.color.gamma_multiply(0.55));

    // Paint-loaded bristles bulging out of the ferrule and tapering to the tip.
    let bristles = vec![
        at(0.60, 2.4),
        at(0.76, 2.8),
        at(1.0, 0.0),
        at(0.76, -2.8),
        at(0.60, -2.4),
    ];
    c.fill_pts(bristles, paint);
}
