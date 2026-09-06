//! Navigation, panel, and theme procedural vector icons.
//! Minimalistically futuristic and precision-engineered for Octant.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

/// Globe: High-tech planetary geoid with equatorial plane and curved meridian arcs.
pub fn draw_globe(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.40;
    if r <= 0.5 {
        return;
    }

    // Outer perimeter ring
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

    let subtle_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.75));
    for win in east_pts.windows(2) {
        painter.line_segment([win[0], win[1]], subtle_stroke);
    }
    for win in west_pts.windows(2) {
        painter.line_segment([win[0], win[1]], subtle_stroke);
    }
}

/// Variables: Futuristic telemetry histogram with chamfered bars and baseline ruler.
pub fn draw_variables(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Baseline axis with corner endpoints
    painter.line_segment([p(0.12, 0.86), p(0.88, 0.86)], stroke);
    painter.line_segment([p(0.12, 0.80), p(0.12, 0.86)], stroke);
    painter.line_segment([p(0.88, 0.80), p(0.88, 0.86)], stroke);

    // 3 Histogram bars with beveled tops
    let bars = [(0.20, 0.44, 0.16), (0.42, 0.18, 0.16), (0.64, 0.32, 0.16)];

    let base_y = 0.86;
    for (nx, ny, nw) in bars {
        let x0 = nx;
        let x1 = nx + nw;
        let y0 = ny;
        let chamfer = nw * 0.30;

        let pts = vec![
            p(x0, base_y),
            p(x0, y0 + chamfer),
            p(x0 + chamfer, y0),
            p(x1, y0),
            p(x1, base_y),
        ];

        let bar_fill = if nx > 0.40 && nx < 0.50 {
            stroke.color.gamma_multiply(0.25)
        } else {
            fill
        };

        painter.add(egui::Shape::convex_polygon(pts, bar_fill, stroke));
    }
}

/// Dimensions: Precision dual-rail slider with futuristic diamond knobs.
pub fn draw_dimensions(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let rail_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.60));

    // Left vertical rail & ticks
    painter.line_segment([p(0.32, 0.14), p(0.32, 0.86)], rail_stroke);
    painter.line_segment([p(0.24, 0.14), p(0.40, 0.14)], rail_stroke);
    painter.line_segment([p(0.24, 0.86), p(0.40, 0.86)], rail_stroke);

    // Right vertical rail & ticks
    painter.line_segment([p(0.68, 0.14), p(0.68, 0.86)], rail_stroke);
    painter.line_segment([p(0.60, 0.14), p(0.76, 0.14)], rail_stroke);
    painter.line_segment([p(0.60, 0.86), p(0.76, 0.86)], rail_stroke);

    // Futuristic diamond slider knob 1 (Left rail, higher up)
    let k1_c = p(0.32, 0.36);
    let dw = rect.width() * 0.15;
    let dh = rect.height() * 0.13;
    let k1_pts = vec![
        pos2(k1_c.x, k1_c.y - dh),
        pos2(k1_c.x + dw, k1_c.y),
        pos2(k1_c.x, k1_c.y + dh),
        pos2(k1_c.x - dw, k1_c.y),
    ];
    painter.add(egui::Shape::convex_polygon(k1_pts, fill, stroke));

    // Futuristic diamond slider knob 2 (Right rail, lower down)
    let k2_c = p(0.68, 0.64);
    let k2_pts = vec![
        pos2(k2_c.x, k2_c.y - dh),
        pos2(k2_c.x + dw, k2_c.y),
        pos2(k2_c.x, k2_c.y + dh),
        pos2(k2_c.x - dw, k2_c.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        k2_pts,
        stroke.color.gamma_multiply(0.25),
        stroke,
    ));
}

/// Settings: High-tech 6-flange star drive / technical cog with central bore.
pub fn draw_settings(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let center = rect.center();
    let r_out = rect.width().min(rect.height()) * 0.40;
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
        let a0 = base_angle - half_tooth * 1.4;
        cog_pts.push(pos2(
            center.x + a0.cos() * r_root,
            center.y + a0.sin() * r_root,
        ));

        // Tip entry
        let a1 = base_angle - half_tooth * 0.7;
        cog_pts.push(pos2(
            center.x + a1.cos() * r_out,
            center.y + a1.sin() * r_out,
        ));

        // Tip exit
        let a2 = base_angle + half_tooth * 0.7;
        cog_pts.push(pos2(
            center.x + a2.cos() * r_out,
            center.y + a2.sin() * r_out,
        ));

        // Root exit
        let a3 = base_angle + half_tooth * 1.4;
        cog_pts.push(pos2(
            center.x + a3.cos() * r_root,
            center.y + a3.sin() * r_root,
        ));
    }

    painter.add(egui::Shape::convex_polygon(cog_pts, fill, stroke));

    // Center circular bore hole
    painter.circle_stroke(center, r_in, stroke);
    painter.circle_filled(center, r_in * 0.35, stroke.color);
}

/// Cache: High-performance memory die / chip with micro-traces and central core.
pub fn draw_cache(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Chamfered microchip package body
    let chip_pts = vec![
        p(0.24, 0.32),
        p(0.32, 0.24),
        p(0.76, 0.24),
        p(0.76, 0.76),
        p(0.24, 0.76),
    ];
    painter.add(egui::Shape::convex_polygon(chip_pts, fill, stroke));

    // Micro-pin traces along 4 sides
    let pin_stroke = Stroke::new(stroke.width * 0.95, stroke.color);
    let pins = [0.38, 0.62];
    for &offset in &pins {
        // Top pins
        painter.line_segment([p(offset, 0.12), p(offset, 0.24)], pin_stroke);
        // Bottom pins
        painter.line_segment([p(offset, 0.76), p(offset, 0.88)], pin_stroke);
        // Left pins
        painter.line_segment([p(0.12, offset), p(0.24, offset)], pin_stroke);
        // Right pins
        painter.line_segment([p(0.76, offset), p(0.88, offset)], pin_stroke);
    }

    // Inner Silicon core matrix
    let core_rect = Rect::from_min_max(p(0.38, 0.38), p(0.62, 0.62));
    painter.rect_filled(core_rect, 1.0, stroke.color.gamma_multiply(0.30));
    painter.rect_stroke(core_rect, 1.0, stroke, StrokeKind::Inside);
}

/// Sun: Clean solar beacon with radiant cardinal/intercardinal emitter spikes.
pub fn draw_sun(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    let r_core = rect.width().min(rect.height()) * 0.20;
    let r_cardinal_start = r_core * 1.50;
    let r_cardinal_end = rect.width().min(rect.height()) * 0.44;
    let r_inter_start = r_core * 1.45;
    let r_inter_end = rect.width().min(rect.height()) * 0.36;

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

/// Moon: Geometric lunar crescent with precision arc terminator.
pub fn draw_moon(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Smooth crescent polygon
    let pts = vec![
        p(0.72, 0.14),
        p(0.48, 0.18),
        p(0.30, 0.34),
        p(0.24, 0.50),
        p(0.30, 0.66),
        p(0.48, 0.82),
        p(0.72, 0.86),
        p(0.56, 0.74),
        p(0.44, 0.58),
        p(0.44, 0.42),
        p(0.56, 0.26),
    ];

    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));

    // Subtle star dot in empty celestial quadrant
    painter.circle_filled(p(0.68, 0.44), rect.width() * 0.045, stroke.color);
}

/// Overflow: Precision triple-diamond telemetry markers.
pub fn draw_overflow(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let spacing = rect.width() * 0.25;
    let d = rect.width().min(rect.height()) * 0.075;

    let draw_diamond = |c: Pos2| {
        let pts = vec![
            pos2(c.x, c.y - d),
            pos2(c.x + d, c.y),
            pos2(c.x, c.y + d),
            pos2(c.x - d, c.y),
        ];
        painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
    };

    draw_diamond(pos2(center.x - spacing, center.y));
    draw_diamond(center);
    draw_diamond(pos2(center.x + spacing, center.y));
}
