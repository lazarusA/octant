//! Dataset and folder icons.

use super::canvas::{IconCanvas, Shade, Weight, key};
use egui::Pos2;
use std::f32::consts::{PI, TAU};

/// Folder outline from the top-left corner across the tab to the top-right
/// corner, shared by the closed and open folders so they stay one silhouette.
const FOLDER_TAB: [(f32, f32); 4] = [
    (key::SQ_MIN, 4.5),
    (9.0, 4.5),
    (11.0, 7.0),
    (key::SQ_MAX, 7.0),
];
const FOLDER_BOTTOM: f32 = 19.5;

/// Dataset cylinder: half width, ellipse half height, cap and base centers.
const CYL_RX: f32 = 8.0;
const CYL_RY: f32 = 2.75;
const CYL_TOP: f32 = 6.5;
const CYL_BOTTOM: f32 = 17.5;

fn cyl_arc(c: &IconCanvas, cy: f32, a0: f32, a1: f32) -> Vec<Pos2> {
    c.arc((key::C, cy), (CYL_RX, CYL_RY), a0, a1)
}

/// Dataset: database cylinder with a lit cap and strata bands, one when compact (16x18 keyline).
pub fn draw_dataset(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    // Silhouette: back half of the cap, then the front half of the base.
    let mut silhouette = cyl_arc(c, CYL_TOP, PI, TAU);
    silhouette.extend(cyl_arc(c, CYL_BOTTOM, 0.0, PI));
    c.fill_pts(silhouette.clone(), c.body());
    c.fill_pts(cyl_arc(c, CYL_TOP, 0.0, TAU), c.shade(Shade::Faint));
    c.curve_closed(silhouette, s);
    c.curve(cyl_arc(c, CYL_TOP, 0.0, PI), s);

    // Strata bands echo the front rim at even spacing down the body.
    let band = c.detail(Shade::Strong);
    let bands = if c.compact() { 1 } else { 2 };
    let pitch = (CYL_BOTTOM - CYL_TOP) / (bands + 1) as f32;
    for k in 1..=bands {
        c.curve(cyl_arc(c, CYL_TOP + pitch * k as f32, 0.0, PI), band);
    }
}

/// Fill the folder tab on its own: the step down from the tab makes the
/// folder outline concave, so the body below is filled separately.
fn fill_folder_tab(c: &IconCanvas) {
    let [tl, tab_end, tab_step, _] = FOLDER_TAB;
    c.fill(
        &[tl, tab_end, tab_step, (key::SQ_MIN, tab_step.1)],
        c.body(),
    );
}

/// Folder: closed folder with a chamfered tab and a lid divider (20x15 keyline).
pub fn draw_folder(c: &IconCanvas) {
    let [tl, tab_end, tab_step, tr] = FOLDER_TAB;
    let (lo, hi) = (key::SQ_MIN, key::SQ_MAX);
    fill_folder_tab(c);
    c.fill(
        &[(lo, tr.1), tr, (hi, FOLDER_BOTTOM), (lo, FOLDER_BOTTOM)],
        c.body(),
    );
    c.closed(
        &[
            tl,
            tab_end,
            tab_step,
            tr,
            (hi, FOLDER_BOTTOM),
            (lo, FOLDER_BOTTOM),
        ],
        c.stroke(Weight::Base),
    );
    if !c.compact() {
        c.line((lo, 9.5), (hi, 9.5), c.detail(Shade::Mid));
    }
}

/// FolderOpen: folder back with its front flap swung forward to show the opening (20x15 keyline).
pub fn draw_folder_open(c: &IconCanvas) {
    let [tl, tab_end, tab_step, _] = FOLDER_TAB;
    let lo = key::SQ_MIN;
    // Back panel stops short of the tab's right edge where the flap top begins.
    let (back_right, flap_top) = (19.0, 10.5);
    let base_left = (lo, FOLDER_BOTTOM);
    let flap_tl = (6.5, flap_top);
    let flap_tr = (22.0, flap_top);
    let flap_br = (19.0, FOLDER_BOTTOM);

    // Back panel fills: tab, band above the flap, and the sliver left of the flap.
    fill_folder_tab(c);
    c.fill(
        &[
            (lo, tab_step.1),
            (back_right, tab_step.1),
            (back_right, flap_top),
            (lo, flap_top),
        ],
        c.body(),
    );
    c.fill(&[(lo, flap_top), flap_tl, base_left], c.body());
    c.fill(
        &[flap_tl, flap_tr, flap_br, base_left],
        c.shade(Shade::Faint),
    );

    let s = c.stroke(Weight::Base);
    c.closed(
        &[
            base_left,
            tl,
            tab_end,
            tab_step,
            (back_right, tab_step.1),
            (back_right, flap_top),
            flap_tr,
            flap_br,
        ],
        s,
    );
    c.path(&[base_left, flap_tl, (back_right, flap_top)], s);
}
