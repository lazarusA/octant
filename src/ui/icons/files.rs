//! Dataset and folder procedural vector icons.
//! Precision-engineered for Octant following standardized 24-unit geometric keylines.

use super::grid_p;
use egui::{Color32, Painter, Pos2, Rect, Stroke};
use std::f32::consts::PI;

/// Folder back outline from the top-left corner across the tab to the top-right
/// corner, shared by the closed and open folders so they stay one silhouette.
const FOLDER_TAB: [(f32, f32); 4] = [(3.5, 6.0), (9.5, 6.0), (11.5, 8.5), (20.5, 8.5)];

/// Dataset cylinder: center x, half width, ellipse half height, cap and base centers.
const CYL_CX: f32 = 12.0;
const CYL_RX: f32 = 8.0;
const CYL_RY: f32 = 2.75;
const CYL_TOP: f32 = 6.5;
const CYL_BOTTOM: f32 = 17.5;

/// Points of the cylinder ellipse centered at height `cy`, from angle `a0` to `a1`
/// (0 = right, PI/2 = front, PI = left, 3PI/2 = back).
fn ellipse_arc(rect: Rect, cy: f32, a0: f32, a1: f32) -> impl Iterator<Item = Pos2> {
    const STEPS: usize = 12;
    (0..=STEPS).map(move |i| {
        let a = a0 + (a1 - a0) * (i as f32 / STEPS as f32);
        grid_p(rect, CYL_CX + CYL_RX * a.cos(), cy + CYL_RY * a.sin())
    })
}

/// Dataset: Crisp database cylinder with a lit cap and two strata bands (16x18dp keyline).
pub fn draw_dataset(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    // Silhouette: back half of the cap, then the front half of the base.
    let silhouette: Vec<Pos2> = ellipse_arc(rect, CYL_TOP, PI, 2.0 * PI)
        .chain(ellipse_arc(rect, CYL_BOTTOM, 0.0, PI))
        .collect();
    painter.add(egui::Shape::convex_polygon(
        silhouette.clone(),
        fill,
        Stroke::NONE,
    ));

    // Lit cap, filled only; its rim is part of the single outline below.
    let cap: Vec<Pos2> = ellipse_arc(rect, CYL_TOP, 0.0, 2.0 * PI).collect();
    painter.add(egui::Shape::convex_polygon(
        cap,
        stroke.color.gamma_multiply(0.18),
        Stroke::NONE,
    ));

    painter.add(egui::Shape::closed_line(silhouette, stroke));
    painter.add(egui::Shape::line(
        ellipse_arc(rect, CYL_TOP, 0.0, PI).collect(),
        stroke,
    ));

    // Strata bands echo the front rim at even spacing down the body.
    let band = Stroke::new(stroke.width * 0.85, stroke.color.gamma_multiply(0.70));
    let pitch = (CYL_BOTTOM - CYL_TOP) / 3.0;
    for k in 1..=2 {
        let cy = CYL_TOP + pitch * k as f32;
        painter.add(egui::Shape::line(
            ellipse_arc(rect, cy, 0.0, PI).collect(),
            band,
        ));
    }
}

/// Folder: Technical data folder with precision chamfered header tab (20x16dp keyline).
pub fn draw_folder(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |(gx, gy): (f32, f32)| grid_p(rect, gx, gy);

    let mut body_pts: Vec<Pos2> = FOLDER_TAB.into_iter().map(p).collect();
    body_pts.extend([p((20.5, 19.5)), p((3.5, 19.5))]);
    painter.add(egui::Shape::convex_polygon(body_pts, fill, stroke));

    // Internal structural divider line
    painter.line_segment(
        [p((3.5, 10.0)), p((20.5, 10.0))],
        Stroke::new(stroke.width * 0.8, stroke.color.gamma_multiply(0.45)),
    );
}

/// FolderOpen: Folder back with its front flap swung forward to show the opening (20x16dp keyline).
pub fn draw_folder_open(painter: &Painter, rect: Rect, stroke: Stroke, fill: Color32) {
    let p = |(gx, gy): (f32, f32)| grid_p(rect, gx, gy);
    // Back panel stops short of the tab's right edge where the flap top begins.
    let (back_right, flap_top) = (19.5, 11.5);
    let base_left = p((3.5, 19.5));
    let flap_tl = p((7.5, flap_top));
    let flap_tr = p((21.5, flap_top));
    let flap_br = p((18.5, 19.5));

    // Back panel fills: tab band above the flap, and the sliver left of the flap.
    let mut back_top: Vec<Pos2> = FOLDER_TAB[..3].iter().copied().map(p).collect();
    back_top.extend([
        p((back_right, 8.5)),
        p((back_right, flap_top)),
        p((3.5, flap_top)),
    ]);
    painter.add(egui::Shape::convex_polygon(back_top, fill, Stroke::NONE));
    painter.add(egui::Shape::convex_polygon(
        vec![p((3.5, flap_top)), flap_tl, base_left],
        fill,
        Stroke::NONE,
    ));

    // Front flap, a shade stronger as the nearest face.
    painter.add(egui::Shape::convex_polygon(
        vec![flap_tl, flap_tr, flap_br, base_left],
        stroke.color.gamma_multiply(0.18),
        Stroke::NONE,
    ));

    // One continuous outline around the whole folder, then the flap's inner edges.
    let mut outline: Vec<Pos2> = vec![base_left];
    outline.extend(FOLDER_TAB[..3].iter().copied().map(p));
    outline.extend([
        p((back_right, 8.5)),
        p((back_right, flap_top)),
        flap_tr,
        flap_br,
    ]);
    painter.add(egui::Shape::closed_line(outline, stroke));
    painter.add(egui::Shape::line(
        vec![base_left, flap_tl, p((back_right, flap_top))],
        stroke,
    ));
}
