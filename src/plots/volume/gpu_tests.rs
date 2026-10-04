//! GPU tests: renders small volumes offscreen and compares the images.
//! Skipped (with a note) when no adapter is available.

use super::gpu_render::{atlas, gpu, params, render_sized, renderer_n};
use super::{VolumeRenderer, VolumeUniformParams};
use crate::utils::colormap::registry;

const SIZE: u32 = 64;
const N: usize = 16;

/// Smooth blob in [0, 1] with the corner octant missing.
fn blob() -> Vec<f32> {
    let mut values = Vec::with_capacity(N * N * N);
    for z in 0..N {
        for y in 0..N {
            for x in 0..N {
                let p = [x, y, z].map(|c| (c as f32 + 0.5) / N as f32 - 0.5);
                let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
                let missing = x < N / 4 && y < N / 4 && z < N / 4;
                values.push(if missing {
                    f32::NAN
                } else {
                    (1.0 - 2.0 * r).clamp(0.0, 1.0)
                });
            }
        }
    }
    values
}

fn render(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &VolumeRenderer,
    p: &VolumeUniformParams,
) -> Vec<u8> {
    render_sized(device, queue, renderer, p, SIZE)
}

fn renderer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[f32],
    hardware: bool,
) -> VolumeRenderer {
    renderer_n(device, queue, data, N, hardware)
}

fn max_diff(a: &[u8], b: &[u8]) -> u8 {
    a.iter()
        .zip(b)
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

fn mean_alpha(pixels: &[u8]) -> f32 {
    let sum: u32 = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| u32::from(p[3]))
        .sum();
    sum as f32 / (SIZE * SIZE) as f32 / 255.0
}

#[test]
fn gpu_volume_rendering() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED gpu_volume_rendering: no GPU adapter, nothing was compared");
        return;
    };
    let data = blob();
    let manual = renderer(&device, &queue, &data, false);
    let dvr = render(&device, &queue, &manual, &params(0, 1.0, 0));
    assert_eq!(dvr.len(), (SIZE * SIZE * 4) as usize, "readback failed");
    assert!(
        mean_alpha(&dvr) > 0.01,
        "the blob must be visible: {}",
        mean_alpha(&dvr)
    );
    // Dithered output stays premultiplied (no channel above alpha).
    assert!(
        dvr.as_chunks::<4>()
            .0
            .iter()
            .all(|p| p[..3].iter().all(|&c| c <= p[3]))
    );

    // Lighting shades color only: opacity is unchanged.
    let unlit_params = VolumeUniformParams {
        lighting: false,
        ..params(0, 1.0, 0)
    };
    let unlit = render(&device, &queue, &manual, &unlit_params);
    assert!(
        dvr.iter()
            .skip(3)
            .step_by(4)
            .eq(unlit.iter().skip(3).step_by(4))
    );
    assert_ne!(dvr, unlit, "lighting must change the shading");

    // Opacity is corrected for the step: quality changes grain, not density.
    let fine = render(&device, &queue, &manual, &params(0, 2.0, 0));
    let coarse = render(&device, &queue, &manual, &params(0, 0.5, 0));
    let (a_fine, a_coarse) = (mean_alpha(&fine), mean_alpha(&coarse));
    assert!(
        (a_fine - a_coarse).abs() < 0.02,
        "alpha {a_fine} vs {a_coarse}"
    );

    // A ring buffer rotated by `shift` renders as the unrotated volume.
    let shift = 5;
    let mut rolled = vec![0.0; data.len()];
    for (i, v) in data.iter().enumerate() {
        let (x, row) = (i % N, i / N);
        rolled[row * N + (x + shift) % N] = *v;
    }
    let rolled_renderer = renderer(&device, &queue, &rolled, false);
    for algorithm in [0, 1] {
        let base = render(&device, &queue, &manual, &params(algorithm, 1.0, 0));
        let ring = render(
            &device,
            &queue,
            &rolled_renderer,
            &params(algorithm, 1.0, shift as u32),
        );
        assert!(
            max_diff(&base, &ring) <= 1,
            "mode {algorithm} differs across the seam"
        );
    }

    // Hardware filtering matches the eight-tap manual filter.
    if filterable {
        let hardware = renderer(&device, &queue, &data, true);
        for algorithm in [0, 1, 3] {
            let a = render(&device, &queue, &manual, &params(algorithm, 1.0, 0));
            let b = render(&device, &queue, &hardware, &params(algorithm, 1.0, 0));
            assert!(
                max_diff(&a, &b) <= 6,
                "mode {algorithm}: hardware vs manual {}",
                max_diff(&a, &b)
            );
        }
    } else {
        eprintln!("SKIPPED hardware filter comparison: no FLOAT32_FILTERABLE");
    }

    // A volume with no data draws nothing.
    let empty = renderer(&device, &queue, &vec![f32::NAN; data.len()], false);
    assert!(empty.textures.has_invalid());
    assert_eq!(
        mean_alpha(&render(&device, &queue, &empty, &params(0, 1.0, 0))),
        0.0
    );
}

#[test]
fn gpu_frame_cache_rerenders_only_on_change() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_frame_cache_rerenders_only_on_change: no GPU adapter");
        return;
    };
    let data = blob();
    let renderer = renderer(&device, &queue, &data, false);
    let resources = atlas(&device, &queue);
    let mut encoder = device.create_command_encoder(&Default::default());
    let mut frame = |p: &VolumeUniformParams, size: [u32; 2]| {
        renderer.render_frame(&device, &queue, &mut encoder, p, size, &resources)
    };
    let base = params(0, 1.0, 0);
    assert!(frame(&base, [32, 32]), "first frame renders");
    assert!(!frame(&base, [32, 32]), "unchanged view reuses the frame");
    let turned = VolumeUniformParams { rot_y: 1.0, ..base };
    assert!(frame(&turned, [32, 32]), "camera change re-renders");
    assert!(frame(&turned, [16, 16]), "size change re-renders");
    renderer.update_data(&queue, &data);
    let mut encoder = device.create_command_encoder(&Default::default());
    assert!(
        renderer.render_frame(&device, &queue, &mut encoder, &turned, [16, 16], &resources),
        "new data re-renders"
    );
}
