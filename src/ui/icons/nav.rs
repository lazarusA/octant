//! Navigation, panel, and theme procedural vector icons.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2, vec2};

pub fn draw_globe(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.42;
    if r <= 0.5 {
        return;
    }

    // Outer circle
    painter.circle_stroke(center, r, stroke);

    // Horizontal equator
    painter.line_segment(
        [pos2(center.x - r, center.y), pos2(center.x + r, center.y)],
        stroke,
    );

    // Vertical central meridian
    painter.line_segment(
        [pos2(center.x, center.y - r), pos2(center.x, center.y + r)],
        stroke,
    );

    // Curved latitude parallels (subtle arcs top & bottom)
    let lat_y = r * 0.48;
    let lat_w = (r * r - lat_y * lat_y).max(0.0).sqrt();
    let p_lat_top = [
        pos2(center.x - lat_w, center.y - lat_y),
        pos2(center.x, center.y - lat_y * 0.4),
        pos2(center.x + lat_w, center.y - lat_y),
    ];
    let p_lat_bot = [
        pos2(center.x - lat_w, center.y + lat_y),
        pos2(center.x, center.y + lat_y * 0.4),
        pos2(center.x + lat_w, center.y + lat_y),
    ];
    painter.line_segment([p_lat_top[0], p_lat_top[1]], stroke);
    painter.line_segment([p_lat_top[1], p_lat_top[2]], stroke);
    painter.line_segment([p_lat_bot[0], p_lat_bot[1]], stroke);
    painter.line_segment([p_lat_bot[1], p_lat_bot[2]], stroke);
}

pub fn draw_variables(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Baseline axis
    painter.line_segment([p(0.12, 0.88), p(0.88, 0.88)], stroke);

    // 3 Histogram / Variable bars with differing heights
    let bar_w = rect.width() * 0.16;
    let bar_rad = 1.2;

    let bars = [(0.22, 0.48), (0.44, 0.18), (0.66, 0.36)];

    for (nx, ny) in bars {
        let top_left = p(nx, ny);
        let bot_right = pos2(top_left.x + bar_w, p(0.0, 0.88).y);
        let bar_rect = Rect::from_min_max(top_left, bot_right);
        painter.rect(bar_rect, bar_rad, fill, stroke, StrokeKind::Inside);
    }
}

pub fn draw_dimensions(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // 2 vertical slider rails
    painter.line_segment([p(0.32, 0.14), p(0.32, 0.86)], stroke);
    painter.line_segment([p(0.68, 0.14), p(0.68, 0.86)], stroke);

    // Slider 1 knob (higher position)
    let k1_center = p(0.32, 0.38);
    let knob_size = vec2(rect.width() * 0.28, rect.height() * 0.16);
    let k1_rect = Rect::from_center_size(k1_center, knob_size);
    painter.rect(k1_rect, 2.0, fill, stroke, StrokeKind::Inside);

    // Slider 2 knob (lower position)
    let k2_center = p(0.68, 0.62);
    let k2_rect = Rect::from_center_size(k2_center, knob_size);
    painter.rect(k2_rect, 2.0, fill, stroke, StrokeKind::Inside);
}

pub fn draw_settings(painter: &Painter, rect: Rect, stroke: Stroke, _fill: Color32) {
    let center = rect.center();
    let r_out = rect.width().min(rect.height()) * 0.40;
    let r_in = r_out * 0.45;
    if r_out <= 1.0 {
        return;
    }

    // Gear teeth: 6 teeth
    let teeth = 6;
    let tooth_len = r_out * 0.22;
    for i in 0..teeth {
        let angle = (i as f32) * (std::f32::consts::TAU / (teeth as f32));
        let (s, c) = angle.sin_cos();
        let p_start = pos2(
            center.x + c * (r_out - tooth_len * 0.5),
            center.y + s * (r_out - tooth_len * 0.5),
        );
        let p_end = pos2(
            center.x + c * (r_out + tooth_len * 0.5),
            center.y + s * (r_out + tooth_len * 0.5),
        );
        let tooth_stroke = Stroke::new(stroke.width * 1.6, stroke.color);
        painter.line_segment([p_start, p_end], tooth_stroke);
    }

    // Outer gear body ring
    painter.circle_stroke(center, r_out - tooth_len * 0.3, stroke);

    // Inner center axle hole
    painter.circle_stroke(center, r_in, stroke);
}

pub fn draw_cache(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Central chip rectangle
    let chip_rect = Rect::from_min_max(p(0.26, 0.26), p(0.74, 0.74));
    painter.rect(chip_rect, 2.0, fill, stroke, StrokeKind::Inside);

    // Microchip pins (top, bottom, left, right)
    let pin_stroke = Stroke::new(stroke.width, stroke.color);
    let pins = [0.38, 0.62];
    for &offset in &pins {
        // Top pins
        painter.line_segment([p(offset, 0.12), p(offset, 0.26)], pin_stroke);
        // Bottom pins
        painter.line_segment([p(offset, 0.74), p(offset, 0.88)], pin_stroke);
        // Left pins
        painter.line_segment([p(0.12, offset), p(0.26, offset)], pin_stroke);
        // Right pins
        painter.line_segment([p(0.74, offset), p(0.88, offset)], pin_stroke);
    }

    // Center node dot
    painter.circle_filled(rect.center(), rect.width() * 0.08, stroke.color);
}

pub fn draw_sun(painter: &Painter, rect: Rect, stroke: Stroke) {
    let center = rect.center();
    let r_core = rect.width().min(rect.height()) * 0.20;
    let r_ray_start = r_core * 1.55;
    let r_ray_end = rect.width().min(rect.height()) * 0.44;

    // Sun core circle
    painter.circle_stroke(center, r_core, stroke);

    // 8 Radial rays
    for i in 0..8 {
        let angle = (i as f32) * (std::f32::consts::TAU / 8.0);
        let (s, c) = angle.sin_cos();
        let p_start = pos2(center.x + c * r_ray_start, center.y + s * r_ray_start);
        let p_end = pos2(center.x + c * r_ray_end, center.y + s * r_ray_end);
        painter.line_segment([p_start, p_end], stroke);
    }
}

pub fn draw_moon(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Crescent moon path
    let points = vec![
        p(0.70, 0.14),
        p(0.48, 0.18),
        p(0.30, 0.34),
        p(0.24, 0.52),
        p(0.30, 0.70),
        p(0.48, 0.84),
        p(0.70, 0.88),
        p(0.56, 0.76),
        p(0.46, 0.60),
        p(0.46, 0.42),
        p(0.56, 0.26),
    ];

    painter.add(egui::Shape::convex_polygon(points, fill, stroke));
}

pub fn draw_overflow(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let spacing = rect.width() * 0.24;
    let r = rect.width().min(rect.height()) * 0.08;

    painter.circle_filled(pos2(center.x - spacing, center.y), r, color);
    painter.circle_filled(center, r, color);
    painter.circle_filled(pos2(center.x + spacing, center.y), r, color);
}
