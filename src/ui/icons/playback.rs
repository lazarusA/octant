//! Playback and timeline procedural vector icons.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

pub fn draw_play(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Right-pointing triangle
    let points = vec![p(0.24, 0.16), p(0.84, 0.50), p(0.24, 0.84)];
    painter.add(egui::Shape::convex_polygon(points, fill, stroke));
}

pub fn draw_pause(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let bar1 = Rect::from_min_max(p(0.22, 0.18), p(0.42, 0.82));
    let bar2 = Rect::from_min_max(p(0.58, 0.18), p(0.78, 0.82));

    painter.rect(bar1, 1.0, fill, stroke, StrokeKind::Inside);
    painter.rect(bar2, 1.0, fill, stroke, StrokeKind::Inside);
}

pub fn draw_stop(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let stop_rect = Rect::from_min_max(p(0.24, 0.24), p(0.76, 0.76));
    painter.rect(stop_rect, 1.5, fill, stroke, StrokeKind::Inside);
}

pub fn draw_step_backward(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Left-pointing triangle
    let points = vec![p(0.76, 0.18), p(0.22, 0.50), p(0.76, 0.82)];
    painter.add(egui::Shape::convex_polygon(points, fill, stroke));
}

pub fn draw_step_forward(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Right-pointing triangle
    let points = vec![p(0.24, 0.18), p(0.78, 0.50), p(0.24, 0.82)];
    painter.add(egui::Shape::convex_polygon(points, fill, stroke));
}

pub fn draw_seek_start(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Left vertical bar
    let bar = Rect::from_min_max(p(0.18, 0.18), p(0.32, 0.82));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);

    // Left-pointing triangle
    let points = vec![p(0.82, 0.18), p(0.36, 0.50), p(0.82, 0.82)];
    painter.add(egui::Shape::convex_polygon(points, fill, stroke));
}

pub fn draw_seek_end(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Right-pointing triangle
    let points = vec![p(0.18, 0.18), p(0.64, 0.50), p(0.18, 0.82)];
    painter.add(egui::Shape::convex_polygon(points, fill, stroke));

    // Right vertical bar
    let bar = Rect::from_min_max(p(0.68, 0.18), p(0.82, 0.82));
    painter.rect(bar, 1.0, fill, stroke, StrokeKind::Inside);
}

pub fn draw_loop(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.36;
    if r <= 1.0 {
        return;
    }

    // Top arc and bottom arc
    let n_pts = 10;
    let mut top_pts = Vec::with_capacity(n_pts);
    let mut bot_pts = Vec::with_capacity(n_pts);

    for i in 0..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        // Top right to top left
        let angle_top = std::f32::consts::PI * 0.1 + frac * (std::f32::consts::PI * 0.8);
        let (s, c) = angle_top.sin_cos();
        top_pts.push(pos2(center.x + c * r, center.y - s * r));

        // Bottom left to bottom right
        let angle_bot = std::f32::consts::PI * 1.1 + frac * (std::f32::consts::PI * 0.8);
        let (s, c) = angle_bot.sin_cos();
        bot_pts.push(pos2(center.x + c * r, center.y - s * r));
    }

    for win in top_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }
    for win in bot_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Top arrow head (pointing right)
    if let Some(&tip) = top_pts.first() {
        let ah = vec![
            pos2(tip.x + r * 0.15, tip.y - r * 0.30),
            pos2(tip.x + r * 0.40, tip.y + r * 0.05),
            pos2(tip.x - r * 0.05, tip.y + r * 0.15),
        ];
        painter.add(egui::Shape::convex_polygon(ah, fill, stroke));
    }

    // Bottom arrow head (pointing left)
    if let Some(&tip) = bot_pts.first() {
        let ah = vec![
            pos2(tip.x - r * 0.15, tip.y + r * 0.30),
            pos2(tip.x - r * 0.40, tip.y - r * 0.05),
            pos2(tip.x + r * 0.05, tip.y - r * 0.15),
        ];
        painter.add(egui::Shape::convex_polygon(ah, fill, stroke));
    }
}

pub fn draw_reset(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.36;
    if r <= 1.0 {
        return;
    }

    // Counter-clockwise gapped arc
    let n_pts = 14;
    let mut arc_pts = Vec::with_capacity(n_pts);
    for i in 0..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        let angle = std::f32::consts::PI * 0.2 + frac * (std::f32::consts::PI * 1.55);
        let (s, c) = angle.sin_cos();
        arc_pts.push(pos2(center.x + c * r, center.y - s * r));
    }

    for win in arc_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Arrow tip pointing down/left at arc start
    if let Some(&tip) = arc_pts.first() {
        let ah = vec![
            pos2(tip.x - r * 0.25, tip.y - r * 0.25),
            pos2(tip.x + r * 0.10, tip.y - r * 0.35),
            pos2(tip.x, tip.y + r * 0.10),
        ];
        painter.add(egui::Shape::convex_polygon(ah, fill, stroke));
    }
}
