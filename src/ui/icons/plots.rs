//! Plot type icons (line, surface, globe, volume, point cloud).
//! The heatmap and colormap live in `palette.rs`.

use super::canvas::{IconCanvas, Shade, Weight, key};
use std::f32::consts::PI;

/// PlotLine: joined L axes and a polyline with diamond markers (18-unit square).
pub fn draw_plot_line(c: &IconCanvas) {
    let axes = c.stroke_toned(Weight::Base, c.shade(Shade::Mid));
    c.path(
        &[(4.0, key::SQ_MIN), (4.0, 20.0), (key::SQ_MAX, 20.0)],
        axes,
    );

    let pts = [(6.0, 16.5), (10.0, 9.0), (14.0, 13.0), (19.0, 5.5)];
    c.path(&pts, c.stroke(Weight::Base));
    if !c.compact() {
        for &m in &[pts[1], pts[3]] {
            c.diamond(m, 1.8, 1.8, c.color, egui::Stroke::NONE);
        }
    }
}

/// Intersection of segment `a-b` with segment `c-d`, as grid coordinates.
fn intersect(a: (f32, f32), b: (f32, f32), c: (f32, f32), d: (f32, f32)) -> (f32, f32) {
    let (r, s) = ((b.0 - a.0, b.1 - a.1), (d.0 - c.0, d.1 - c.1));
    let denom = r.0 * s.1 - r.1 * s.0;
    if denom.abs() < f32::EPSILON {
        return b;
    }
    let t = ((c.0 - a.0) * s.1 - (c.1 - a.1) * s.0) / denom;
    (a.0 + t * r.0, a.1 + t * r.1)
}

/// PlotSurface: two overlapping ridges sharing one silhouette outline (18-unit square).
pub fn draw_plot_surface(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let base = 20.0;
    let (back_l, back_peak, back_r) = ((key::SQ_MIN, base), (11.0, 4.0), (key::SQ_MAX, base));
    let (front_l, front_peak) = ((9.5, base), (16.0, 9.0));
    let cross = intersect(back_peak, back_r, front_l, front_peak);

    c.fill(&[back_l, back_peak, back_r], c.body());
    c.fill(&[front_l, front_peak, back_r], c.shade(Shade::Soft));
    c.closed(&[back_l, back_peak, cross, front_peak, back_r], s);
    c.line(front_l, cross, s);
}

/// PlotGlobe: tilted globe on a spin axis that pokes out of the sphere (20-unit circle).
pub fn draw_plot_globe(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let (ctr, r) = ((key::C, key::C), 8.5);
    c.circle(ctr, r, c.body(), s);

    // Axial tilt ~23.5 degrees.
    let (tc, ts) = (0.9171_f32, 0.3987_f32);
    let rot = |x: f32, y: f32| c.p(ctr.0 + x * tc - y * ts, ctr.1 + x * ts + y * tc);

    c.seg(rot(-r, 0.0), rot(r, 0.0), s);
    let meridian: Vec<_> = (0..=16)
        .map(|i| {
            let a = -PI / 2.0 + PI * i as f32 / 16.0;
            rot(a.cos() * r * 0.45, a.sin() * r)
        })
        .collect();
    c.curve(meridian, s);

    let axis = c.stroke_toned(Weight::Base, c.shade(Shade::Strong));
    c.seg(rot(0.0, -11.0), rot(0.0, -r), axis);
    c.seg(rot(0.0, r), rot(0.0, 11.0), axis);
}

/// PlotVolume: isometric voxel cube with shaded faces and one outline (18-unit square).
pub fn draw_plot_volume(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let (top, ur, lr, bot, ll, ul, mid) = (
        (12.0, 3.0),
        (20.0, 7.5),
        (20.0, 16.5),
        (12.0, 21.0),
        (4.0, 16.5),
        (4.0, 7.5),
        (12.0, 12.0),
    );
    c.fill(&[top, ur, mid, ul], c.shade(Shade::Soft));
    c.fill(&[mid, ur, lr, bot], c.shade(Shade::Faint));
    c.fill(&[ul, mid, bot, ll], c.body());
    c.closed(&[top, ur, lr, bot, ll, ul], s);
    c.path(&[ul, mid, ur], s);
    c.line(mid, bot, s);
}

/// PlotPointCloud: scattered points over a faint crosshair; fewer, larger points when compact.
pub fn draw_plot_point_cloud(c: &IconCanvas) {
    if c.compact() {
        let points = [
            (12.0, 12.0, 2.4),
            (6.5, 7.0, 1.9),
            (17.5, 6.5, 1.7),
            (7.0, 17.0, 1.7),
            (17.5, 17.0, 1.9),
        ];
        for (x, y, r) in points {
            c.dot((x, y), r, c.color);
        }
        return;
    }
    let hair = c.detail(Shade::Soft);
    c.line((4.0, key::C), (20.0, key::C), hair);
    c.line((key::C, 4.0), (key::C, 20.0), hair);
    let points = [
        (12.0, 12.0, 2.0),
        (7.5, 7.5, 1.5),
        (17.5, 6.5, 1.2),
        (6.5, 16.5, 1.2),
        (17.0, 16.0, 1.5),
        (11.5, 19.5, 1.1),
        (12.5, 5.0, 1.1),
    ];
    for (x, y, r) in points {
        c.dot((x, y), r, c.color);
    }
}
