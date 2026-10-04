//! Review sheet: `cargo test --lib volume_contact_sheet -- --ignored` writes
//! `target/icon_sheets/volume.png`: a 48³ gyroid with a missing corner octant
//! in every mode (0-7), then opaque DVR and close-ups of opaque DVR, MIP,
//! absorption, lit and unlit DVR and the label surface, then DVR at Density
//! 1, 6 and 10 (the default is 3), over mid-gray.

use super::gpu_render::{gpu, params, render_sized, renderer_n};
use crate::ui::test_render::sheet_dir;
use crate::utils::colormap::registry;

const N: usize = 48;
const TILE: u32 = 384;
const COLUMNS: u32 = 4;

fn gyroid() -> Vec<f32> {
    let mut values = Vec::with_capacity(N * N * N);
    for z in 0..N {
        for y in 0..N {
            for x in 0..N {
                let [px, py, pz] = [x, y, z].map(|c| (c as f32 + 0.5) / N as f32 * 9.0);
                let g = px.sin() * py.cos() + py.sin() * pz.cos() + pz.sin() * px.cos();
                let missing = x < N / 3 && y < N / 3 && z >= 2 * N / 3;
                values.push(if missing {
                    f32::NAN
                } else {
                    (g / 3.0 + 0.5).clamp(0.0, 1.0)
                });
            }
        }
    }
    values
}

#[test]
#[ignore = "writes target/icon_sheets/volume.png for visual review"]
fn volume_contact_sheet() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED volume_contact_sheet: no GPU adapter");
        return;
    };
    let renderer = renderer_n(&device, &queue, &gyroid(), N, filterable);
    let opaque = super::VolumeUniformParams {
        transparency: false,
        color: crate::plots::common::PlotColorParams {
            cmin: 0.7,
            ..params(0, 1.0, 0).color
        },
        ..params(0, 1.0, 0)
    };
    let unlit = super::VolumeUniformParams {
        lighting: false,
        ..params(0, 1.0, 0)
    };
    let close = |p: super::VolumeUniformParams| super::VolumeUniformParams { zoom: 1.1, ..p };
    let cases: Vec<_> = (0..8u32)
        .map(|algorithm| params(algorithm, 1.0, 0))
        .chain([
            opaque,
            close(opaque),
            close(params(1, 1.0, 0)),
            close(params(5, 1.0, 0)),
        ])
        .chain([
            close(params(0, 1.0, 0)),
            close(unlit),
            close(params(4, 1.0, 0)),
        ])
        .chain(
            [1.0, 6.0, 10.0].map(|opacity_scale| super::VolumeUniformParams {
                opacity_scale,
                ..params(0, 1.0, 0)
            }),
        )
        .collect();
    let rows = (cases.len() as u32).div_ceil(COLUMNS);
    let mut sheet =
        image::RgbaImage::from_pixel(TILE * COLUMNS, TILE * rows, image::Rgba([96, 96, 96, 255]));
    for (i, p) in cases.iter().enumerate() {
        let pixels = render_sized(&device, &queue, &renderer, p, TILE);
        let (ox, oy) = ((i as u32 % COLUMNS) * TILE, (i as u32 / COLUMNS) * TILE);
        for (j, px) in pixels.as_chunks::<4>().0.iter().enumerate() {
            let (x, y) = (ox + j as u32 % TILE, oy + j as u32 / TILE);
            let bg = sheet.get_pixel(x, y).0;
            // Premultiplied "over" onto the background.
            let over = |c: usize| {
                px[c].saturating_add(((u32::from(bg[c]) * u32::from(255 - px[3])) / 255) as u8)
            };
            sheet.put_pixel(x, y, image::Rgba([over(0), over(1), over(2), 255]));
        }
    }
    sheet
        .save(sheet_dir().join("volume.png"))
        .expect("save sheet");
}
