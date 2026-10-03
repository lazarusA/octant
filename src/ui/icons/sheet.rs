//! Contact-sheet renderer for visual icon review.
//!
//! `cargo test --lib icon_contact_sheet -- --ignored` tessellates every icon
//! with egui's real tessellator and rasterizes the meshes on the CPU into
//! `target/icon_sheets/<category>.png`. Columns: 12/14/18/24 at 1x and 18 at 2x,
//! dark theme then light theme; each tile is upscaled so pixels stay visible.

use super::Icon;
use egui::{Color32, Context, LayerId, Pos2, RawInput, Rect, Visuals, pos2, vec2};
use image::{Rgba, RgbaImage};
use std::sync::Arc;

const TILE_PT: f32 = 28.0;
const TILE_OUT: u32 = 112;
pub(super) const COLUMNS: [(f32, f32); 5] = [
    (12.0, 1.0),
    (14.0, 1.0),
    (18.0, 1.0),
    (24.0, 1.0),
    (18.0, 2.0),
];

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
fn rasterize(
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
        for tri in mesh.indices.chunks_exact(3) {
            let v = std::array::from_fn(|k| &mesh.vertices[tri[k] as usize]);
            raster_triangle(&mut out, v, atlas, ppp);
        }
    }
    out.buf
}

/// Paint `icons` stacked in one column of tiles and tessellate them with
/// egui's real tessellator. Returns the meshes and the font atlas.
pub(super) fn tessellate(
    icons: &[Icon],
    size: f32,
    ppp: f32,
    visuals: &Visuals,
) -> (Vec<egui::ClippedPrimitive>, Arc<egui::ColorImage>) {
    let ctx = Context::default();
    ctx.set_pixels_per_point(ppp);
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            vec2(TILE_PT, TILE_PT * icons.len() as f32),
        )),
        ..Default::default()
    };
    ctx.begin_pass(input);
    {
        let painter = ctx.layer_painter(LayerId::background());
        for (i, icon) in icons.iter().enumerate() {
            icon.paint(
                &painter,
                tile_rect(i, size),
                visuals.text_color(),
                visuals.dark_mode,
            );
        }
    }
    let mut out = ctx.end_pass();
    let atlas = out
        .textures_delta
        .set
        .iter()
        .find(|(id, _)| **id == egui::TextureId::default())
        .and_then(|(_, deltas)| deltas.first())
        .map(|delta| match &delta.image {
            egui::ImageData::Color(img) => img.clone(),
        })
        .expect("font atlas delta");
    out.textures_delta.clear();
    (ctx.tessellate(out.shapes, out.pixels_per_point), atlas)
}

/// Icon box of tile `i`, centered in its tile.
pub(super) fn tile_rect(i: usize, size: f32) -> Rect {
    let off = (TILE_PT - size) * 0.5;
    Rect::from_min_size(pos2(off, i as f32 * TILE_PT + off), vec2(size, size))
}

/// Render `icons` at one scale and theme; returns tiles in row-major order.
fn render(icons: &[Icon], size: f32, ppp: f32, visuals: &Visuals) -> Vec<RgbaImage> {
    let (prims, atlas) = tessellate(icons, size, ppp, visuals);
    let tile_px = (TILE_PT * ppp) as usize;
    let buf = rasterize(
        &prims,
        &atlas,
        ppp,
        tile_px,
        tile_px * icons.len(),
        visuals.panel_fill,
    );
    let scale = TILE_OUT as usize / tile_px;
    (0..icons.len())
        .map(|i| {
            RgbaImage::from_fn(TILE_OUT, TILE_OUT, |x, y| {
                let (sx, sy) = (x as usize / scale, i * tile_px + y as usize / scale);
                let c = buf[sy * tile_px + sx];
                Rgba(c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8))
            })
        })
        .collect()
}

#[test]
#[ignore = "writes review images to target/icon_sheets"]
fn icon_contact_sheet() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/icon_sheets");
    std::fs::create_dir_all(&dir).expect("create sheet dir");
    for &category in Icon::CATEGORIES {
        let icons: Vec<Icon> = Icon::ALL
            .iter()
            .copied()
            .filter(|i| i.category() == category)
            .collect();
        let n_cols = COLUMNS.len() as u32 * 2;
        let mut sheet = RgbaImage::new(TILE_OUT * n_cols, TILE_OUT * icons.len() as u32);
        for (t, visuals) in [Visuals::dark(), Visuals::light()].iter().enumerate() {
            for (c, &(size, ppp)) in COLUMNS.iter().enumerate() {
                let col = (t * COLUMNS.len() + c) as u32;
                for (row, tile) in render(&icons, size, ppp, visuals).iter().enumerate() {
                    image::imageops::replace(
                        &mut sheet,
                        tile,
                        i64::from(col * TILE_OUT),
                        i64::from(row as u32 * TILE_OUT),
                    );
                }
            }
        }
        let name = category
            .split_whitespace()
            .next()
            .unwrap_or("icons")
            .to_lowercase();
        sheet
            .save(dir.join(format!("{name}.png")))
            .expect("save sheet");
    }
}
