//! GPU test of sub-block skipping (`brick_subs`): a bright ball in a dark
//! volume drawn with each 8^3 sub-block's range draws the same as with
//! every sub-block open, in the sampled modes. Skipped (with a note)
//! without an adapter.

use super::bricks::{AtlasTexel, BrickLevelGpu, BrickUniforms, MAX_LEVELS};
use super::gpu_render::{gpu, params};
use super::gpu_tests::render;
use super::{VolumeRenderer, VolumeUniformParams};
use crate::data::multiscale::bricks::{APRON, BRICK, STORED};
use crate::utils::colormap::registry;

const N: usize = 128;

/// 1 inside a ball off the center, 0.1 elsewhere.
fn value(x: usize, y: usize, z: usize) -> f32 {
    let f = |i: usize, c: f32| (i as f32 + 0.5) / N as f32 - c;
    let r2 = f(x, 0.6).powi(2) + f(y, 0.45).powi(2) + f(z, 0.5).powi(2);
    if r2 < 0.04 { 1.0 } else { 0.1 }
}

/// Brick `b`'s stored cube and its sub-block ranges (cells `8s..8s + 10`).
fn brick(b: [u32; 3]) -> (Vec<u16>, Vec<[f32; 4]>) {
    let stored = STORED as usize;
    let cell = |i: usize, s: usize| {
        (b[i] as usize * BRICK as usize + s)
            .saturating_sub(APRON as usize)
            .min(N - 1)
    };
    let at = |sx, sy, sz| value(cell(0, sx), cell(1, sy), cell(2, sz));
    let mut cube = Vec::with_capacity(stored * stored * stored);
    for sz in 0..stored {
        for sy in 0..stored {
            for sx in 0..stored {
                cube.push(half::f16::from_f32(at(sx, sy, sz)).to_bits());
            }
        }
    }
    let mut subs = Vec::with_capacity(512);
    for k in 0..8 {
        for j in 0..8 {
            for i in 0..8 {
                let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
                for sz in 8 * k..8 * k + 10 {
                    for sy in 8 * j..8 * j + 10 {
                        for sx in 8 * i..8 * i + 10 {
                            let v = at(sx, sy, sz);
                            (lo, hi) = (lo.min(v), hi.max(v));
                        }
                    }
                }
                subs.push([lo, hi, 0.0, 0.0]);
            }
        }
    }
    (cube, subs)
}

/// The ball in 8 bricks; every sub-block open unless `skip`. Also how many
/// sub-blocks draw nothing at `cmin`.
fn bricked(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    skip: bool,
    cmin: f32,
) -> (VolumeRenderer, usize) {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texels = ([2, 2, 2], [2, 2, 2], AtlasTexel::Half);
    let renderer = VolumeRenderer::new_bricked(device, queue, format, texels).expect("fits");
    let bricks = renderer.bricks().expect("bricked");
    let (mut table, mut closed) = (Vec::new(), 0);
    for b in (0..8u32).map(|i| [i % 2, i / 2 % 2, i / 4]) {
        let (cube, subs) = brick(b);
        closed += subs.iter().filter(|s| s[1] < cmin).count();
        bricks.upload_brick(queue, b, &cube);
        bricks.write_range(queue, b, [0.1, 1.0], false);
        let open = vec![[f32::MIN, f32::MAX, 0.0, 0.0]; 512];
        bricks.write_subs(queue, b, if skip { &subs } else { &open });
        table.push([b[0] as u8, b[1] as u8, b[2] as u8, 1]);
    }
    bricks.write_table(queue, &[0..2, 0..2, 0..2], &table);
    let mut levels = [BrickLevelGpu::default(); MAX_LEVELS];
    levels[0] = BrickLevelGpu {
        factor: [1.0; 4],
        offset: [0.0; 4],
        count: [2, 2, 2, 0],
    };
    let uniforms = BrickUniforms {
        lo: [0.0; 4],
        span: [N as f32, N as f32, N as f32, 0.0],
        flip: [0; 4],
        grid: [2, 2, 2, 0],
        value: [0.0, 1.0, 0.0, 0.0],
        slots: [2, 2, 2, 0],
        levels,
    };
    bricks.set_uniforms(queue, &uniforms);
    bricks.set_dims([N as u32; 3]);
    (renderer, closed)
}

#[test]
fn gpu_skipping_sub_blocks_changes_nothing() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_skipping_sub_blocks_changes_nothing: no GPU adapter");
        return;
    };
    let cmin = 0.5;
    let (open, _) = bricked(&device, &queue, false, cmin);
    let (skipping, closed) = bricked(&device, &queue, true, cmin);
    assert!(closed > 1000, "most sub-blocks are dark: {closed}");
    for algorithm in [0, 2, 3] {
        let mut p = VolumeUniformParams {
            lighting: algorithm == 0,
            ..params(algorithm, 1.0, 0)
        };
        p.color.cmin = cmin;
        let (a, b) = (
            render(&device, &queue, &open, &p),
            render(&device, &queue, &skipping, &p),
        );
        let lit: u64 = a.chunks(4).map(|px| u64::from(px[3])).sum();
        let diff: u64 = a
            .iter()
            .zip(&b)
            .map(|(x, y)| u64::from(x.abs_diff(*y)))
            .sum();
        let mean = diff as f32 / a.len() as f32;
        eprintln!("mode {algorithm}: skipping sub-blocks differs by {mean}");
        assert!(lit > 0, "mode {algorithm}: the ball draws");
        assert!(
            mean < 0.05,
            "mode {algorithm}: skipping changed the image by {mean}"
        );
    }
}

#[test]
fn gpu_minip_skips_nothing_above_the_range() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_minip_skips_nothing_above_the_range: no GPU adapter");
        return;
    };
    // Every value lies above cmax: MinIP draws the top color everywhere.
    let mut p = params(2, 1.0, 0);
    (p.color.cmin, p.color.cmax) = (0.0, 0.05);
    // No visible low clip color: MinIP may skip blocks below the range.
    p.color.lowclip_color = [0.0; 4];
    let (open, _) = bricked(&device, &queue, false, p.color.cmin);
    let (skipping, _) = bricked(&device, &queue, true, p.color.cmin);
    let (a, b) = (
        render(&device, &queue, &open, &p),
        render(&device, &queue, &skipping, &p),
    );
    let lit = |f: &[u8]| f.chunks(4).map(|px| u64::from(px[3])).sum::<u64>();
    assert!(lit(&a) > 0, "the volume draws above its range");
    assert_eq!(lit(&a), lit(&b), "nothing above the range is skipped");
}
