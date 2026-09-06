//! Data store, file tree, cache, and filesystem procedural vector icons.
//! Minimalistically futuristic and precision-engineered for Octant.

use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2};

/// Folder: Technical data folder with precision chamfered header tab.
pub fn draw_folder(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    let body_pts = vec![
        p(0.14, 0.28),
        p(0.40, 0.28),
        p(0.48, 0.38),
        p(0.86, 0.38),
        p(0.86, 0.82),
        p(0.14, 0.82),
    ];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Internal structural divider line
    painter.line_segment(
        [p(0.14, 0.42), p(0.86, 0.42)],
        Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.45)),
    );
}

/// FolderOpen: Technical data repository with open holographic intake flap.
pub fn draw_folder_open(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Back tab
    let back_pts = vec![
        p(0.14, 0.24),
        p(0.40, 0.24),
        p(0.48, 0.34),
        p(0.84, 0.34),
        p(0.84, 0.52),
        p(0.14, 0.52),
    ];
    painter.add(egui::Shape::convex_polygon(back_pts, fill, stroke));

    // Front tilted flap
    let front_pts = vec![p(0.10, 0.82), p(0.24, 0.44), p(0.90, 0.44), p(0.76, 0.82)];
    painter.add(egui::Shape::convex_polygon(
        front_pts,
        stroke.color.gamma_multiply(0.25),
        stroke,
    ));
}

/// VariableDoc: Technical schema specification sheet with precision 45° corner fold.
pub fn draw_variable_doc(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Main document body with dog-ear corner cut
    let sheet_pts = vec![
        p(0.22, 0.16),
        p(0.56, 0.16),
        p(0.78, 0.38),
        p(0.78, 0.84),
        p(0.22, 0.84),
    ];
    painter.add(egui::Shape::convex_polygon(sheet_pts, fill, stroke));

    // Dog-ear corner fold facet
    let fold_pts = vec![p(0.56, 0.16), p(0.78, 0.38), p(0.56, 0.38)];
    painter.add(egui::Shape::convex_polygon(
        fold_pts,
        stroke.color.gamma_multiply(0.35),
        stroke,
    ));

    // Micro schema tracks inside document
    let line_stroke = Stroke::new(stroke.width * 0.9, stroke.color.gamma_multiply(0.65));
    painter.line_segment([p(0.32, 0.48), p(0.68, 0.48)], line_stroke);
    painter.line_segment([p(0.32, 0.60), p(0.68, 0.60)], line_stroke);
    painter.line_segment([p(0.32, 0.72), p(0.52, 0.72)], line_stroke);
}

/// Icechunk: 3D Isometric crystalline facet cluster with sharp specular reflections.
pub fn draw_icechunk(painter: &Painter, rect: Rect, stroke: Stroke, _fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // 3 Visible isometric crystalline facets
    let top_face = vec![p(0.50, 0.14), p(0.84, 0.32), p(0.50, 0.50), p(0.16, 0.32)];
    let left_face = vec![p(0.16, 0.32), p(0.50, 0.50), p(0.50, 0.88), p(0.16, 0.70)];
    let right_face = vec![p(0.50, 0.50), p(0.84, 0.32), p(0.84, 0.70), p(0.50, 0.88)];

    let base = stroke.color;
    painter.add(egui::Shape::convex_polygon(
        top_face,
        base.gamma_multiply(0.55),
        stroke,
    ));
    painter.add(egui::Shape::convex_polygon(
        left_face,
        base.gamma_multiply(0.20),
        stroke,
    ));
    painter.add(egui::Shape::convex_polygon(
        right_face,
        base.gamma_multiply(0.35),
        stroke,
    ));

    // Internal crystalline cleavage line on top facet
    let cleave_stroke = Stroke::new(stroke.width * 0.75, stroke.color.gamma_multiply(0.80));
    painter.line_segment([p(0.50, 0.14), p(0.50, 0.50)], cleave_stroke);
}

/// Catalog: Technical dataset cassette library with vertical spine tracks.
pub fn draw_catalog(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Standing Cartridge 1
    let c1 = Rect::from_min_max(p(0.16, 0.20), p(0.36, 0.82));
    painter.rect(c1, 1.5, fill, stroke, StrokeKind::Inside);
    painter.line_segment(
        [p(0.26, 0.30), p(0.26, 0.72)],
        Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.50)),
    );

    // Standing Cartridge 2
    let c2 = Rect::from_min_max(p(0.40, 0.16), p(0.60, 0.82));
    painter.rect(
        c2,
        1.5,
        stroke.color.gamma_multiply(0.25),
        stroke,
        StrokeKind::Inside,
    );
    painter.line_segment(
        [p(0.50, 0.26), p(0.50, 0.72)],
        Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.65)),
    );

    // Leaning Cartridge 3
    let b3_pts = vec![p(0.64, 0.32), p(0.82, 0.22), p(0.86, 0.82), p(0.68, 0.82)];
    painter.add(egui::Shape::convex_polygon(b3_pts, fill, stroke));
}

/// Save: Technical 3.5" data cartridge with metallic write-shutter.
pub fn draw_save(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Chamfered cartridge chassis
    let disk_pts = vec![
        p(0.16, 0.16),
        p(0.72, 0.16),
        p(0.84, 0.28),
        p(0.84, 0.84),
        p(0.16, 0.84),
    ];
    painter.add(egui::Shape::convex_polygon(disk_pts, fill, stroke));

    // Metallic slider shutter (top)
    let slider = Rect::from_min_max(p(0.32, 0.16), p(0.68, 0.44));
    painter.rect_filled(slider, 1.0, stroke.color.gamma_multiply(0.25));
    painter.rect_stroke(slider, 1.0, stroke, StrokeKind::Inside);

    // Read window slot in shutter
    painter.line_segment([p(0.42, 0.22), p(0.42, 0.38)], stroke);

    // Bottom label window
    let label_rect = Rect::from_min_max(p(0.26, 0.54), p(0.74, 0.84));
    painter.rect_stroke(label_rect, 1.0, stroke, StrokeKind::Inside);
}

/// Snapshot: Futuristic camera viewfinder with prism ridge and precision lens.
pub fn draw_snapshot(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Viewfinder chassis with pentaprism top
    let body_pts = vec![
        p(0.14, 0.34),
        p(0.32, 0.34),
        p(0.40, 0.20),
        p(0.60, 0.20),
        p(0.68, 0.34),
        p(0.86, 0.34),
        p(0.86, 0.80),
        p(0.14, 0.80),
    ];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Central concentric optical lens
    let center = p(0.50, 0.57);
    let r_outer = rect.width() * 0.17;
    let r_inner = rect.width() * 0.08;
    painter.circle_stroke(center, r_outer, stroke);
    painter.circle_filled(center, r_inner, stroke.color);
}

/// DropTray: Inverted data hopper with downward ingestion vector and chamfered cradle.
pub fn draw_drop_tray(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Chamfered container cradle
    let tray_pts = [
        p(0.16, 0.48),
        p(0.16, 0.80),
        p(0.24, 0.84),
        p(0.76, 0.84),
        p(0.84, 0.80),
        p(0.84, 0.48),
    ];
    for win in tray_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Precision downward arrow
    let arrow_stem = [p(0.50, 0.16), p(0.50, 0.62)];
    painter.line_segment(arrow_stem, stroke);

    let arrow_head = [p(0.32, 0.44), p(0.50, 0.62), p(0.68, 0.44)];
    painter.line_segment([arrow_head[0], arrow_head[1]], stroke);
    painter.line_segment([arrow_head[1], arrow_head[2]], stroke);
}

/// Search: Optical sensor scanning reticle with precision focus lens and 45° grip.
pub fn draw_search(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Sensor lens ring
    let center = p(0.44, 0.44);
    let r = rect.width() * 0.24;
    painter.circle_stroke(center, r, stroke);

    // Center focal point
    painter.circle_filled(center, rect.width() * 0.04, stroke.color);

    // 45-degree angled handle with end chamfer
    let handle_start = p(0.61, 0.61);
    let handle_end = p(0.85, 0.85);
    let handle_stroke = Stroke::new(stroke.width * 1.6, stroke.color);
    painter.line_segment([handle_start, handle_end], handle_stroke);
}

/// Trash: High-tech deallocation incinerator with top sealing flange and vertical ribs.
pub fn draw_trash(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Top sealing flange
    painter.line_segment([p(0.14, 0.26), p(0.86, 0.26)], stroke);
    let handle_pts = [p(0.38, 0.26), p(0.38, 0.16), p(0.62, 0.16), p(0.62, 0.26)];
    for win in handle_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Tapered canister body
    let body_pts = vec![p(0.22, 0.26), p(0.78, 0.26), p(0.72, 0.84), p(0.28, 0.84)];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Vertical dissipation ribs
    let flute_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.60));
    painter.line_segment([p(0.40, 0.36), p(0.40, 0.74)], flute_stroke);
    painter.line_segment([p(0.50, 0.36), p(0.50, 0.74)], flute_stroke);
    painter.line_segment([p(0.60, 0.36), p(0.60, 0.74)], flute_stroke);
}

/// Clipboard: Digital manifest tablet with top clamping bus and data track lines.
pub fn draw_clipboard(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |nx: f32, ny: f32| -> Pos2 {
        pos2(
            rect.min.x + nx * rect.width(),
            rect.min.y + ny * rect.height(),
        )
    };

    // Tablet body with chamfered bottom corners
    let pad_pts = vec![
        p(0.18, 0.20),
        p(0.82, 0.20),
        p(0.82, 0.80),
        p(0.76, 0.86),
        p(0.24, 0.86),
        p(0.18, 0.80),
    ];
    painter.add(egui::Shape::convex_polygon(pad_pts, fill, stroke));

    // Top metal clip connector
    let clip = Rect::from_min_max(p(0.34, 0.14), p(0.66, 0.28));
    painter.rect_filled(clip, 2.0, stroke.color.gamma_multiply(0.30));
    painter.rect_stroke(clip, 2.0, stroke, StrokeKind::Inside);

    // Digital manifest lines
    let line_stroke = Stroke::new(stroke.width * 0.9, stroke.color.gamma_multiply(0.65));
    painter.line_segment([p(0.30, 0.44), p(0.70, 0.44)], line_stroke);
    painter.line_segment([p(0.30, 0.58), p(0.70, 0.58)], line_stroke);
    painter.line_segment([p(0.30, 0.72), p(0.56, 0.72)], line_stroke);
}
