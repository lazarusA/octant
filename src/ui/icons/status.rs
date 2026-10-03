//! Status badges, indicators, tools, and notification procedural vector icons.
//! Precision-engineered for Octant following standardized 24-unit geometric keylines.

use super::grid_p;
use egui::{Color32, Painter, Rect, Stroke, StrokeKind, pos2};

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

    // Smooth rounded semi-circular shackle (zero-allocation)
    painter.line_segment([p(8.0, 11.0), p(8.0, 7.5)], stroke);
    let n_arc = 8;
    let mut prev = p(8.0, 7.5);
    for i in 1..=n_arc {
        let frac = (i as f32) / (n_arc as f32);
        let angle = std::f32::consts::PI - frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        let curr = p(12.0 + c * 4.0, 7.5 - s * 4.0);
        painter.line_segment([prev, curr], stroke);
        prev = curr;
    }
    painter.line_segment([prev, p(16.0, 11.0)], stroke);

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

    // Lifted & open smooth shackle (zero-allocation)
    painter.line_segment([p(8.0, 11.0), p(8.0, 5.0)], stroke);
    let n_arc = 8;
    let mut prev = p(8.0, 5.0);
    for i in 1..=n_arc {
        let frac = (i as f32) / (n_arc as f32);
        let angle = std::f32::consts::PI - frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        let curr = p(12.0 + c * 4.0, 5.0 - s * 4.0);
        painter.line_segment([prev, curr], stroke);
        prev = curr;
    }
    painter.line_segment([prev, p(16.0, 6.5)], stroke);

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

/// Hourglass: Crisp sand timer with a flat waist and sand settling in both chambers (14x18dp keyline).
pub fn draw_hourglass(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);
    // Glass spans x 6.5..17.5 between the plates, narrowing to a flat waist.
    let (top, bottom, waist_top, waist_bot) = (4.0, 20.0, 11.25, 12.75);
    let (outer, inner) = (5.5, 0.75);
    let slope = (outer - inner) / (waist_top - top);
    // Half width of the glass at height `y`, and the sand inset inside it.
    let half = |y: f32| {
        if y <= waist_top {
            outer - (y - top) * slope
        } else if y >= waist_bot {
            inner + (y - waist_bot) * slope
        } else {
            inner
        }
    };
    let sand_at = |y: f32, side: f32| p(12.0 + side * (half(y) - 1.0), y);

    // Glass fill in three bands: upper chamber, waist, lower chamber.
    for (y0, y1) in [
        (top, waist_top),
        (waist_top, waist_bot),
        (waist_bot, bottom),
    ] {
        let (h0, h1) = (half(y0), half(y1));
        let pts = vec![
            p(12.0 - h0, y0),
            p(12.0 + h0, y0),
            p(12.0 + h1, y1),
            p(12.0 - h1, y1),
        ];
        painter.add(egui::Shape::convex_polygon(pts, fill, Stroke::NONE));
    }

    // Sand: a remnant above the waist and a mounded pile below it.
    let sand = stroke.color.gamma_multiply(0.55);
    let remnant = vec![sand_at(8.0, -1.0), sand_at(8.0, 1.0), p(12.0, 10.5)];
    painter.add(egui::Shape::convex_polygon(remnant, sand, Stroke::NONE));
    let pile = vec![
        sand_at(17.0, -1.0),
        p(12.0, 15.5),
        sand_at(17.0, 1.0),
        sand_at(19.0, 1.0),
        sand_at(19.0, -1.0),
    ];
    painter.add(egui::Shape::convex_polygon(pile, sand, Stroke::NONE));

    // Glass sides as two open strokes so nothing doubles up at the waist.
    for side in [-1.0_f32, 1.0] {
        let x = |dx: f32| 12.0 + side * dx;
        let pts = vec![
            p(x(outer), top),
            p(x(inner), waist_top),
            p(x(inner), waist_bot),
            p(x(outer), bottom),
        ];
        painter.add(egui::Shape::line(pts, stroke));
    }

    // End plates, only slightly wider than the glass.
    painter.line_segment([p(5.0, top), p(19.0, top)], stroke);
    painter.line_segment([p(5.0, bottom), p(19.0, bottom)], stroke);
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
