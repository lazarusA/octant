//! GPU test of a composite drawn from bricks: a packed RGB field cut into 8 bricks of an `Rgba8Unorm` atlas
//! matches the same field drawn dense, in DVR, MIP and label surfaces.
//! Skipped (with a note) without an adapter.

use super::bricks::{BrickLevelGpu, BrickUniforms, MAX_LEVELS};
use super::encode::packed_rgba;
use super::gpu_render::{gpu, params};
use super::gpu_tests::render;
use super::{VolumeEncoding, VolumeRenderer, VolumeUniformParams};
use crate::data::multiscale::bricks::{APRON, BRICK, STORED};
use crate::utils::colormap::registry;

const N: usize = 128;

/// A packed color rising red along x, green along z, bright inside a ball.
fn color(x: usize, y: usize, z: usize) -> f32 {
    let f = |i: usize| (i as f32 + 0.5) / N as f32 - 0.5;
    let inside = f(x).powi(2) + f(y).powi(2) + f(z).powi(2) < 0.16;
    let r = (255.0 * (f(x) + 0.5)) as u32;
    let g = (255.0 * (f(z) + 0.5)) as u32;
    let b = if inside { 200 } else { 0 };
    (r | g << 8 | b << 16) as f32
}

fn dense(device: &wgpu::Device, queue: &wgpu::Queue, filterable: bool) -> VolumeRenderer {
    let mut data = Vec::with_capacity(N * N * N);
    for z in 0..N {
        for y in 0..N {
            data.extend((0..N).map(|x| color(x, y, z)));
        }
    }
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let geometry = (&data[..], N as u32, N as u32);
    VolumeRenderer::build(
        device,
        queue,
        format,
        geometry,
        VolumeEncoding::PackedRgb,
        filterable,
    )
    .expect("fits")
}

fn bricked(device: &wgpu::Device, queue: &wgpu::Queue) -> VolumeRenderer {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let renderer = VolumeRenderer::new_bricked(
        device,
        format,
        ([2, 2, 2], [2, 2, 2], super::bricks::AtlasTexel::Color),
    )
    .expect("fits");
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
                    cube.push(packed_rgba(color(cell(0, sx), cell(1, sy), cell(2, sz))));
                }
            }
        }
        bricks.upload_brick(queue, b, &cube);
        bricks.write_range(queue, b, [f32::MIN, f32::MAX], false);
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
    let span = [N as f32, N as f32, N as f32, 0.0];
    bricks.set_uniforms(
        queue,
        &BrickUniforms {
            lo: [0.0; 4],
            span,
            flip: [0; 4],
            grid: [2, 2, 2, 0],
            value: [0.0, 1.0, 0.0, 0.0],
            slots: [2, 2, 2, 0],
            levels,
        },
    );
    bricks.set_dims([N as u32; 3]);
    renderer
}

#[test]
fn gpu_a_bricked_composite_matches_the_dense_one() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED gpu_a_bricked_composite_matches_the_dense_one: no GPU adapter");
        return;
    };
    let (dense, bricked) = (dense(&device, &queue, filterable), bricked(&device, &queue));
    for algorithm in [0, 1, 4] {
        let p = VolumeUniformParams {
            lighting: false,
            ..params(algorithm, 1.0, 0)
        };
        let (a, b) = (
            render(&device, &queue, &dense, &p),
            render(&device, &queue, &bricked, &p),
        );
        let lit: u64 = a.chunks(4).map(|px| u64::from(px[3])).sum();
        let diff: u64 = a
            .iter()
            .zip(&b)
            .map(|(x, y)| u64::from(x.abs_diff(*y)))
            .sum();
        let mean = diff as f32 / a.len() as f32;
        eprintln!("composite mode {algorithm}: bricked differs by {mean}");
        assert!(lit > 0, "mode {algorithm}: the dense composite draws");
        assert!(
            mean < 1.0,
            "mode {algorithm}: bricked composite differs by {mean}"
        );
    }
}
