//! Test-only CPU renderer for visual review sheets.
//!
//! Runs egui passes, tessellates them with egui's real tessellator and
//! rasterizes the meshes (including font-atlas discs) so icons and brand marks
//! can be reviewed as PNGs without a GPU. Used by `icons::sheet` and
//! `brand::tests`; outputs go to `target/icon_sheets/`.

use egui::{Color32, Context, Pos2, RawInput, Rect, Vec2, Visuals, pos2};
use image::{Rgba, RgbaImage};
use std::path::PathBuf;
use std::sync::Arc;

fn premul(c: Color32) -> [f32; 4] {
    let [r, g, b, a] = c.to_array();
    [r, g, b, a].map(|v| f32::from(v) / 255.0)
}

/// Bilinear sample of the font atlas (egui draws small discs from it).
fn sample(atlas: &egui::ColorImage, u: f32, v: f32) -> [f32; 4] {
    let [w, h] = atlas.size;
    let (x, y) = (u * w as f32 - 0.5, v * h as f32 - 0.5);
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let at = |dx: f32, dy: f32| {
        let xi = ((x0 + dx).max(0.0) as usize).min(w - 1);
        let yi = ((y0 + dy).max(0.0) as usize).min(h - 1);
        premul(atlas.pixels[yi * w + xi])
    };
    let (a, b, c, d) = (at(0.0, 0.0), at(1.0, 0.0), at(0.0, 1.0), at(1.0, 1.0));
    std::array::from_fn(|k| {
        (a[k] * (1.0 - fx) + b[k] * fx) * (1.0 - fy) + (c[k] * (1.0 - fx) + d[k] * fx) * fy
    })
}

/// Premultiplied RGBA float image the rasterizer blends into.
struct Canvas {
    buf: Vec<[f32; 4]>,
    w: usize,
    h: usize,
}

/// Blend one tessellated triangle into `out` with barycentric interpolation.
fn raster_triangle(
    out: &mut Canvas,
    v: [&egui::epaint::Vertex; 3],
    atlas: &egui::ColorImage,
    ppp: f32,
) {
    let p: [Pos2; 3] = v.map(|v| pos2(v.pos.x * ppp, v.pos.y * ppp));
    let area = (p[1].x - p[0].x) * (p[2].y - p[0].y) - (p[2].x - p[0].x) * (p[1].y - p[0].y);
    if area.abs() < 1e-6 {
        return;
    }
    let lo =
        |f: fn(&Pos2) -> f32| p.iter().map(f).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let hi =
        |f: fn(&Pos2) -> f32| p.iter().map(f).fold(f32::MIN, f32::max).ceil().max(0.0) as usize;
    let (x0, x1) = (lo(|q| q.x), hi(|q| q.x).min(out.w));
    let (y0, y1) = (lo(|q| q.y), hi(|q| q.y).min(out.h));
    let cols = v.map(|v| premul(v.color));
    for y in y0..y1 {
        for x in x0..x1 {
            // Nudge off exact pixel centers so a pixel on a shared triangle edge is
            // counted once, like the GPU's top-left fill rule.
            let (px, py) = (x as f32 + 0.5 + 1e-3, y as f32 + 0.5 + 2e-3);
            let e = |a: Pos2, b: Pos2| (b.x - a.x) * (py - a.y) - (px - a.x) * (b.y - a.y);
            let wts = [
                e(p[1], p[2]) / area,
                e(p[2], p[0]) / area,
                e(p[0], p[1]) / area,
            ];
            if wts.iter().any(|&w| w < 0.0) {
                continue;
            }
            let lerp = |f: &dyn Fn(usize) -> f32| wts[0] * f(0) + wts[1] * f(1) + wts[2] * f(2);
            let tex = sample(atlas, lerp(&|i| v[i].uv.x), lerp(&|i| v[i].uv.y));
            let dst = &mut out.buf[y * out.w + x];
            let src: [f32; 4] = std::array::from_fn(|k| lerp(&|i| cols[i][k]) * tex[k]);
            for k in 0..4 {
                dst[k] = src[k] + dst[k] * (1.0 - src[3]);
            }
        }
    }
}

/// Rasterize tessellated meshes over a solid background.
pub fn rasterize(
    prims: &[egui::ClippedPrimitive],
    atlas: &egui::ColorImage,
    ppp: f32,
    w: usize,
    h: usize,
    bg: Color32,
) -> Vec<[f32; 4]> {
    let mut out = Canvas {
        buf: vec![premul(bg); w * h],
        w,
        h,
    };
    for prim in prims {
        let egui::epaint::Primitive::Mesh(mesh) = &prim.primitive else {
            continue;
        };
        let (triangles, _) = mesh.indices.as_chunks::<3>();
        for tri in triangles {
            let v = tri.map(|i| &mesh.vertices[i as usize]);
            raster_triangle(&mut out, v, atlas, ppp);
        }
    }
    out.buf
}

/// Run one egui pass with `draw`, then tessellate it. Returns the meshes and
/// the font atlas.
pub fn tessellate_pass(
    screen: Vec2,
    ppp: f32,
    visuals: &Visuals,
    draw: impl Fn(&Context),
) -> (Vec<egui::ClippedPrimitive>, Arc<egui::ColorImage>) {
    let ctx = Context::default();
    ctx.set_pixels_per_point(ppp);
    ctx.set_visuals(visuals.clone());
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen)),
        ..Default::default()
    };
    // Two passes: areas are invisible on the frame they first appear. The
    // font atlas only ships with the first pass, the shapes come from the second.
    ctx.begin_pass(input.clone());
    draw(&ctx);
    let mut first = ctx.end_pass();
    let atlas = first
        .textures_delta
        .set
        .iter()
        .find(|(id, _)| **id == egui::TextureId::default())
        .and_then(|(_, deltas)| deltas.first())
        .map(|delta| match &delta.image {
            egui::ImageData::Color(img) => img.clone(),
        })
        .expect("font atlas delta");
    first.textures_delta.clear();
    ctx.begin_pass(input);
    draw(&ctx);
    let mut out = ctx.end_pass();
    out.textures_delta.clear();
    (ctx.tessellate(out.shapes, out.pixels_per_point), atlas)
}

/// Nearest-neighbor upscale of a rasterized buffer region into an image.
pub fn to_image(
    buf: &[[f32; 4]],
    stride: usize,
    origin: (usize, usize),
    size: (u32, u32),
    scale: usize,
) -> RgbaImage {
    RgbaImage::from_fn(size.0, size.1, |x, y| {
        let (sx, sy) = (origin.0 + x as usize / scale, origin.1 + y as usize / scale);
        let c = buf[sy * stride + sx];
        Rgba(c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8))
    })
}

/// Output directory for review sheets, created on demand.
pub fn sheet_dir() -> PathBuf {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/icon_sheets");
    std::fs::create_dir_all(&dir).expect("create sheet dir");
    dir
}

/// Default dark and light visuals, in sheet column order.
pub fn themes() -> [Visuals; 2] {
    [Visuals::dark(), Visuals::light()]
}
