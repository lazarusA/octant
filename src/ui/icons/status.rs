//! Status badges, indicators, tools, and notification procedural vector icons.
//! Precision-engineered for Octant following standardized 24-unit geometric keylines.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

/// Helper to map (0..24) normalized grid coordinates into the target bounding `rect`.
#[inline]
fn grid_p(rect: Rect, gx: f32, gy: f32) -> Pos2 {
    pos2(
        rect.min.x + (gx / 24.0) * rect.width(),
        rect.min.y + (gy / 24.0) * rect.height(),
    )
}

/// Scissors: Precision vector slicing shears with optical pivot (20x20dp keyline).
pub fn draw_scissors(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Two geometric handle rings at bottom
    let r_loop = rect.width() * (2.8 / 24.0);
    painter.circle_stroke(p(7.0, 17.5), r_loop, stroke);
    painter.circle_stroke(p(17.0, 17.5), r_loop, stroke);

    // Central optical pivot boss
    let pivot = p(12.0, 12.0);
    painter.circle_filled(pivot, rect.width() * (1.2 / 24.0), stroke.color);

    // Precision tapered cutting blades
    let b1 = [p(8.0, 15.0), p(18.0, 4.0)];
    let b2 = [p(16.0, 15.0), p(6.0, 4.0)];
    painter.line_segment(b1, stroke);
    painter.line_segment(b2, stroke);

    // Secondary blade relief edge
    let b1_relief = [p(12.0, 12.0), p(17.0, 4.0)];
    painter.line_segment(
        b1_relief,
        Stroke::new(stroke.width * 0.75, stroke.color.gamma_multiply(0.40)),
    );
}

/// Check: High-tech verification vector with 45° dynamic takeoff (18x18dp keyline).
pub fn draw_check(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    let check_stroke = Stroke::new(stroke.width * 1.45, stroke.color);
    let pts = [p(4.0, 12.0), p(9.5, 17.5), p(20.5, 6.0)];
    painter.line_segment([pts[0], pts[1]], check_stroke);
    painter.line_segment([pts[1], pts[2]], check_stroke);
}

/// Cross: Precision 45° cancel reticle with balanced terminal endpoints (16x16dp keyline).
pub fn draw_cross(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    let cross_stroke = Stroke::new(stroke.width * 1.35, stroke.color);
    painter.line_segment([p(5.0, 5.0), p(19.0, 19.0)], cross_stroke);
    painter.line_segment([p(19.0, 5.0), p(5.0, 19.0)], cross_stroke);
}

/// Lock: High-security padlock with rounded shackle and laser keyway (14x18dp keyline).
pub fn draw_lock(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Smooth rounded semi-circular shackle
    let mut shackle_pts = Vec::with_capacity(12);
    shackle_pts.push(p(8.0, 11.0));
    let n_arc = 8;
    for i in 0..=n_arc {
        let frac = (i as f32) / (n_arc as f32);
        let angle = std::f32::consts::PI - frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        shackle_pts.push(p(12.0 + c * 4.0, 7.5 - s * 4.0));
    }
    shackle_pts.push(p(16.0, 11.0));

    for win in shackle_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Padlock body with chamfered corners
    let body = Rect::from_min_max(p(5.5, 11.0), p(18.5, 20.5));
    painter.rect(body, 2.0, fill, stroke, StrokeKind::Inside);

    // Refined circular laser keyhole with vertical stem
    painter.circle_filled(p(12.0, 14.5), rect.width() * (1.3 / 24.0), stroke.color);
    let keyway = Stroke::new(stroke.width * 1.2, stroke.color);
    painter.line_segment([p(12.0, 14.5), p(12.0, 17.5)], keyway);
}

/// Unlock: Lifted open shackle with clear disengaged clearance (14x18dp keyline).
pub fn draw_unlock(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Lifted & open smooth shackle
    let mut shackle_pts = Vec::with_capacity(12);
    shackle_pts.push(p(8.0, 11.0));
    let n_arc = 8;
    for i in 0..=n_arc {
        let frac = (i as f32) / (n_arc as f32);
        let angle = std::f32::consts::PI - frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        shackle_pts.push(p(12.0 + c * 4.0, 5.0 - s * 4.0));
    }
    shackle_pts.push(p(16.0, 6.5));

    for win in shackle_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Padlock body (identical placement to Lock for seamless toggle)
    let body = Rect::from_min_max(p(5.5, 11.0), p(18.5, 20.5));
    painter.rect(body, 2.0, fill, stroke, StrokeKind::Inside);

    // Refined keyhole
    painter.circle_filled(p(12.0, 14.5), rect.width() * (1.3 / 24.0), stroke.color);
    let keyway = Stroke::new(stroke.width * 1.2, stroke.color);
    painter.line_segment([p(12.0, 14.5), p(12.0, 17.5)], keyway);
}

/// Bolt: High-voltage discharge vector with dual acute inflection angles (14x20dp keyline).
pub fn draw_bolt(painter: &Painter, rect: Rect, fill: Color32, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    let bolt_pts = vec![
        p(14.0, 3.0),
        p(6.0, 12.0),
        p(11.5, 12.0),
        p(9.5, 21.0),
        p(18.0, 10.0),
        p(13.0, 10.0),
    ];
    painter.add(egui::Shape::convex_polygon(bolt_pts, fill, stroke));
}

/// Hourglass: High-precision chrono-emitter with dual converging chambers (16x18dp keyline).
pub fn draw_hourglass(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Top and bottom plates with end notches
    painter.line_segment([p(4.5, 4.0), p(19.5, 4.0)], stroke);
    painter.line_segment([p(4.5, 20.0), p(19.5, 20.0)], stroke);

    // Triangular glass chambers meeting at central flux point
    let top_chamber = vec![p(6.5, 4.0), p(17.5, 4.0), p(12.0, 12.0)];
    let bot_chamber = vec![p(12.0, 12.0), p(17.5, 20.0), p(6.5, 20.0)];

    painter.add(egui::Shape::convex_polygon(top_chamber, fill, stroke));
    painter.add(egui::Shape::convex_polygon(bot_chamber, fill, stroke));

    // Lower illuminated sand reservoir
    let sand = vec![p(8.5, 16.5), p(15.5, 16.5), p(17.0, 20.0), p(7.0, 20.0)];
    painter.add(egui::Shape::convex_polygon(
        sand,
        stroke.color.gamma_multiply(0.45),
        Stroke::NONE,
    ));
}

/// Warning: Caution hazard trihedron with rounded vertex chamfers (18x18dp keyline).
pub fn draw_warning(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Chamfered hazard triangle
    let tri_pts = vec![
        p(12.0, 3.5),
        p(21.0, 19.5),
        p(19.5, 20.5),
        p(4.5, 20.5),
        p(3.0, 19.5),
    ];
    painter.add(egui::Shape::convex_polygon(tri_pts, fill, stroke));

    // Exclamation point stem & dot
    let ex_stroke = Stroke::new(stroke.width * 1.3, stroke.color);
    painter.line_segment([p(12.0, 9.0), p(12.0, 14.0)], ex_stroke);
    painter.circle_filled(p(12.0, 17.5), rect.width() * (1.2 / 24.0), stroke.color);
}

/// Info: Circular telemetry beacon with notification dot and vertical pillar (20dp circle keyline).
pub fn draw_info(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * (10.0 / 24.0);
    if r <= 1.0 {
        return;
    }

    // Outer circle
    painter.circle_stroke(center, r, stroke);

    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    painter.circle_filled(p(12.0, 8.0), rect.width() * (1.3 / 24.0), stroke.color);
    let stem_stroke = Stroke::new(stroke.width * 1.2, stroke.color);
    painter.line_segment([p(12.0, 11.0), p(12.0, 16.5)], stem_stroke);
}

/// Bullet: Precision geometric diamond telemetry node (Centered 10dp diamond).
pub fn draw_bullet(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let d = rect.width().min(rect.height()) * (5.0 / 24.0);
    let pts = vec![
        pos2(center.x, center.y - d),
        pos2(center.x + d, center.y),
        pos2(center.x, center.y + d),
        pos2(center.x - d, center.y),
    ];
    painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
}

/// ChevronRight: Crisp 90° forward vector pointer (14x16dp keyline).
pub fn draw_chevron_right(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    let chevron_stroke = Stroke::new(stroke.width * 1.2, stroke.color);
    let pts = [p(8.5, 5.0), p(15.5, 12.0), p(8.5, 19.0)];
    painter.line_segment([pts[0], pts[1]], chevron_stroke);
    painter.line_segment([pts[1], pts[2]], chevron_stroke);
}
