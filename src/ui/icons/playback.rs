//! Playback and timeline procedural vector icons.
//! Precision-engineered, minimalistically futuristic geometry for Octant.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

/// Play: Precision right-pointing directional triangle with chamfered back corners.
pub fn draw_play(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let pts = vec![
        p(0.26, 0.16),
        p(0.82, 0.50),
        p(0.26, 0.84),
        p(0.22, 0.80),
        p(0.22, 0.20),
    ];
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
}

/// Pause: Dual chamfered technical vertical pillars.
pub fn draw_pause(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let bar1 = Rect::from_min_max(p(0.22, 0.18), p(0.40, 0.82));
    let bar2 = Rect::from_min_max(p(0.60, 0.18), p(0.78, 0.82));

    painter.rect(bar1, 1.5, fill, stroke, StrokeKind::Inside);
    painter.rect(bar2, 1.5, fill, stroke, StrokeKind::Inside);
}

/// Stop: Chamfered technical telemetry square.
pub fn draw_stop(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let pts = vec![
        p(0.28, 0.22),
        p(0.72, 0.22),
        p(0.78, 0.28),
        p(0.78, 0.72),
        p(0.72, 0.78),
        p(0.28, 0.78),
        p(0.22, 0.72),
        p(0.22, 0.28),
    ];
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
}

/// StepBackward: Left directional triangle with end stop bar.
pub fn draw_step_backward(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // End stop bar
    let bar = Rect::from_min_max(p(0.18, 0.20), p(0.28, 0.80));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);

    // Left-pointing triangle
    let pts = vec![
        p(0.78, 0.20),
        p(0.34, 0.50),
        p(0.78, 0.80),
        p(0.82, 0.76),
        p(0.82, 0.24),
    ];
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
}

/// StepForward: Right directional triangle with end stop bar.
pub fn draw_step_forward(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Right-pointing triangle
    let pts = vec![
        p(0.22, 0.20),
        p(0.66, 0.50),
        p(0.22, 0.80),
        p(0.18, 0.76),
        p(0.18, 0.24),
    ];
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));

    // End stop bar
    let bar = Rect::from_min_max(p(0.72, 0.20), p(0.82, 0.80));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);
}

/// SeekStart: Dual rapid seek triangles with left boundary limit.
pub fn draw_seek_start(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Left limit bar
    let bar = Rect::from_min_max(p(0.14, 0.20), p(0.24, 0.80));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);

    // Triangle 1 (inner)
    let t1 = vec![p(0.54, 0.22), p(0.28, 0.50), p(0.54, 0.78)];
    painter.add(egui::Shape::convex_polygon(t1, fill, stroke));

    // Triangle 2 (outer)
    let t2 = vec![p(0.84, 0.22), p(0.58, 0.50), p(0.84, 0.78)];
    painter.add(egui::Shape::convex_polygon(t2, fill, stroke));
}

/// SeekEnd: Dual rapid seek triangles with right boundary limit.
pub fn draw_seek_end(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Triangle 1 (outer)
    let t1 = vec![p(0.16, 0.22), p(0.42, 0.50), p(0.16, 0.78)];
    painter.add(egui::Shape::convex_polygon(t1, fill, stroke));

    // Triangle 2 (inner)
    let t2 = vec![p(0.46, 0.22), p(0.72, 0.50), p(0.46, 0.78)];
    painter.add(egui::Shape::convex_polygon(t2, fill, stroke));

    // Right limit bar
    let bar = Rect::from_min_max(p(0.76, 0.20), p(0.86, 0.80));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);
}

/// Loop: Dual continuous swept orbital trajectory with stealth arrowheads.
pub fn draw_loop(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.36;
    if r <= 1.0 {
        return;
    }

    let n_pts = 10;
    let mut top_pts = Vec::with_capacity(n_pts + 1);
    let mut bot_pts = Vec::with_capacity(n_pts + 1);

    for i in 0..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        let angle_top = std::f32::consts::PI * 0.08 + frac * (std::f32::consts::PI * 0.84);
        let (s, c) = angle_top.sin_cos();
        top_pts.push(pos2(center.x + c * r, center.y - s * r));

        let angle_bot = std::f32::consts::PI * 1.08 + frac * (std::f32::consts::PI * 0.84);
        let (s, c) = angle_bot.sin_cos();
        bot_pts.push(pos2(center.x + c * r, center.y - s * r));
    }

    for win in top_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }
    for win in bot_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Top swept arrowhead (pointing right)
    if let Some(&tip) = top_pts.first() {
        let ah = vec![
            pos2(tip.x + r * 0.40, tip.y),
            pos2(tip.x + r * 0.05, tip.y - r * 0.32),
            pos2(tip.x + r * 0.12, tip.y),
            pos2(tip.x + r * 0.05, tip.y + r * 0.32),
        ];
        painter.add(egui::Shape::convex_polygon(ah, fill, stroke));
    }

    // Bottom swept arrowhead (pointing left)
    if let Some(&tip) = bot_pts.first() {
        let ah = vec![
            pos2(tip.x - r * 0.40, tip.y),
            pos2(tip.x - r * 0.05, tip.y + r * 0.32),
            pos2(tip.x - r * 0.12, tip.y),
            pos2(tip.x - r * 0.05, tip.y - r * 0.32),
        ];
        painter.add(egui::Shape::convex_polygon(ah, fill, stroke));
    }
}

/// Reset: Counter-clockwise telemetry rewind loop with swept technical arrow.
pub fn draw_reset(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.35;
    if r <= 1.0 {
        return;
    }

    let n_pts = 16;
    let mut arc_pts = Vec::with_capacity(n_pts + 1);
    let start_angle = std::f32::consts::TAU * 0.38; // ~137 deg
    let end_angle = std::f32::consts::TAU * 1.20; // ~432 deg (past top)

    for i in 0..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        let angle = start_angle + frac * (end_angle - start_angle);
        let (s, c) = angle.sin_cos();
        arc_pts.push(pos2(center.x + c * r, center.y - s * r));
    }

    for win in arc_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // High-tech swept arrowhead at top (12 o'clock) pointing left
    let tip = pos2(center.x - r * 0.28, center.y - r);
    let ah = vec![
        tip,
        pos2(center.x + r * 0.16, center.y - r - r * 0.30),
        pos2(center.x + r * 0.08, center.y - r),
        pos2(center.x + r * 0.16, center.y - r + r * 0.30),
    ];
    painter.add(egui::Shape::convex_polygon(ah, fill, stroke));
}
