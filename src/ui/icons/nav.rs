//! Navigation, panel, and theme procedural vector icons.
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

/// Globe: High-tech planetary geoid with equatorial plane and curved meridian arcs (20dp circle keyline).
pub fn draw_globe(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    // 20dp circle keyline: radius = 10/24 * dimension
    let r = rect.width().min(rect.height()) * (10.0 / 24.0);
    if r <= 0.5 {
        return;
    }

    // Outer perimeter ring (Circle keyline)
    painter.circle_stroke(center, r, stroke);

    // Horizontal equator line
    painter.line_segment(
        [pos2(center.x - r, center.y), pos2(center.x + r, center.y)],
        stroke,
    );

    // Central vertical prime meridian
    painter.line_segment(
        [pos2(center.x, center.y - r), pos2(center.x, center.y + r)],
        stroke,
    );

    // Eastern & Western curved meridian ellipses (60° parallels)
    let rw = r * 0.52;
    let n_pts = 12;
    let mut east_pts = Vec::with_capacity(n_pts + 1);
    let mut west_pts = Vec::with_capacity(n_pts + 1);

    for i in 0..=n_pts {
        let frac = (i as f32) / (n_pts as f32);
        let angle = -std::f32::consts::FRAC_PI_2 + frac * std::f32::consts::PI;
        let (s, c) = angle.sin_cos();
        east_pts.push(pos2(center.x + c * rw, center.y + s * r));
        west_pts.push(pos2(center.x - c * rw, center.y + s * r));
    }

    let subtle_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.70));
    for win in east_pts.windows(2) {
        painter.line_segment([win[0], win[1]], subtle_stroke);
    }
    for win in west_pts.windows(2) {
        painter.line_segment([win[0], win[1]], subtle_stroke);
    }
}

/// Variables: Mathematical coordinate variable (x) with precision scientific curves (20x18dp keyline).
pub fn draw_variables(painter: &Painter, rect: Rect, stroke: Stroke, _fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Left parenthesis arc '('
    let left_paren = [
        p(5.5, 4.0),
        p(3.8, 7.5),
        p(2.8, 12.0),
        p(3.8, 16.5),
        p(5.5, 20.0),
    ];
    for win in left_paren.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Right parenthesis arc ')'
    let right_paren = [
        p(18.5, 4.0),
        p(20.2, 7.5),
        p(21.2, 12.0),
        p(20.2, 16.5),
        p(18.5, 20.0),
    ];
    for win in right_paren.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Stylized algebraic variable 'x' with scientific serifs
    let diag1 = [
        p(7.5, 8.5),
        p(8.5, 7.0),
        p(12.0, 12.0),
        p(15.5, 17.0),
        p(16.5, 15.5),
    ];
    for win in diag1.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    let diag2 = [
        p(16.5, 8.5),
        p(15.5, 7.0),
        p(12.0, 12.0),
        p(8.5, 17.0),
        p(7.5, 15.5),
    ];
    for win in diag2.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }
}

/// Dimensions: Precision dual-rail slider with futuristic diamond knobs (16x20dp vertical keyline).
pub fn draw_dimensions(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);
    let rail_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.60));

    // Left vertical rail & ticks
    painter.line_segment([p(7.5, 3.5), p(7.5, 20.5)], rail_stroke);
    painter.line_segment([p(5.5, 3.5), p(9.5, 3.5)], rail_stroke);
    painter.line_segment([p(5.5, 20.5), p(9.5, 20.5)], rail_stroke);

    // Right vertical rail & ticks
    painter.line_segment([p(16.5, 3.5), p(16.5, 20.5)], rail_stroke);
    painter.line_segment([p(14.5, 3.5), p(18.5, 3.5)], rail_stroke);
    painter.line_segment([p(14.5, 20.5), p(18.5, 20.5)], rail_stroke);

    // Diamond slider knob 1 (Left rail at y=8.5)
    let k1_c = p(7.5, 8.5);
    let dw = rect.width() * (3.5 / 24.0);
    let dh = rect.height() * (3.0 / 24.0);
    let k1_pts = vec![
        pos2(k1_c.x, k1_c.y - dh),
        pos2(k1_c.x + dw, k1_c.y),
        pos2(k1_c.x, k1_c.y + dh),
        pos2(k1_c.x - dw, k1_c.y),
    ];
    painter.add(egui::Shape::convex_polygon(k1_pts, fill, stroke));

    // Diamond slider knob 2 (Right rail at y=15.5)
    let k2_c = p(16.5, 15.5);
    let k2_pts = vec![
        pos2(k2_c.x, k2_c.y - dh),
        pos2(k2_c.x + dw, k2_c.y),
        pos2(k2_c.x, k2_c.y + dh),
        pos2(k2_c.x - dw, k2_c.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        k2_pts,
        stroke.color.gamma_multiply(0.28),
        stroke,
    ));
}

/// Settings: Precision 6-flange star drive / technical cog with central bore (20dp circle keyline).
pub fn draw_settings(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r_out = rect.width().min(rect.height()) * (10.0 / 24.0);
    let r_in = r_out * 0.44;
    let r_root = r_out * 0.74;
    if r_out <= 1.0 {
        return;
    }

    let teeth = 6;
    let mut cog_pts = Vec::with_capacity(teeth * 4);

    for i in 0..teeth {
        let base_angle = (i as f32) * (std::f32::consts::TAU / (teeth as f32));
        let half_tooth = std::f32::consts::TAU / (teeth as f32 * 4.0);

        // Root entry
        let a0 = base_angle - half_tooth * 1.35;
        cog_pts.push(pos2(
            center.x + a0.cos() * r_root,
            center.y + a0.sin() * r_root,
        ));

        // Tip entry
        let a1 = base_angle - half_tooth * 0.65;
        cog_pts.push(pos2(
            center.x + a1.cos() * r_out,
            center.y + a1.sin() * r_out,
        ));

        // Tip exit
        let a2 = base_angle + half_tooth * 0.65;
        cog_pts.push(pos2(
            center.x + a2.cos() * r_out,
            center.y + a2.sin() * r_out,
        ));

        // Root exit
        let a3 = base_angle + half_tooth * 1.35;
        cog_pts.push(pos2(
            center.x + a3.cos() * r_root,
            center.y + a3.sin() * r_root,
        ));
    }

    painter.add(egui::Shape::convex_polygon(cog_pts, fill, stroke));

    // Center circular bore hole
    painter.circle_stroke(center, r_in, stroke);
    painter.circle_filled(center, r_in * 0.38, stroke.color);
}

/// Cache: High-performance memory die / chip with micro-traces (18x18dp square keyline).
pub fn draw_cache(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Chamfered microchip package body (12x12dp inside 18x18 keyline)
    let chip_pts = vec![
        p(5.5, 7.5),
        p(7.5, 5.5),
        p(18.5, 5.5),
        p(18.5, 18.5),
        p(5.5, 18.5),
    ];
    painter.add(egui::Shape::convex_polygon(chip_pts, fill, stroke));

    // Micro-pin traces along 4 sides reaching 2dp margin
    let pin_stroke = Stroke::new(stroke.width * 0.95, stroke.color);
    let pins = [9.0, 15.0];
    for &offset in &pins {
        // Top pins
        painter.line_segment([p(offset, 2.5), p(offset, 5.5)], pin_stroke);
        // Bottom pins
        painter.line_segment([p(offset, 18.5), p(offset, 21.5)], pin_stroke);
        // Left pins
        painter.line_segment([p(2.5, offset), p(5.5, offset)], pin_stroke);
        // Right pins
        painter.line_segment([p(18.5, offset), p(21.5, offset)], pin_stroke);
    }

    // Inner Silicon core matrix
    let core_rect = Rect::from_min_max(p(9.0, 9.0), p(15.0, 15.0));
    painter.rect_filled(core_rect, 1.0, stroke.color.gamma_multiply(0.30));
    painter.rect_stroke(core_rect, 1.0, stroke, StrokeKind::Inside);
}

/// Sun: Clean solar beacon with radiant cardinal/intercardinal emitter spikes (20dp circle keyline).
pub fn draw_sun(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    let r_core = rect.width().min(rect.height()) * (4.5 / 24.0);
    let r_cardinal_start = r_core * 1.45;
    let r_cardinal_end = rect.width().min(rect.height()) * (10.0 / 24.0);
    let r_inter_start = r_core * 1.40;
    let r_inter_end = rect.width().min(rect.height()) * (8.5 / 24.0);

    // Center radiant emitter core
    painter.circle_stroke(center, r_core, stroke);
    painter.circle_filled(center, r_core * 0.45, stroke.color);

    // 8 Radial precision rays (longer cardinal, shorter diagonal)
    for i in 0..8 {
        let angle = (i as f32) * (std::f32::consts::TAU / 8.0);
        let (s, c) = angle.sin_cos();
        let is_cardinal = i % 2 == 0;
        let (r0, r1) = if is_cardinal {
            (r_cardinal_start, r_cardinal_end)
        } else {
            (r_inter_start, r_inter_end)
        };

        let p_start = pos2(center.x + c * r0, center.y + s * r0);
        let p_end = pos2(center.x + c * r1, center.y + s * r1);
        painter.line_segment([p_start, p_end], stroke);
    }
}

/// Moon: Full lunar orb with subtle crater mare basins (20dp circle keyline).
pub fn draw_moon(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let dim = rect.width().min(rect.height());
    let r = dim * (9.5 / 24.0);
    if r <= 0.5 {
        return;
    }

    // Full circular lunar disc body
    painter.circle(center, r, fill, stroke);

    // Subtle lunar crater mares / impact basins on the lunar surface
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);
    let crater_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.70));
    let crater_fill = stroke.color.gamma_multiply(0.20);

    // Mare Tranquillitatis / Serenetatis crater cluster
    painter.circle(p(9.0, 9.5), dim * (2.4 / 24.0), crater_fill, crater_stroke);
    // Oceanus Procellarum / Tycho basin
    painter.circle(
        p(14.5, 14.0),
        dim * (1.8 / 24.0),
        crater_fill,
        crater_stroke,
    );
    // Mare Crisium
    painter.circle(p(8.0, 15.5), dim * (1.3 / 24.0), crater_fill, crater_stroke);
}

/// Overflow: Precision triple-diamond telemetry markers (Horizontal 20dp alignment).
pub fn draw_overflow(painter: &Painter, rect: Rect, color: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);
    let d = rect.width().min(rect.height()) * (1.8 / 24.0);

    let draw_diamond = |c: Pos2| {
        let pts = vec![
            pos2(c.x, c.y - d),
            pos2(c.x + d, c.y),
            pos2(c.x, c.y + d),
            pos2(c.x - d, c.y),
        ];
        painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
    };

    draw_diamond(p(5.5, 12.0));
    draw_diamond(p(12.0, 12.0));
    draw_diamond(p(18.5, 12.0));
}
