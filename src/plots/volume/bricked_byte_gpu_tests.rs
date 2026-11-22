//! GPU test of a byte atlas (`AtlasTexel::Byte`): 8-bit integer data cut into
//! 8 bricks of an `R8Unorm` atlas, values `(v - 0) / 255`, matches the same
//! data drawn dense, in DVR and MIP. Skipped (with a note) without an
//! adapter.

use super::bricks::{AtlasTexel, BrickLevelGpu, BrickUniforms, MAX_LEVELS};
use super::gpu_render::{gpu, params, renderer_wh};
use super::gpu_tests::render;
use super::{VolumeRenderer, VolumeUniformParams};
use crate::data::multiscale::bricks::{APRON, BRICK, STORED};
use crate::utils::colormap::registry;

const N: usize = 128;

/// Integers 0..=255, rising along y, rippling along x.
fn value(x: usize, y: usize, z: usize) -> u8 {
    let f = |i: usize| i as f32 / N as f32;
    (255.0 * (0.2 + 0.5 * f(y) + 0.2 * (6.0 * f(x)).sin() * f(z))).round() as u8
}

fn bricked(device: &wgpu::Device, queue: &wgpu::Queue) -> VolumeRenderer {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texels = ([2, 2, 2], [2, 2, 2], AtlasTexel::Byte);
    let renderer = VolumeRenderer::new_bricked(device, queue, format, texels).expect("fits");
    let bricks = renderer.bricks().expect("bricked");
    let stored = STORED as usize;
    let mut table = Vec::new();
    for b in (0..8u32).map(|i| [i % 2, i / 2 % 2, i / 4]) {
        let cell = |i: usize, s: usize| {
            (b[i] as usize * BRICK as usize + s)
                .saturating_sub(APRON as usize)
                .min(N - 1)
        };
        let mut cube = Vec::with_capacity(stored * stored * stored);
        for sz in 0..stored {
            for sy in 0..stored {
                for sx in 0..stored {
                    cube.push(value(cell(0, sx), cell(1, sy), cell(2, sz)));
                }
            }
        }
        bricks.upload_brick(queue, b, &cube);
        bricks.write_range(queue, b, [0.0, 255.0], false);
        bricks.write_subs(queue, b, &[[f32::MIN, f32::MAX, 0.0, 0.0]; 512]);
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
        value: [0.0, 255.0, 0.0, 0.0],
        slots: [2, 2, 2, 0],
        levels,
    };
    bricks.set_uniforms(queue, &uniforms);
    bricks.set_dims([N as u32; 3]);
    renderer
}

#[test]
fn gpu_a_byte_atlas_matches_the_dense_volume() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED gpu_a_byte_atlas_matches_the_dense_volume: no GPU adapter");
        return;
    };
    let mut data = Vec::with_capacity(N * N * N);
    for z in 0..N {
        for y in 0..N {
            data.extend((0..N).map(|x| f32::from(value(x, y, z))));
        }
    }
    let dense = renderer_wh(&device, &queue, &data, [N, N], filterable);
    let bricked = bricked(&device, &queue);
    for algorithm in [0, 1, 4] {
        let mut p = VolumeUniformParams {
            lighting: false,
            ..params(algorithm, 1.0, 0)
        };
        (p.color.cmin, p.color.cmax) = (0.0, 255.0);
        let (a, b) = (
            render(&device, &queue, &dense, &p),
            render(&device, &queue, &bricked, &p),
        );
        let diff: u64 = a
            .iter()
            .zip(&b)
            .map(|(x, y)| u64::from(x.abs_diff(*y)))
            .sum();
        let mean = diff as f32 / a.len() as f32;
        eprintln!("byte atlas mode {algorithm}: differs by {mean}");
        assert!(mean < 0.5, "mode {algorithm}: byte atlas differs by {mean}");
    }
}
