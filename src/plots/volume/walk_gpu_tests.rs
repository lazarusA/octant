//! GPU tests of the voxel walk (MIP) and empty-space skipping.
//! Skipped (with a note) when no adapter is available.

use super::gpu_render::{gpu, params, renderer_wh};
use super::gpu_tests::{SIZE, max_diff, mean_alpha, render};
use super::{VolumeRenderer, VolumeUniformParams};
use crate::utils::colormap::registry;

#[test]
fn gpu_mip_never_skips_a_voxel() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_mip_never_skips_a_voxel: no GPU adapter, nothing was compared");
        return;
    };
    // Deeper than the 384 samples rays took: a one-voxel plane among 600
    // must show on every ray, as a volume full of its value does.
    const W: usize = 16;
    const D: usize = 600;
    let mut plane = vec![0.0f32; W * W * D];
    plane[W * W * 301..W * W * 302].fill(1.0);
    let full = vec![1.0f32; W * W * D];
    let p = params(1, 1.0, 0);
    let draw = |data: &[f32]| {
        let r = renderer_wh(&device, &queue, data, [W, W], false);
        render(&device, &queue, &r, &p)
    };
    let (plane, full) = (draw(&plane), draw(&full));
    assert!(mean_alpha(&full) > 0.05, "the box must show");
    // Rays through the frame's center cross the box's middle plane; those
    // near its edges may enter after or leave before it.
    let center = |pixels: &[u8]| -> Vec<u8> {
        let rows = (SIZE / 2 - 6)..(SIZE / 2 + 6);
        rows.flat_map(|y| {
            let start = ((y * SIZE + SIZE / 2 - 6) * 4) as usize;
            pixels[start..start + 48].to_vec()
        })
        .collect()
    };
    assert!(
        max_diff(&center(&plane), &center(&full)) <= 2,
        "a ray stepped over the plane"
    );
}

/// Two small balls in a 40^3 volume, with a missing corner.
fn balls() -> Vec<f32> {
    let mut data = vec![0.0f32; M * M * M];
    for (i, v) in data.iter_mut().enumerate() {
        let p = [i % M, (i / M) % M, i / (M * M)].map(|c| c as f32 / M as f32);
        let ball = |c: [f32; 3]| {
            let d = (0..3).map(|k| (p[k] - c[k]).powi(2)).sum::<f32>().sqrt();
            (1.0 - d / 0.15).max(0.0)
        };
        *v = ball([0.3, 0.3, 0.6]).max(ball([0.7, 0.6, 0.3]));
        if p.iter().all(|&c| c < 0.15) {
            *v = f32::NAN;
        }
    }
    data
}

const M: usize = 40;

/// Every mode that skips blocks, ring-shifted or not, plus a value surface
/// and shown NaN colors.
fn skipping_cases() -> Vec<VolumeUniformParams> {
    let mut cases = vec![];
    for (algorithm, transparency) in [(0, true), (0, false), (1, true), (4, false), (4, true)] {
        for shift in [0, 13] {
            let mut p = params(algorithm, 1.0, shift);
            p.transparency = transparency;
            p.color.cmin = 0.2;
            p.color.opacity = 0.6;
            cases.push(p);
        }
    }
    let mut value = params(0, 1.0, 0);
    value.transparency = false;
    value.surface = 1;
    value.isovalue = 0.5;
    cases.push(value);
    let mut nan_shown = params(0, 1.0, 0);
    nan_shown.color.use_nan_color = 1;
    nan_shown.color.nan_color = [1.0, 0.0, 0.0, 0.5];
    cases.push(nan_shown);
    cases
}

#[test]
fn gpu_skipping_blocks_changes_nothing() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_skipping_blocks_changes_nothing: no GPU adapter");
        return;
    };
    let data = balls();
    let skipping: VolumeRenderer = renderer_wh(&device, &queue, &data, [M, M], false);
    let full = renderer_wh(&device, &queue, &data, [M, M], false);
    full.textures.blocks.open_all(&queue);
    for p in skipping_cases() {
        let a = render(&device, &queue, &skipping, &p);
        let b = render(&device, &queue, &full, &p);
        assert!(mean_alpha(&b) > 0.0, "mode {} draws", p.algorithm);
        assert!(
            max_diff(&a, &b) <= 1,
            "mode {} transparency {} shift {}: {}",
            p.algorithm,
            p.transparency,
            p.shift_x,
            max_diff(&a, &b)
        );
    }
}
