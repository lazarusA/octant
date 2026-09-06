//! Status badges, indicators, tools, and notification procedural vector icons.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

pub fn draw_scissors(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Two finger loops at bottom
    let r_loop = rect.width() * 0.12;
    painter.circle_stroke(p(0.30, 0.72), r_loop, stroke);
    painter.circle_stroke(p(0.70, 0.72), r_loop, stroke);

    // Pivot point
    let pivot = p(0.50, 0.50);
    painter.circle_filled(pivot, rect.width() * 0.05, stroke.color);

    // Crossed blades
    painter.line_segment([p(0.36, 0.62), p(0.74, 0.18)], stroke);
    painter.line_segment([p(0.64, 0.62), p(0.26, 0.18)], stroke);
}

pub fn draw_check(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let check_stroke = Stroke::new(stroke.width * 1.3, stroke.color);
    let pts = [p(0.20, 0.52), p(0.42, 0.76), p(0.82, 0.24)];
    painter.line_segment([pts[0], pts[1]], check_stroke);
    painter.line_segment([pts[1], pts[2]], check_stroke);
}

pub fn draw_cross(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let cross_stroke = Stroke::new(stroke.width * 1.3, stroke.color);
    painter.line_segment([p(0.24, 0.24), p(0.76, 0.76)], cross_stroke);
    painter.line_segment([p(0.76, 0.24), p(0.24, 0.76)], cross_stroke);
}

pub fn draw_lock(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Smooth rounded semi-circular shackle
    let mut shackle_pts = Vec::with_capacity(12);
    shackle_pts.push(p(0.34, 0.46));
    let n_arc = 8;
    for i in 0..=n_arc {
        let frac = (i as f32) / (n_arc as f32);
        let angle = std::f32::consts::PI - frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        shackle_pts.push(p(0.50 + c * 0.16, 0.32 - s * 0.16));
    }
    shackle_pts.push(p(0.66, 0.46));

    for win in shackle_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Padlock body with subtle rounded corners
    let body = Rect::from_min_max(p(0.24, 0.46), p(0.76, 0.86));
    painter.rect(body, 2.5, fill, stroke, StrokeKind::Inside);

    // Refined keyhole: circular top with tapered keyway slot
    painter.circle_filled(p(0.50, 0.61), rect.width() * 0.055, stroke.color);
    let keyway = Stroke::new(stroke.width * 1.1, stroke.color);
    painter.line_segment([p(0.50, 0.61), p(0.50, 0.73)], keyway);
}

pub fn draw_unlock(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Lifted & open smooth shackle with distinct clear opening
    let mut shackle_pts = Vec::with_capacity(12);
    shackle_pts.push(p(0.34, 0.46));
    let n_arc = 8;
    for i in 0..=n_arc {
        let frac = (i as f32) / (n_arc as f32);
        let angle = std::f32::consts::PI - frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        shackle_pts.push(p(0.50 + c * 0.16, 0.24 - s * 0.16));
    }
    shackle_pts.push(p(0.66, 0.30));

    for win in shackle_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Padlock body (identical placement to Lock for seamless toggle)
    let body = Rect::from_min_max(p(0.24, 0.46), p(0.76, 0.86));
    painter.rect(body, 2.5, fill, stroke, StrokeKind::Inside);

    // Refined keyhole
    painter.circle_filled(p(0.50, 0.61), rect.width() * 0.055, stroke.color);
    let keyway = Stroke::new(stroke.width * 1.1, stroke.color);
    painter.line_segment([p(0.50, 0.61), p(0.50, 0.73)], keyway);
}

pub fn draw_bolt(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Jagged lightning bolt
    let bolt_pts = vec![
        p(0.58, 0.12),
        p(0.28, 0.48),
        p(0.48, 0.48),
        p(0.38, 0.88),
        p(0.74, 0.42),
        p(0.54, 0.42),
    ];
    painter.add(egui::Shape::convex_polygon(bolt_pts, fill, stroke));
}

pub fn draw_hourglass(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Top and bottom plates
    painter.line_segment([p(0.22, 0.16), p(0.78, 0.16)], stroke);
    painter.line_segment([p(0.22, 0.84), p(0.78, 0.84)], stroke);

    // Triangular glass chambers meeting at center
    let top_chamber = vec![p(0.28, 0.16), p(0.72, 0.16), p(0.50, 0.50)];
    let bot_chamber = vec![p(0.50, 0.50), p(0.72, 0.84), p(0.28, 0.84)];

    painter.add(egui::Shape::convex_polygon(top_chamber, fill, stroke));
    painter.add(egui::Shape::convex_polygon(bot_chamber, fill, stroke));

    // Bottom sand fill
    let sand = vec![p(0.38, 0.70), p(0.62, 0.70), p(0.70, 0.84), p(0.30, 0.84)];
    painter.add(egui::Shape::convex_polygon(
        sand,
        stroke.color.gamma_multiply(0.45),
        Stroke::NONE,
    ));
}

pub fn draw_warning(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Hazard triangle
    let tri_pts = vec![p(0.50, 0.14), p(0.88, 0.84), p(0.12, 0.84)];
    painter.add(egui::Shape::convex_polygon(tri_pts, fill, stroke));

    // Exclamation point stem & dot
    let ex_stroke = Stroke::new(stroke.width * 1.2, stroke.color);
    painter.line_segment([p(0.50, 0.38), p(0.50, 0.60)], ex_stroke);
    painter.circle_filled(p(0.50, 0.72), rect.width() * 0.05, stroke.color);
}

pub fn draw_info(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.40;
    if r <= 1.0 {
        return;
    }

    // Outer circle
    painter.circle_stroke(center, r, stroke);

    // Letter 'i' (dot + vertical stem)
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    painter.circle_filled(p(0.50, 0.34), rect.width() * 0.055, stroke.color);
    let stem_stroke = Stroke::new(stroke.width * 1.1, stroke.color);
    painter.line_segment([p(0.50, 0.46), p(0.50, 0.70)], stem_stroke);
}

pub fn draw_bullet(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.22;
    painter.circle_filled(center, r, color);
}

pub fn draw_chevron_right(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let pts = [p(0.36, 0.22), p(0.64, 0.50), p(0.36, 0.78)];
    painter.line_segment([pts[0], pts[1]], stroke);
    painter.line_segment([pts[1], pts[2]], stroke);
}
