//! Contact-sheet renderer for visual icon review.
//!
//! `cargo test --lib icon_contact_sheet -- --ignored` renders every icon via
//! `ui::test_render` into `target/icon_sheets/<category>.png`. Columns:
//! 12/14/18/24 at 1x and 18 at 2x, dark theme then light theme; each tile is
//! upscaled so pixels stay visible.

use super::Icon;
use crate::ui::test_render::{rasterize, sheet_dir, tessellate_pass, themes, to_image};
use egui::{LayerId, Rect, Visuals, pos2, vec2};
use image::RgbaImage;
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

/// Paint `icons` stacked in one column of tiles and tessellate them with
/// egui's real tessellator. Returns the meshes and the font atlas.
pub(super) fn tessellate(
    icons: &[Icon],
    size: f32,
    ppp: f32,
    visuals: &Visuals,
) -> (Vec<egui::ClippedPrimitive>, Arc<egui::ColorImage>) {
    let screen = vec2(TILE_PT, TILE_PT * icons.len() as f32);
    tessellate_pass(screen, ppp, visuals, |ctx| {
        let painter = ctx.layer_painter(LayerId::background());
        for (i, icon) in icons.iter().enumerate() {
            let rect = tile_rect(i, size);
            icon.paint(&painter, rect, visuals.text_color(), visuals.dark_mode);
        }
    })
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
    let rows = tile_px * icons.len();
    let buf = rasterize(&prims, &atlas, ppp, tile_px, rows, visuals.panel_fill);
    let scale = TILE_OUT as usize / tile_px;
    (0..icons.len())
        .map(|i| to_image(&buf, tile_px, (0, i * tile_px), (TILE_OUT, TILE_OUT), scale))
        .collect()
}

#[test]
#[ignore = "writes review images to target/icon_sheets"]
fn icon_contact_sheet() {
    let dir = sheet_dir();
    for &category in Icon::CATEGORIES {
        let icons: Vec<Icon> = Icon::ALL
            .iter()
            .copied()
            .filter(|i| i.category() == category)
            .collect();
        let n_cols = COLUMNS.len() as u32 * 2;
        let mut sheet = RgbaImage::new(TILE_OUT * n_cols, TILE_OUT * icons.len() as u32);
        for (t, visuals) in themes().iter().enumerate() {
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
