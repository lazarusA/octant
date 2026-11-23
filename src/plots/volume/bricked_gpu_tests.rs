//! GPU test of the bricked raymarch: a
//! 128^3 field cut into 8 bricks with their apron, drawn from the atlas
//! through the table, matches the same field drawn dense, also with y
//! stored upside down. Skipped (with a note) without an adapter.

use super::bricks::{BrickLevelGpu, BrickUniforms, MAX_LEVELS};
use super::gpu_render::{gpu, params, renderer_wh};
use super::gpu_tests::render;
use super::{VolumeRenderer, VolumeUniformParams};
use crate::data::multiscale::bricks::{APRON, BRICK, STORED};
use crate::utils::colormap::registry;

pub(super) const N: usize = 128;

/// A smooth field, rising along y so a wrong flip shows.
fn value(x: usize, y: usize, z: usize) -> f32 {
    let f = |i: usize| i as f32 / N as f32;
    0.2 + 0.5 * f(y) + 0.2 * (6.0 * f(x)).sin() * f(z)
}

/// The dense volume as drawn: row 0 at the top, stored rows reversed when
/// `flip_y`.
fn dense(flip_y: bool) -> Vec<f32> {
    dense_of(value, flip_y)
}

/// `dense` of the field `value`.
pub(super) fn dense_of(value: Field, flip_y: bool) -> Vec<f32> {
    let mut out = Vec::with_capacity(N * N * N);
    for z in 0..N {
        for row in 0..N {
            let y = if flip_y { N - 1 - row } else { row };
            out.extend((0..N).map(|x| value(x, y, z)));
        }
    }
    out
}

/// A field over the 128^3 volume's voxels (x, y, z).
pub(super) type Field = fn(usize, usize, usize) -> f32;

/// Brick `b`'s stored cube of the field `value` in storage order, f16 bits.
fn brick(b: [usize; 3], value: Field) -> Vec<u16> {
    let stored = STORED as usize;
    let cell = |i: usize, s: usize| {
        (b[i] * BRICK as usize + s)
            .saturating_sub(APRON as usize)
            .min(N - 1)
    };
    let mut out = Vec::with_capacity(stored * stored * stored);
    for sz in 0..stored {
        for sy in 0..stored {
            for sx in 0..stored {
                let v = value(cell(0, sx), cell(1, sy), cell(2, sz));
                out.push(half::f16::from_f32(v).to_bits());
            }
        }
    }
    out
}

pub(super) fn bricked(device: &wgpu::Device, queue: &wgpu::Queue, flip_y: bool) -> VolumeRenderer {
    bricked_of(device, queue, flip_y, (value, false))
}

/// `bricked` of the field `value`, its bricks flagged as `missing` voxels.
pub(super) fn bricked_of(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    flip_y: bool,
    (value, missing): (Field, bool),
) -> VolumeRenderer {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let renderer = VolumeRenderer::new_bricked(
        device,
        format,
        ([2, 2, 2], [2, 2, 2], super::bricks::AtlasTexel::Half),
    )
    .expect("fits");
    let bricks = renderer.bricks().expect("bricked");
    let mut table = Vec::new();
    for z in 0..2u32 {
        for y in 0..2u32 {
            for x in 0..2u32 {
                let cube = brick([x, y, z].map(|c| c as usize), value);
                bricks.upload_brick(queue, [x, y, z], &cube);
                bricks.write_range(queue, [x, y, z], [0.0, 1.0], missing);
                let gap = f32::from(u8::from(missing));
                bricks.write_subs(queue, [x, y, z], &[[f32::MIN, f32::MAX, gap, 0.0]; 512]);
                table.push([x as u8, y as u8, z as u8, 1]);
            }
        }
    }
    bricks.write_table(queue, &[0..2, 0..2, 0..2], &table);
    let mut levels = [BrickLevelGpu::default(); MAX_LEVELS];
    levels[0] = BrickLevelGpu {
        factor: [1.0; 4],
        offset: [0.0; 4],
        count: [2, 2, 2, 0],
    };
    bricks.set_uniforms(
        queue,
        &BrickUniforms {
            lo: [0.0; 4],
            span: [N as f32, N as f32, N as f32, 0.0],
            flip: [0, u32::from(flip_y), 0, 0],
            grid: [2, 2, 2, 0],
            value: [0.0, 1.0, 0.0, 0.0],
            slots: [2, 2, 2, 0],
            levels,
        },
    );
    bricks.set_dims([N as u32; 3]);
    renderer
}

/// The field at half resolution (64^3): one level-1 brick over the 128^3
/// box, drawn with steps of level 0's voxels (`set_dims`), so rays take one
/// sample in two there and weigh it twice.
fn coarse_only(device: &wgpu::Device, queue: &wgpu::Queue) -> VolumeRenderer {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let renderer = VolumeRenderer::new_bricked(
        device,
        format,
        ([1, 1, 1], [2, 2, 2], super::bricks::AtlasTexel::Half),
    )
    .expect("fits");
    let bricks = renderer.bricks().expect("bricked");
    let h = N / 2;
    let stored = STORED as usize;
    let cell = |s: usize| s.saturating_sub(APRON as usize).min(h - 1);
    let mut cube = Vec::with_capacity(stored * stored * stored);
    for sz in 0..stored {
        for sy in 0..stored {
            for sx in 0..stored {
                let v = value(2 * cell(sx), 2 * cell(sy), 2 * cell(sz));
                cube.push(half::f16::from_f32(v).to_bits());
            }
        }
    }
    bricks.upload_brick(queue, [0, 0, 0], &cube);
    bricks.write_range(queue, [0, 0, 0], [0.0, 1.0], false);
    bricks.write_subs(queue, [0, 0, 0], &[[f32::MIN, f32::MAX, 0.0, 0.0]; 512]);
    bricks.write_table(queue, &[0..2, 0..2, 0..2], &[[0, 0, 0, 2]; 8]);
    let mut levels = [BrickLevelGpu::default(); MAX_LEVELS];
    levels[1] = BrickLevelGpu {
        factor: [2.0; 4],
        offset: [0.0; 4],
        count: [1, 1, 1, 0],
    };
    bricks.set_uniforms(
        queue,
        &BrickUniforms {
            lo: [0.0; 4],
            span: [N as f32, N as f32, N as f32, 0.0],
            flip: [0; 4],
            grid: [2, 2, 2, 0],
            value: [0.0, 1.0, 0.0, 0.0],
            slots: [1, 1, 1, 0],
            levels,
        },
    );
    bricks.set_dims([N as u32; 3]);
    renderer
}

/// The 64^3 field `coarse_only` holds, dense, rows top-down.
pub(super) fn coarse_dense() -> Vec<f32> {
    let h = N / 2;
    let mut out = Vec::with_capacity(h * h * h);
    for z in 0..h {
        for y in 0..h {
            out.extend((0..h).map(|x| value(2 * x, 2 * y, 2 * z)));
        }
    }
    out
}

pub(super) fn mean_diff(a: &[u8], b: &[u8]) -> f32 {
    let total: u64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum();
    total as f32 / a.len().max(1) as f32
}

#[test]
fn gpu_bricked_volume_matches_the_dense_one() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED gpu_bricked_volume_matches_the_dense_one: no GPU adapter");
        return;
    };
    // DVR samples; MIP (1) and label surfaces (4, lit by the faces the
    // walk crosses) walk the voxels.
    for (algorithm, flip_y) in [0, 1, 2, 3, 4, 5, 6, 7]
        .into_iter()
        .flat_map(|a| [(a, false), (a, true)])
    {
        let mut p = VolumeUniformParams {
            lighting: algorithm == 4,
            ..params(algorithm, 1.0, 0)
        };
        if algorithm == 4 {
            // Surfaces inside the box, entered through every face direction.
            p.color.cmin = 0.55;
        }
        let dense = renderer_wh(&device, &queue, &dense(flip_y), [N, N], filterable);
        let expected = render(&device, &queue, &dense, &p);
        let drawn = render(&device, &queue, &bricked(&device, &queue, flip_y), &p);
        let wrong = render(&device, &queue, &bricked(&device, &queue, !flip_y), &p);
        let (close, far) = (mean_diff(&expected, &drawn), mean_diff(&expected, &wrong));
        eprintln!("mode {algorithm} flip_y {flip_y}: bricked {close}, other flip {far}");
        assert!(
            close < 0.5,
            "mode {algorithm} flip_y {flip_y}: differs by {close}"
        );
        // Indexed palettes (7) see one index in this field: no flip shows.
        assert!(
            algorithm == 7 || far > 4.0 * close,
            "mode {algorithm} flip_y {flip_y}: {far} vs {close}"
        );
    }
}

#[test]
fn gpu_missing_bricked_voxels_draw_nothing() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_missing_bricked_voxels_draw_nothing: no GPU adapter");
        return;
    };
    let renderer = bricked(&device, &queue, false);
    let bricks = renderer.bricks().expect("bricked");
    let nan = vec![half::f16::NAN.to_bits(); (STORED * STORED * STORED) as usize];
    for z in 0..2u32 {
        for y in 0..2u32 {
            for x in 0..2u32 {
                bricks.upload_brick(&queue, [x, y, z], &nan);
            }
        }
    }
    let p = VolumeUniformParams {
        lighting: false,
        ..params(0, 1.0, 0)
    };
    let frame = render(&device, &queue, &renderer, &p);
    let alpha: u64 = frame.chunks(4).map(|px| u64::from(px[3])).sum();
    assert_eq!(alpha, 0, "missing voxels are not data");
}

#[test]
fn gpu_coarse_bricks_take_fewer_samples_at_the_same_opacity() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!(
            "SKIPPED gpu_coarse_bricks_take_fewer_samples_at_the_same_opacity: no GPU adapter"
        );
        return;
    };
    // MIP and label surfaces walk the coarse brick's own voxels.
    for algorithm in [0, 1, 2, 3, 4, 5, 6, 7] {
        let p = VolumeUniformParams {
            lighting: false,
            ..params(algorithm, 1.0, 0)
        };
        let dense = renderer_wh(&device, &queue, &coarse_dense(), [N / 2, N / 2], filterable);
        let expected = render(&device, &queue, &dense, &p);
        let drawn = render(&device, &queue, &coarse_only(&device, &queue), &p);
        let close = mean_diff(&expected, &drawn);
        eprintln!("mode {algorithm}: coarse brick on level-0 steps vs dense coarse: {close}");
        assert!(
            close < 1.0,
            "mode {algorithm}: coarse brick differs by {close}"
        );
    }
}
