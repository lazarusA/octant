//! Data store, file tree, cache, and filesystem procedural vector icons.
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

/// Dataset: Structured hierarchical data repository platter with coordinate slice strata (20x20dp keyline).
pub fn draw_dataset(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Top elliptical platter
    let top_pts = vec![
        p(12.0, 3.5),
        p(16.5, 4.2),
        p(19.8, 5.8),
        p(20.5, 7.5),
        p(19.8, 9.2),
        p(16.5, 10.8),
        p(12.0, 11.5),
        p(7.5, 10.8),
        p(4.2, 9.2),
        p(3.5, 7.5),
        p(4.2, 5.8),
        p(7.5, 4.2),
    ];
    painter.add(egui::Shape::convex_polygon(top_pts, fill, stroke));

    // Outer cylindrical body hull down to base
    let body_pts = vec![
        p(3.5, 7.5),
        p(3.5, 16.5),
        p(4.2, 18.2),
        p(7.5, 19.8),
        p(12.0, 20.5),
        p(16.5, 19.8),
        p(19.8, 18.2),
        p(20.5, 16.5),
        p(20.5, 7.5),
    ];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Mid-tier dataset partition strata
    let mid_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.70));
    let draw_shelf = |y_base: f32| {
        let pts = [
            p(3.5, y_base),
            p(7.5, y_base + 3.3),
            p(12.0, y_base + 4.0),
            p(16.5, y_base + 3.3),
            p(20.5, y_base),
        ];
        for win in pts.windows(2) {
            painter.line_segment([win[0], win[1]], mid_stroke);
        }
    };
    draw_shelf(8.5);
    draw_shelf(13.5);

    // Front vertical coordinate slice indicator notch
    painter.line_segment([p(12.0, 7.5), p(12.0, 20.5)], mid_stroke);
}

/// Folder: Technical data folder with precision chamfered header tab (20x16dp keyline).
pub fn draw_folder(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    let body_pts = vec![
        p(3.5, 6.0),
        p(9.5, 6.0),
        p(11.5, 8.5),
        p(20.5, 8.5),
        p(20.5, 19.5),
        p(3.5, 19.5),
    ];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Internal structural divider line
    painter.line_segment(
        [p(3.5, 10.0), p(20.5, 10.0)],
        Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.45)),
    );
}

/// FolderOpen: Technical data repository with open holographic intake flap (20x16dp keyline).
pub fn draw_folder_open(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Back tab
    let back_pts = vec![
        p(3.5, 5.0),
        p(9.5, 5.0),
        p(11.5, 7.5),
        p(20.0, 7.5),
        p(20.0, 12.5),
        p(3.5, 12.5),
    ];
    painter.add(egui::Shape::convex_polygon(back_pts, fill, stroke));

    // Front tilted flap
    let front_pts = vec![p(2.5, 19.5), p(6.0, 10.5), p(21.5, 10.5), p(18.0, 19.5)];
    painter.add(egui::Shape::convex_polygon(
        front_pts,
        stroke.color.gamma_multiply(0.28),
        stroke,
    ));
}

/// VariableDoc: Technical schema specification sheet with 45° corner fold (16x20dp keyline).
pub fn draw_variable_doc(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Main document body with dog-ear corner cut
    let sheet_pts = vec![
        p(5.0, 3.5),
        p(13.5, 3.5),
        p(19.0, 9.0),
        p(19.0, 20.5),
        p(5.0, 20.5),
    ];
    painter.add(egui::Shape::convex_polygon(sheet_pts, fill, stroke));

    // Dog-ear corner fold facet
    let fold_pts = vec![p(13.5, 3.5), p(19.0, 9.0), p(13.5, 9.0)];
    painter.add(egui::Shape::convex_polygon(
        fold_pts,
        stroke.color.gamma_multiply(0.35),
        stroke,
    ));

    // Micro schema tracks inside document
    let line_stroke = Stroke::new(stroke.width * 0.9, stroke.color.gamma_multiply(0.65));
    painter.line_segment([p(7.5, 11.5), p(16.5, 11.5)], line_stroke);
    painter.line_segment([p(7.5, 14.5), p(16.5, 14.5)], line_stroke);
    painter.line_segment([p(7.5, 17.5), p(12.5, 17.5)], line_stroke);
}

/// Icechunk: 3D Isometric crystalline facet cluster with sharp specular reflections (18x18dp keyline).
pub fn draw_icechunk(painter: &Painter, rect: Rect, stroke: Stroke, _fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // 3 Visible isometric crystalline facets
    let top_face = vec![p(12.0, 3.0), p(20.0, 7.5), p(12.0, 12.0), p(4.0, 7.5)];
    let left_face = vec![p(4.0, 7.5), p(12.0, 12.0), p(12.0, 21.0), p(4.0, 16.5)];
    let right_face = vec![p(12.0, 12.0), p(20.0, 7.5), p(20.0, 16.5), p(12.0, 21.0)];

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
    painter.line_segment([p(12.0, 3.0), p(12.0, 12.0)], cleave_stroke);
}

/// Catalog: Technical dataset cassette library with vertical spine tracks (18x18dp keyline).
pub fn draw_catalog(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Standing Cartridge 1
    let c1 = Rect::from_min_max(p(4.0, 4.5), p(8.5, 19.5));
    painter.rect(c1, 1.5, fill, stroke, StrokeKind::Inside);
    painter.line_segment(
        [p(6.25, 7.0), p(6.25, 17.0)],
        Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.50)),
    );

    // Standing Cartridge 2
    let c2 = Rect::from_min_max(p(9.5, 3.5), p(14.0, 19.5));
    painter.rect(
        c2,
        1.5,
        stroke.color.gamma_multiply(0.25),
        stroke,
        StrokeKind::Inside,
    );
    painter.line_segment(
        [p(11.75, 6.0), p(11.75, 17.0)],
        Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.65)),
    );

    // Leaning Cartridge 3
    let b3_pts = vec![p(15.5, 7.5), p(19.5, 5.0), p(20.5, 19.5), p(16.5, 19.5)];
    painter.add(egui::Shape::convex_polygon(b3_pts, fill, stroke));
}

/// Save: Technical 3.5" data cartridge with metallic write-shutter (18x18dp keyline).
pub fn draw_save(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Chamfered cartridge chassis
    let disk_pts = vec![
        p(4.0, 4.0),
        p(17.0, 4.0),
        p(20.0, 7.0),
        p(20.0, 20.0),
        p(4.0, 20.0),
    ];
    painter.add(egui::Shape::convex_polygon(disk_pts, fill, stroke));

    // Metallic slider shutter (top)
    let slider = Rect::from_min_max(p(7.5, 4.0), p(16.5, 10.5));
    painter.rect_filled(slider, 1.0, stroke.color.gamma_multiply(0.25));
    painter.rect_stroke(slider, 1.0, stroke, StrokeKind::Inside);

    // Read window slot in shutter
    painter.line_segment([p(10.0, 5.5), p(10.0, 9.0)], stroke);

    // Bottom label window
    let label_rect = Rect::from_min_max(p(6.5, 13.0), p(17.5, 19.5));
    painter.rect_stroke(label_rect, 1.0, stroke, StrokeKind::Inside);
}

/// Snapshot: Futuristic camera viewfinder with prism ridge and precision lens (20x16dp keyline).
pub fn draw_snapshot(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Viewfinder chassis with pentaprism top
    let body_pts = vec![
        p(3.5, 8.0),
        p(7.5, 8.0),
        p(9.5, 5.0),
        p(14.5, 5.0),
        p(16.5, 8.0),
        p(20.5, 8.0),
        p(20.5, 19.5),
        p(3.5, 19.5),
    ];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Central concentric optical lens
    let center = p(12.0, 13.5);
    let r_outer = rect.width() * (4.0 / 24.0);
    let r_inner = rect.width() * (1.8 / 24.0);
    painter.circle_stroke(center, r_outer, stroke);
    painter.circle_filled(center, r_inner, stroke.color);
}

/// DropTray: Inverted data hopper with downward ingestion vector and chamfered cradle (18x18dp keyline).
pub fn draw_drop_tray(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Chamfered container cradle
    let tray_pts = [
        p(4.0, 11.5),
        p(4.0, 19.0),
        p(6.0, 20.0),
        p(18.0, 20.0),
        p(20.0, 19.0),
        p(20.0, 11.5),
    ];
    for win in tray_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Precision downward arrow
    let arrow_stem = [p(12.0, 4.0), p(12.0, 15.0)];
    painter.line_segment(arrow_stem, stroke);

    let arrow_head = [p(7.5, 10.5), p(12.0, 15.0), p(16.5, 10.5)];
    painter.line_segment([arrow_head[0], arrow_head[1]], stroke);
    painter.line_segment([arrow_head[1], arrow_head[2]], stroke);
}

/// Search: Optical sensor scanning reticle with precision focus lens and 45° grip (20x20dp keyline).
pub fn draw_search(painter: &Painter, rect: Rect, stroke: Stroke) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Sensor lens ring (Diameter 12dp)
    let center = p(10.5, 10.5);
    let r = rect.width() * (6.0 / 24.0);
    painter.circle_stroke(center, r, stroke);

    // Center focal point
    painter.circle_filled(center, rect.width() * (1.0 / 24.0), stroke.color);

    // 45-degree angled handle with end chamfer
    let handle_start = p(14.5, 14.5);
    let handle_end = p(20.5, 20.5);
    let handle_stroke = Stroke::new(stroke.width * 1.6, stroke.color);
    painter.line_segment([handle_start, handle_end], handle_stroke);
}

/// Trash: High-tech deallocation incinerator with top sealing flange and vertical ribs (16x18dp keyline).
pub fn draw_trash(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Top sealing flange
    painter.line_segment([p(3.5, 6.0), p(20.5, 6.0)], stroke);
    let handle_pts = [p(9.0, 6.0), p(9.0, 3.5), p(15.0, 3.5), p(15.0, 6.0)];
    for win in handle_pts.windows(2) {
        painter.line_segment([win[0], win[1]], stroke);
    }

    // Tapered canister body
    let body_pts = vec![p(5.5, 6.0), p(18.5, 6.0), p(17.0, 20.0), p(7.0, 20.0)];
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Vertical dissipation ribs
    let flute_stroke = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.60));
    painter.line_segment([p(9.5, 8.5), p(9.5, 17.5)], flute_stroke);
    painter.line_segment([p(12.0, 8.5), p(12.0, 17.5)], flute_stroke);
    painter.line_segment([p(14.5, 8.5), p(14.5, 17.5)], flute_stroke);
}

/// Clipboard: Digital manifest tablet with top clamping bus and data track lines (16x20dp keyline).
pub fn draw_clipboard(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |gx: f32, gy: f32| grid_p(rect, gx, gy);

    // Tablet body with chamfered bottom corners
    let pad_pts = vec![
        p(4.5, 5.0),
        p(19.5, 5.0),
        p(19.5, 19.0),
        p(18.0, 20.5),
        p(6.0, 20.5),
        p(4.5, 19.0),
    ];
    painter.add(egui::Shape::convex_polygon(pad_pts, fill, stroke));

    // Top metal clip connector
    let clip = Rect::from_min_max(p(8.5, 3.5), p(15.5, 7.0));
    painter.rect_filled(clip, 2.0, stroke.color.gamma_multiply(0.30));
    painter.rect_stroke(clip, 2.0, stroke, StrokeKind::Inside);

    // Digital manifest lines
    let line_stroke = Stroke::new(stroke.width * 0.9, stroke.color.gamma_multiply(0.65));
    painter.line_segment([p(7.5, 10.5), p(16.5, 10.5)], line_stroke);
    painter.line_segment([p(7.5, 13.5), p(16.5, 13.5)], line_stroke);
    painter.line_segment([p(7.5, 16.5), p(13.5, 16.5)], line_stroke);
}
