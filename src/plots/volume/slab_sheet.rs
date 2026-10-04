//! Review sheet: `cargo test --lib volume_slab_contact_sheet -- --ignored`
//! writes `target/icon_sheets/volume_slab.png`: a thin 96×64×12 slab with a
//! smooth, mid-range field (a climate-like layer stack) over the dark theme's
//! background, as the app shows it: default camera, oblique, top-down, MIP,
//! unlit DVR and DVR at Density 20.

use super::VolumeUniformParams;
use super::gpu_render::{gpu, params, render_sized, renderer_wh};
use crate::ui::test_render::sheet_dir;
use crate::utils::colormap::registry;

const W: usize = 96;
const H: usize = 64;
const D: usize = 12;
const TILE: u32 = 384;
const COLUMNS: u32 = 3;
const BACKGROUND: [u8; 4] = [27, 27, 27, 255];

/// Smooth field mostly in the middle of [0, 1]: waves that weaken with height.
fn slab() -> Vec<f32> {
    let mut values = Vec::with_capacity(W * H * D);
    for z in 0..D {
        for y in 0..H {
            for x in 0..W {
                let [u, v, w] = [
                    x as f32 / W as f32,
                    y as f32 / H as f32,
                    z as f32 / D as f32,
                ];
                let wave = (u * 9.0).sin() * (v * 7.0).cos() * (1.0 - 0.6 * w);
                values.push((0.5 + 0.3 * wave - 0.15 * w).clamp(0.0, 1.0));
            }
        }
    }
    values
}

fn camera(rot_x: f32, rot_y: f32, algorithm: u32) -> VolumeUniformParams {
    VolumeUniformParams {
        rot_x,
        rot_y,
        aspect_x: 1.0,
        aspect_y: H as f32 / W as f32,
        aspect_z: 0.25,
        ..params(algorithm, 1.0, 0)
    }
}

#[test]
#[ignore = "writes target/icon_sheets/volume_slab.png for visual review"]
fn volume_slab_contact_sheet() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED volume_slab_contact_sheet: no GPU adapter");
        return;
    };
    let renderer = renderer_wh(&device, &queue, &slab(), [W, H], filterable);
    let oblique = camera(0.6, 0.6, 0);
    let cases = [
        camera(0.25, 0.0, 0),
        oblique,
        camera(1.45, 0.0, 0),
        camera(0.6, 0.6, 1),
        VolumeUniformParams {
            lighting: false,
            ..oblique
        },
        VolumeUniformParams {
            opacity_scale: 20.0,
            ..oblique
        },
    ];
    let rows = (cases.len() as u32).div_ceil(COLUMNS);
    let mut sheet =
        image::RgbaImage::from_pixel(TILE * COLUMNS, TILE * rows, image::Rgba(BACKGROUND));
    for (i, p) in cases.iter().enumerate() {
        let pixels = render_sized(&device, &queue, &renderer, p, TILE);
        let (ox, oy) = ((i as u32 % COLUMNS) * TILE, (i as u32 / COLUMNS) * TILE);
        for (j, px) in pixels.as_chunks::<4>().0.iter().enumerate() {
            let (x, y) = (ox + j as u32 % TILE, oy + j as u32 / TILE);
            // Premultiplied "over" onto the background.
            let over = |c: usize| {
                px[c].saturating_add(
                    ((u32::from(BACKGROUND[c]) * u32::from(255 - px[3])) / 255) as u8,
                )
            };
            sheet.put_pixel(x, y, image::Rgba([over(0), over(1), over(2), 255]));
        }
    }
    sheet
        .save(sheet_dir().join("volume_slab.png"))
        .expect("save sheet");
}
