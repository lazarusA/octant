//! Soft radial glow behind the hero cube.

use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{Color32, Pos2, Shape, Visuals};

/// Segments around the glow; enough that the rings read as circles.
const SEGMENTS: u32 = 48;
/// Middle ring: its distance from the center and its share of the center's alpha,
/// so the falloff eases out instead of fading in a straight cone.
const MID_RADIUS: f32 = 0.45;
const MID_ALPHA: f32 = 0.45;

/// Glow color at its center: a lift toward the text in dark mode, toward
/// the brightest background in light mode, so it always lightens the page.
fn center_color(visuals: &Visuals) -> Color32 {
    if visuals.dark_mode {
        visuals.strong_text_color().gamma_multiply(0.07)
    } else {
        visuals.extreme_bg_color.gamma_multiply(0.9)
    }
}

/// Radial glow of `radius` around `center`, fading to fully transparent at its edge.
pub fn glow_shape(visuals: &Visuals, center: Pos2, radius: f32) -> Shape {
    let inner = center_color(visuals);
    let mid = inner.gamma_multiply(MID_ALPHA);

    let mut mesh = Mesh::default();
    mesh.vertices.push(Vertex {
        pos: center,
        uv: WHITE_UV,
        color: inner,
    });
    for i in 0..SEGMENTS {
        let angle = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let dir = egui::vec2(angle.cos(), angle.sin());
        for (r, color) in [(radius * MID_RADIUS, mid), (radius, Color32::TRANSPARENT)] {
            mesh.vertices.push(Vertex {
                pos: center + dir * r,
                uv: WHITE_UV,
                color,
            });
        }
    }
    // Vertex 0 is the center; ring i has its mid vertex at 1 + 2i, outer at 2 + 2i.
    for i in 0..SEGMENTS {
        let (m0, o0) = (1 + 2 * i, 2 + 2 * i);
        let j = (i + 1) % SEGMENTS;
        let (m1, o1) = (1 + 2 * j, 2 + 2 * j);
        mesh.add_triangle(0, m0, m1);
        mesh.add_triangle(m0, o0, o1);
        mesh.add_triangle(m0, o1, m1);
    }
    Shape::mesh(mesh)
}
