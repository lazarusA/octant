//! GPU test of sample weights where a coarse brick meets a finer one: a
//! uniform field drawn half from level-0 bricks and half from one brick 8x
//! coarser (one sample in eight, each standing for eight steps) is as opaque as
//! the same field drawn all from level 0, so no step is lost or counted
//! twice at the border. Skipped (with a note) without an adapter.

use super::bricks::{AtlasTexel, BrickLevelGpu, BrickUniforms, MAX_LEVELS};
use super::gpu_render::{gpu, params};
use super::gpu_tests::render;
use super::{VolumeRenderer, VolumeUniformParams};
use crate::data::multiscale::bricks::STORED;
use crate::plots::camera::OrbitCamera;
use crate::utils::colormap::registry;

const N: usize = 128;
/// Most a pixel's alpha (of 255) may differ between the two.
const PER_PIXEL: u8 = 2;

/// The field 0.5 everywhere in 8 level-0 bricks (slots 0-7) and one level-1
/// brick (slot 8); table cells with x = 1 read level 1 when `mixed`.
fn uniform(device: &wgpu::Device, queue: &wgpu::Queue, mixed: bool) -> VolumeRenderer {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texels = ([3, 3, 1], [2, 2, 2], AtlasTexel::Half);
    let renderer = VolumeRenderer::new_bricked(device, format, texels).expect("fits");
    let bricks = renderer.bricks().expect("bricked");
    let cube = vec![half::f16::from_f32(0.5).to_bits(); (STORED * STORED * STORED) as usize];
    let open = [[f32::MIN, f32::MAX, 0.0, 0.0]; 512];
    let slot = |i: u32| [i % 3, i / 3, 0];
    for i in 0..9 {
        bricks.upload_brick(queue, slot(i), &cube);
        bricks.write_range(queue, slot(i), [0.5, 0.5], false);
        bricks.write_subs(queue, slot(i), &open);
    }
    let mut table = Vec::new();
    for b in (0..8u32).map(|i| [i % 2, i / 2 % 2, i / 4]) {
        let fine = slot(b[0] + 2 * b[1] + 4 * b[2]);
        let entry = match mixed && b[0] == 1 {
            true => [2, 2, 0, 2],
            false => [fine[0] as u8, fine[1] as u8, 0, 1],
        };
        table.push(entry);
    }
    bricks.write_table(queue, &[0..2, 0..2, 0..2], &table);
    let mut levels = [BrickLevelGpu::default(); MAX_LEVELS];
    levels[0] = BrickLevelGpu {
        factor: [1.0; 4],
        offset: [0.0; 4],
        count: [2, 2, 2, 0],
    };
    levels[1] = BrickLevelGpu {
        factor: [8.0; 4],
        offset: [0.0; 4],
        count: [1, 1, 1, 0],
    };
    let uniforms = BrickUniforms {
        lo: [0.0; 4],
        span: [N as f32, N as f32, N as f32, 0.0],
        flip: [0; 4],
        grid: [2, 2, 2, 0],
        value: [0.0, 1.0, 0.0, 0.0],
        slots: [3, 3, 1, 0],
        levels,
    };
    bricks.set_uniforms(queue, &uniforms);
    bricks.set_dims([N as u32; 3]);
    renderer
}

#[test]
fn gpu_samples_weigh_right_where_levels_meet() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_samples_weigh_right_where_levels_meet: no GPU adapter");
        return;
    };
    // Seen from the side, so rays cross the border between the halves.
    let camera = OrbitCamera {
        rotation_y: 1.1,
        distance: 2.0,
        ..OrbitCamera::default()
    };
    let p = VolumeUniformParams {
        lighting: false,
        opacity_scale: 0.05,
        camera: camera.uniform(),
        ..params(0, 1.0, 0)
    };
    let fine = render(&device, &queue, &uniform(&device, &queue, false), &p);
    let mixed = render(&device, &queue, &uniform(&device, &queue, true), &p);
    let alpha = |f: &[u8]| f.chunks(4).map(|px| u64::from(px[3])).sum::<u64>();
    let (a, b) = (alpha(&fine), alpha(&mixed));
    let off = (a as f64 - b as f64).abs() / a.max(1) as f64;
    // Per pixel: a ray losing steps past a coarse brick shows there even
    // when the image's total hides it.
    let per_pixel = fine
        .chunks(4)
        .zip(mixed.chunks(4))
        .map(|(x, y)| x[3].abs_diff(y[3]))
        .max()
        .unwrap_or(0);
    eprintln!("opacity all fine {a}, mixed {b}: off by {off}, at most {per_pixel} per pixel");
    assert!(a > 0, "the field draws");
    // Steps lost or counted twice at the border showed as 0.3% overall and
    // 7 per pixel (stride 8).
    assert!(off < 0.0015, "mixed levels change opacity by {off}");
    assert!(
        per_pixel <= PER_PIXEL,
        "a pixel's opacity changes by {per_pixel}"
    );
}
