//! Status badges, indicators, tools, and notification procedural vector icons.
//! Minimalistically futuristic and precision-engineered for Octant.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

/// Scissors: Precision vector slicing shears with beveled bypass blades and optical pivot.
pub fn draw_scissors(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Two geometric handle rings at bottom
    let r_loop = rect.width() * 0.11;
    painter.circle_stroke(p(0.30, 0.72), r_loop, stroke);
    painter.circle_stroke(p(0.70, 0.72), r_loop, stroke);

    // Central optical pivot boss
    let pivot = p(0.50, 0.50);
    painter.circle_filled(pivot, rect.width() * 0.05, stroke.color);

    // Precision tapered cutting blades
    let b1 = [p(0.34, 0.63), p(0.74, 0.16)];
    let b2 = [p(0.66, 0.63), p(0.26, 0.16)];
    painter.line_segment(b1, stroke);
    painter.line_segment(b2, stroke);

    // Secondary blade relief edge
    let b1_relief = [p(0.50, 0.50), p(0.70, 0.16)];
    painter.line_segment(
        b1_relief,
        Stroke::new(stroke.width * 0.75, stroke.color.gamma_multiply(0.40)),
    );
}

/// Check: High-tech verification vector with 45° dynamic takeoff.
pub fn draw_check(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let check_stroke = Stroke::new(stroke.width * 1.4, stroke.color);
    let pts = [p(0.18, 0.52), p(0.40, 0.76), p(0.84, 0.22)];
    painter.line_segment([pts[0], pts[1]], check_stroke);
    painter.line_segment([pts[1], pts[2]], check_stroke);
}

/// Cross: Precision 45° cancel reticle with balanced terminal endpoints.
pub fn draw_cross(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let cross_stroke = Stroke::new(stroke.width * 1.35, stroke.color);
    painter.line_segment([p(0.24, 0.24), p(0.76, 0.76)], cross_stroke);
    painter.line_segment([p(0.76, 0.24), p(0.24, 0.76)], cross_stroke);
}

/// Lock: High-security padlock with rounded shackle and laser keyway.
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

    // Padlock body with chamfered corners
    let body = Rect::from_min_max(p(0.24, 0.46), p(0.76, 0.86));
    painter.rect(body, 2.5, fill, stroke, StrokeKind::Inside);

    // Refined circular laser keyhole with vertical stem
    painter.circle_filled(p(0.50, 0.61), rect.width() * 0.055, stroke.color);
    let keyway = Stroke::new(stroke.width * 1.2, stroke.color);
    painter.line_segment([p(0.50, 0.61), p(0.50, 0.73)], keyway);
}

/// Unlock: Lifted open shackle with clear disengaged clearance.
pub fn draw_unlock(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Lifted & open smooth shackle
    let mut shackle_pts = Vec::with_capacity(12);
    shackle_pts.push(p(0.34, 0.46));
    let n_arc = 8;
    for i in 0..=n_arc {
        let frac = (i as f32) / (n_arc as f32);
        let angle = std::f32::consts::PI - frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        shackle_pts.push(p(0.50 + c * 0.16, 0.22 - s * 0.16));
    }
    shackle_pts.push(p(0.66, 0.28));

    for win in shackle_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Padlock body (identical placement to Lock for seamless toggle)
    let body = Rect::from_min_max(p(0.24, 0.46), p(0.76, 0.86));
    painter.rect(body, 2.5, fill, stroke, StrokeKind::Inside);

    // Refined keyhole
    painter.circle_filled(p(0.50, 0.61), rect.width() * 0.055, stroke.color);
    let keyway = Stroke::new(stroke.width * 1.2, stroke.color);
    painter.line_segment([p(0.50, 0.61), p(0.50, 0.73)], keyway);
}

/// Bolt: Angular high-voltage discharge vector with dual acute inflection angles.
pub fn draw_bolt(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let bolt_pts = vec![
        p(0.58, 0.12),
        p(0.26, 0.48),
        p(0.48, 0.48),
        p(0.38, 0.88),
        p(0.74, 0.42),
        p(0.52, 0.42),
    ];
    painter.add(egui::Shape::convex_polygon(bolt_pts, fill, stroke));
}

/// Hourglass: High-precision chrono-emitter with dual converging pyramidal chambers.
pub fn draw_hourglass(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Top and bottom plates with end notches
    painter.line_segment([p(0.20, 0.16), p(0.80, 0.16)], stroke);
    painter.line_segment([p(0.20, 0.84), p(0.80, 0.84)], stroke);

    // Triangular glass chambers meeting at central flux point
    let top_chamber = vec![p(0.28, 0.16), p(0.72, 0.16), p(0.50, 0.50)];
    let bot_chamber = vec![p(0.50, 0.50), p(0.72, 0.84), p(0.28, 0.84)];

    painter.add(egui::Shape::convex_polygon(top_chamber, fill, stroke));
    painter.add(egui::Shape::convex_polygon(bot_chamber, fill, stroke));

    // Lower illuminated sand reservoir
    let sand = vec![p(0.36, 0.70), p(0.64, 0.70), p(0.70, 0.84), p(0.30, 0.84)];
    painter.add(egui::Shape::convex_polygon(
        sand,
        stroke.color.gamma_multiply(0.45),
        Stroke::NONE,
    ));
}

/// Warning: Caution hazard trihedron with rounded vertex chamfers.
pub fn draw_warning(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Chamfered hazard triangle
    let tri_pts = vec![
        p(0.50, 0.14),
        p(0.88, 0.82),
        p(0.82, 0.86),
        p(0.18, 0.86),
        p(0.12, 0.82),
    ];
    painter.add(egui::Shape::convex_polygon(tri_pts, fill, stroke));

    // Exclamation point stem & dot
    let ex_stroke = Stroke::new(stroke.width * 1.3, stroke.color);
    painter.line_segment([p(0.50, 0.38), p(0.50, 0.60)], ex_stroke);
    painter.circle_filled(p(0.50, 0.73), rect.width() * 0.05, stroke.color);
}

/// Info: Circular telemetry beacon with notification dot and vertical pillar.
pub fn draw_info(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.40;
    if r <= 1.0 {
        return;
    }

    // Outer circle
    painter.circle_stroke(center, r, stroke);

    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    painter.circle_filled(p(0.50, 0.34), rect.width() * 0.055, stroke.color);
    let stem_stroke = Stroke::new(stroke.width * 1.2, stroke.color);
    painter.line_segment([p(0.50, 0.46), p(0.50, 0.70)], stem_stroke);
}

/// Bullet: Precision geometric diamond telemetry node.
pub fn draw_bullet(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let d = rect.width().min(rect.height()) * 0.22;
    let pts = vec![
        pos2(center.x, center.y - d),
        pos2(center.x + d, center.y),
        pos2(center.x, center.y + d),
        pos2(center.x - d, center.y),
    ];
    painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
}

/// ChevronRight: Crisp 60° forward vector pointer.
pub fn draw_chevron_right(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let chevron_stroke = Stroke::new(stroke.width * 1.2, stroke.color);
    let pts = [p(0.36, 0.22), p(0.64, 0.50), p(0.36, 0.78)];
    painter.line_segment([pts[0], pts[1]], chevron_stroke);
    painter.line_segment([pts[1], pts[2]], chevron_stroke);
}
