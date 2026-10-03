//! Data store and file action icons.

use super::canvas::{IconCanvas, Shade, Weight, key};
use egui::Color32;

/// Text lines inside documents: three at full size, two when compact.
fn text_lines(c: &IconCanvas, x0: f32, full: [(f32, f32); 3], compact: [(f32, f32); 2]) {
    let s = c.detail(Shade::Strong);
    let lines: &[(f32, f32)] = if c.compact() { &compact } else { &full };
    for &(y, x1) in lines {
        c.line((x0, y), (x1, y), s);
    }
}

/// VariableDoc: document with a folded corner and text lines (14x18 keyline).
pub fn draw_variable_doc(c: &IconCanvas) {
    let sheet = [
        (5.0, 3.0),
        (14.0, 3.0),
        (19.0, 8.0),
        (19.0, 21.0),
        (5.0, 21.0),
    ];
    c.fill(&sheet, c.body());
    c.fill(
        &[(14.0, 3.0), (19.0, 8.0), (14.0, 8.0)],
        c.shade(Shade::Soft),
    );
    let s = c.stroke(Weight::Base);
    c.closed(&sheet, s);
    c.path(&[(14.0, 3.0), (14.0, 8.0), (19.0, 8.0)], s);
    text_lines(
        c,
        8.0,
        [(12.0, 16.0), (15.0, 16.0), (18.0, 13.0)],
        [(13.0, 16.0), (17.0, 13.0)],
    );
}

/// Icechunk: faceted crystal with a shaded crown, distinct from the voxel cube (18x18 keyline).
pub fn draw_icechunk(c: &IconCanvas) {
    let (tl, tr, r, bot, l) = (
        (7.0, 3.5),
        (17.0, 3.5),
        (21.0, 9.0),
        (12.0, 21.0),
        (3.0, 9.0),
    );
    let (gl, gr) = (9.5, 14.5);
    c.fill(&[tl, tr, r, bot, l], c.body());
    c.fill(&[tl, tr, r, l], c.shade(Shade::Soft));
    c.fill(&[(gl, r.1), (gr, r.1), bot], c.shade(Shade::Faint));

    let s = c.stroke(Weight::Base);
    c.closed(&[tl, tr, r, bot, l], s);
    c.line(l, r, s);
    let facet = c.detail(Shade::Strong);
    c.line(tl, (gl, r.1), facet);
    c.line(tr, (gr, r.1), facet);
    if !c.compact() {
        c.line((gl, r.1), bot, facet);
        c.line((gr, r.1), bot, facet);
    }
}

/// Catalog: three volumes on a shelf, the last one leaning (18x17 keyline).
pub fn draw_catalog(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    c.rrect((4.0, 5.0), (8.0, 20.0), 1.0, c.body(), s);
    c.rrect((9.5, 3.5), (13.5, 20.0), 1.0, c.shade(Shade::Soft), s);
    let lean = [(15.0, 7.5), (18.8, 5.5), (21.0, 20.0), (17.0, 20.0)];
    c.polygon(&lean, c.body(), s);
}

/// Save: floppy disk with shutter and label, each edge drawn once (16x16 keyline).
pub fn draw_save(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let chassis = [
        (4.0, 4.0),
        (17.0, 4.0),
        (20.0, 7.0),
        (20.0, 20.0),
        (4.0, 20.0),
    ];
    c.fill(&chassis, c.body());
    c.fill(
        &[(7.5, 4.0), (16.5, 4.0), (16.5, 10.0), (7.5, 10.0)],
        c.shade(Shade::Soft),
    );
    c.closed(&chassis, s);
    // Shutter and label reuse the chassis edge as their top and bottom.
    c.path(&[(7.5, 4.0), (7.5, 10.0), (16.5, 10.0), (16.5, 4.0)], s);
    c.path(&[(7.0, 20.0), (7.0, 13.5), (17.0, 13.5), (17.0, 20.0)], s);
    if !c.compact() {
        c.line((13.5, 5.5), (13.5, 8.5), s);
    }
}

/// Snapshot: camera body with a viewfinder hump and lens (18x15 keyline).
pub fn draw_snapshot(c: &IconCanvas) {
    let (lo, hi) = (key::SQ_MIN, key::SQ_MAX);
    // The hump makes the body concave: fill the body and the hump separately.
    c.fill(&[(lo, 8.0), (hi, 8.0), (hi, 20.0), (lo, 20.0)], c.body());
    c.fill(
        &[(7.5, 8.0), (9.5, 5.0), (14.5, 5.0), (16.5, 8.0)],
        c.body(),
    );
    let s = c.stroke(Weight::Base);
    let body = [
        (lo, 8.0),
        (7.5, 8.0),
        (9.5, 5.0),
        (14.5, 5.0),
        (16.5, 8.0),
        (hi, 8.0),
        (hi, 20.0),
        (lo, 20.0),
    ];
    c.closed(&body, s);
    if c.compact() {
        c.circle((12.0, 14.0), 3.5, Color32::TRANSPARENT, s);
    } else {
        c.circle((12.0, 14.0), 4.0, Color32::TRANSPARENT, s);
        c.dot((12.0, 14.0), 1.5, c.color);
    }
}

/// DropTray: open tray with a downward arrow (16x17 keyline).
pub fn draw_drop_tray(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    c.path(&[(4.0, 12.0), (4.0, 20.0), (20.0, 20.0), (20.0, 12.0)], s);
    c.line((12.0, 3.5), (12.0, 15.0), s);
    c.path(&[(7.5, 10.5), (12.0, 15.0), (16.5, 10.5)], s);
}

/// Search: lens ring and a bold handle with a round end (18x18 keyline).
pub fn draw_search(c: &IconCanvas) {
    c.circle(
        (10.5, 10.5),
        6.5,
        Color32::TRANSPARENT,
        c.stroke(Weight::Base),
    );
    c.path_round(&[(15.3, 15.3), (20.5, 20.5)], c.stroke(Weight::Bold));
}

/// Trash: lid with handle over a tapered can; ribs at full size (17x18 keyline).
pub fn draw_trash(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let can = [(5.5, 6.0), (18.5, 6.0), (17.0, 20.5), (7.0, 20.5)];
    c.fill(&can, c.body());
    // The lid line doubles as the can's top edge.
    c.path(&[(5.5, 6.0), (7.0, 20.5), (17.0, 20.5), (18.5, 6.0)], s);
    c.line((3.5, 6.0), (20.5, 6.0), s);
    c.path(&[(9.0, 6.0), (9.0, 3.5), (15.0, 3.5), (15.0, 6.0)], s);
    if !c.compact() {
        let rib = c.detail(Shade::Mid);
        c.line((10.0, 9.0), (10.0, 17.5), rib);
        c.line((14.0, 9.0), (14.0, 17.5), rib);
    }
}

/// Clipboard: board with a solid clip over its top edge and text lines (15x18 keyline).
pub fn draw_clipboard(c: &IconCanvas) {
    c.rrect(
        (4.5, 5.0),
        (19.5, 21.0),
        1.5,
        c.body(),
        c.stroke(Weight::Base),
    );
    c.rrect_fill((8.5, 3.0), (15.5, 7.0), 1.5, c.color);
    text_lines(
        c,
        8.0,
        [(11.0, 16.0), (14.0, 16.0), (17.0, 13.0)],
        [(12.0, 16.0), (16.0, 13.0)],
    );
}
