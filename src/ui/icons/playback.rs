//! Playback and timeline procedural vector icons.
//! Precision-engineered for Octant following standardized 24-unit geometric keylines.

use super::grid_p;
use egui::{Color32, Painter, Rect, Stroke, StrokeKind, pos2};

/// Play: Precision right-pointing directional triangle with optical center compensation (14x16dp keyline).
pub fn draw_play(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Optically centered with +1.0dp X-shift
    let pts = vec![
        p(7.0, 4.0),
        p(20.0, 12.0),
        p(7.0, 20.0),
        p(5.5, 18.5),
        p(5.5, 5.5),
    ];
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
}

/// Pause: Dual chamfered technical vertical pillars (16x16dp keyline).
pub fn draw_pause(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    let bar1 = Rect::from_min_max(p(5.0, 4.0), p(9.0, 20.0));
    let bar2 = Rect::from_min_max(p(15.0, 4.0), p(19.0, 20.0));

    painter.rect(bar1, 1.5, fill, stroke, StrokeKind::Inside);
    painter.rect(bar2, 1.5, fill, stroke, StrokeKind::Inside);
}

/// Stop: Chamfered technical telemetry square (14x14dp keyline).
pub fn draw_stop(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    let pts = vec![
        p(6.5, 5.0),
        p(17.5, 5.0),
        p(19.0, 6.5),
        p(19.0, 17.5),
        p(17.5, 19.0),
        p(6.5, 19.0),
        p(5.0, 17.5),
        p(5.0, 6.5),
    ];
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
}

/// StepBackward: Left directional triangle with end stop bar (17x16dp keyline).
pub fn draw_step_backward(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // End stop bar
    let bar = Rect::from_min_max(p(4.0, 4.0), p(7.0, 20.0));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);

    // Left-pointing triangle with 2dp clearance to bar
    let pts = vec![
        p(19.0, 4.5),
        p(8.5, 12.0),
        p(19.0, 19.5),
        p(20.0, 18.5),
        p(20.0, 5.5),
    ];
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
}

/// StepForward: Right directional triangle with end stop bar (17x16dp keyline).
pub fn draw_step_forward(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Right-pointing triangle
    let pts = vec![
        p(5.0, 4.5),
        p(15.5, 12.0),
        p(5.0, 19.5),
        p(4.0, 18.5),
        p(4.0, 5.5),
    ];
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));

    // End stop bar
    let bar = Rect::from_min_max(p(17.0, 4.0), p(20.0, 20.0));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);
}

/// SeekStart: Dual rapid seek triangles with left boundary limit (18x16dp keyline).
pub fn draw_seek_start(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Left limit bar
    let bar = Rect::from_min_max(p(3.5, 4.0), p(6.5, 20.0));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);

    // Triangle 1 (inner)
    let t1 = vec![p(13.5, 5.0), p(7.5, 12.0), p(13.5, 19.0)];
    painter.add(egui::Shape::convex_polygon(t1, fill, stroke));

    // Triangle 2 (outer)
    let t2 = vec![p(20.5, 5.0), p(14.5, 12.0), p(20.5, 19.0)];
    painter.add(egui::Shape::convex_polygon(t2, fill, stroke));
}

/// SeekEnd: Dual rapid seek triangles with right boundary limit (18x16dp keyline).
pub fn draw_seek_end(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Triangle 1 (outer)
    let t1 = vec![p(3.5, 5.0), p(9.5, 12.0), p(3.5, 19.0)];
    painter.add(egui::Shape::convex_polygon(t1, fill, stroke));

    // Triangle 2 (inner)
    let t2 = vec![p(10.5, 5.0), p(16.5, 12.0), p(10.5, 19.0)];
    painter.add(egui::Shape::convex_polygon(t2, fill, stroke));

    // Right limit bar
    let bar = Rect::from_min_max(p(17.5, 4.0), p(20.5, 20.0));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);
}

/// Loop: Dual continuous swept orbital trajectory with stealth arrowheads (18dp circle keyline).
pub fn draw_loop(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * (8.5 / 24.0);
    if r <= 1.0 {
        return;
    }

    let n_pts = 12;
    let angle_top_0 = std::f32::consts::PI * 0.08;
    let (s0, c0) = angle_top_0.sin_cos();
    let tip_top = pos2(center.x + c0 * r, center.y - s0 * r);
    let mut prev_top = tip_top;

    let angle_bot_0 = std::f32::consts::PI * 1.08;
    let (s0_b, c0_b) = angle_bot_0.sin_cos();
    let tip_bot = pos2(center.x + c0_b * r, center.y - s0_b * r);
    let mut prev_bot = tip_bot;

    for i in 1..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        let angle_top = std::f32::consts::PI * 0.08 + frac * (std::f32::consts::PI * 0.84);
        let (s, c) = angle_top.sin_cos();
        let curr_top = pos2(center.x + c * r, center.y - s * r);
        painter.line_segment([prev_top, curr_top], stroke);
        prev_top = curr_top;

        let angle_bot = std::f32::consts::PI * 1.08 + frac * (std::f32::consts::PI * 0.84);
        let (s, c) = angle_bot.sin_cos();
        let curr_bot = pos2(center.x + c * r, center.y - s * r);
        painter.line_segment([prev_bot, curr_bot], stroke);
        prev_bot = curr_bot;
    }

    // Top swept arrowhead (pointing right)
    let ah = vec![
        pos2(tip_top.x + r * 0.38, tip_top.y),
        pos2(tip_top.x + r * 0.04, tip_top.y - r * 0.30),
        pos2(tip_top.x + r * 0.10, tip_top.y),
        pos2(tip_top.x + r * 0.04, tip_top.y + r * 0.30),
    ];
    painter.add(egui::Shape::convex_polygon(ah, fill, stroke));

    // Bottom swept arrowhead (pointing left)
    let ah_b = vec![
        pos2(tip_bot.x - r * 0.38, tip_bot.y),
        pos2(tip_bot.x - r * 0.04, tip_bot.y + r * 0.30),
        pos2(tip_bot.x - r * 0.10, tip_bot.y),
        pos2(tip_bot.x - r * 0.04, tip_bot.y - r * 0.30),
    ];
    painter.add(egui::Shape::convex_polygon(ah_b, fill, stroke));
}

/// Reset: Counter-clockwise telemetry rewind loop with swept technical arrow (18dp circle keyline).
pub fn draw_reset(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * (8.5 / 24.0);
    if r <= 1.0 {
        return;
    }

    let n_pts = 16;
    let start_angle = std::f32::consts::TAU * 0.38; // ~137 deg
    let end_angle = std::f32::consts::TAU * 1.20; // ~432 deg (past top)

    let (s0, c0) = start_angle.sin_cos();
    let mut prev_pt = pos2(center.x + c0 * r, center.y - s0 * r);

    for i in 1..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        let angle = start_angle + frac * (end_angle - start_angle);
        let (s, c) = angle.sin_cos();
        let curr_pt = pos2(center.x + c * r, center.y - s * r);
        painter.line_segment([prev_pt, curr_pt], stroke);
        prev_pt = curr_pt;
    }

    // High-tech swept arrowhead at top pointing left
    let tip = pos2(center.x - r * 0.28, center.y - r);
    let ah = vec![
        tip,
        pos2(center.x + r * 0.14, center.y - r - r * 0.28),
        pos2(center.x + r * 0.06, center.y - r),
        pos2(center.x + r * 0.14, center.y - r + r * 0.28),
    ];
    painter.add(egui::Shape::convex_polygon(ah, fill, stroke));
}
