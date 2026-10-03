//! Color-encoding procedural vector icons: the heatmap plot type and the
//! colormap brush. Precision-engineered on the standard 24-unit keyline grid.

use super::grid_p;
use super::style::IconTone;
use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind};

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

/// PlotPlane (heatmap): Rounded frame holding a 2x2 grid of shaded value cells (18x18dp keyline).
pub fn draw_plot_plane(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);
    let unit = rect.width() / 24.0;

    let frame = Rect::from_min_max(p(3.0, 3.0), p(21.0, 21.0));
    painter.rect(frame, 2.5 * unit, fill, stroke, StrokeKind::Inside);

    let cells = [(7.0, 7.0), (13.0, 7.0), (7.0, 13.0), (13.0, 13.0)];
    for ((x, y), alpha) in cells.into_iter().zip(HEATMAP_CELL_ALPHA) {
        let cell = Rect::from_min_max(p(x, y), p(x + 4.0, y + 4.0));
        painter.rect_filled(cell, 0.75 * unit, stroke.color.gamma_multiply(alpha));
    }
}

/// Colormap: Diagonal paintbrush laying a five-tone semantic paint stroke (20x20dp keyline).
pub fn draw_colormap(painter: &Painter, rect: Rect, stroke: Stroke, is_dark: bool) {
    // Follow the caller's opacity so dimmed buttons also dim the paint.
    let opacity = f32::from(stroke.color.a()) / 255.0;
    let tone = |t: IconTone| {
        t.rgb(is_dark)
            .unwrap_or(stroke.color)
            .gamma_multiply(opacity)
    };
    draw_paint_stroke(painter, rect, tone);
    draw_brush(painter, rect, stroke.color, tone(IconTone::Accent));
}

/// Gently arched paint band split into one segment per [`BRUSH_TONES`] entry.
fn draw_paint_stroke(painter: &Painter, rect: Rect, tone: impl Fn(IconTone) -> Color32) {
    let (x_start, x_end, half) = (3.5_f32, 20.5_f32, 1.6_f32);
    let center_y = |x: f32| 19.8 - (std::f32::consts::PI * (x - x_start) / (x_end - x_start)).sin();
    let seg_w = (x_end - x_start) / BRUSH_TONES.len() as f32;

    for (i, t) in BRUSH_TONES.into_iter().enumerate() {
        let x0 = x_start + i as f32 * seg_w;
        let x1 = x0 + seg_w;
        let pts = vec![
            grid_p(rect, x0, center_y(x0) - half),
            grid_p(rect, x1, center_y(x1) - half),
            grid_p(rect, x1, center_y(x1) + half),
            grid_p(rect, x0, center_y(x0) + half),
        ];
        painter.add(egui::Shape::convex_polygon(pts, tone(t), Stroke::NONE));
    }

    // Rounded end caps in the first and last tones.
    let r = rect.width() * (half / 24.0);
    painter.circle_filled(
        grid_p(rect, x_start, center_y(x_start)),
        r,
        tone(BRUSH_TONES[0]),
    );
    painter.circle_filled(
        grid_p(rect, x_end, center_y(x_end)),
        r,
        tone(BRUSH_TONES[4]),
    );
}

/// Brush from the handle end (top-right) to the bristle tip resting on the paint.
fn draw_brush(painter: &Painter, rect: Rect, color: Color32, paint: Color32) {
    let (start, tip) = ((20.5_f32, 3.5_f32), (6.0_f32, 18.0_f32));
    // Point at fraction `t` along the brush axis, offset `off` units across it.
    let at = |t: f32, off: f32| -> Pos2 {
        let n = std::f32::consts::FRAC_1_SQRT_2;
        let x = start.0 + (tip.0 - start.0) * t + off * n;
        let y = start.1 + (tip.1 - start.1) * t + off * n;
        grid_p(rect, x, y)
    };

    // Tapered handle with a rounded end.
    let handle = vec![at(0.0, 1.2), at(0.44, 1.6), at(0.44, -1.6), at(0.0, -1.2)];
    painter.add(egui::Shape::convex_polygon(handle, color, Stroke::NONE));
    painter.circle_filled(at(0.0, 0.0), rect.width() * (1.2 / 24.0), color);

    // Metal ferrule, set off from the handle by a hairline gap.
    let ferrule = vec![at(0.47, 2.4), at(0.60, 2.4), at(0.60, -2.4), at(0.47, -2.4)];
    painter.add(egui::Shape::convex_polygon(
        ferrule,
        color.gamma_multiply(0.55),
        Stroke::NONE,
    ));

    // Paint-loaded bristles bulging out of the ferrule and tapering to the tip.
    let bristles = vec![
        at(0.60, 2.4),
        at(0.76, 2.8),
        at(1.0, 0.0),
        at(0.76, -2.8),
        at(0.60, -2.4),
    ];
    painter.add(egui::Shape::convex_polygon(bristles, paint, Stroke::NONE));
}
