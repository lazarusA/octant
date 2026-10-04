//! GPU tests of empty-space skipping: skipping must not change a single
//! pixel, only the work. `volume_skip_benchmark` (ignored) times it.

use super::gpu_render::{gpu, params, render_sized, renderer_n};
use super::{VolumeRenderer, VolumeUniformParams};
use crate::utils::colormap::registry;
use std::time::Instant;

/// `n`³ volume of zeros with two small blobs and a missing corner: mostly
/// empty space.
fn sparse(n: usize) -> Vec<f32> {
    let mut values = Vec::with_capacity(n * n * n);
    let centers = [[0.3, 0.35, 0.4], [0.7, 0.6, 0.65]];
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let p = [x, y, z].map(|c| (c as f32 + 0.5) / n as f32);
                let v = centers
                    .iter()
                    .map(|c| {
                        let d2: f32 = p.iter().zip(c).map(|(a, b)| (a - b) * (a - b)).sum();
                        (1.0 - d2.sqrt() / 0.15).clamp(0.0, 1.0)
                    })
                    .fold(0.0, f32::max);
                let missing = x < n / 8 && y < n / 8 && z < n / 8;
                values.push(if missing { f32::NAN } else { v });
            }
        }
    }
    values
}

fn cases() -> Vec<VolumeUniformParams> {
    let opaque = |algorithm| VolumeUniformParams {
        transparency: false,
        ..params(algorithm, 1.0, 0)
    };
    let mut cases: Vec<_> = (0..8).map(|a| params(a, 1.0, 0)).collect();
    cases.extend([opaque(0), opaque(5)]);
    // A ring-buffer shift moves bricks across the seam.
    cases.push(params(0, 1.0, 7));
    cases
}

fn render_all(device: &wgpu::Device, queue: &wgpu::Queue, r: &VolumeRenderer) -> Vec<Vec<u8>> {
    cases()
        .iter()
        .map(|p| render_sized(device, queue, r, p, 128))
        .collect()
}

#[test]
fn gpu_skipping_leaves_pixels_unchanged() {
    let _registry = registry::test_lock();
    let Some((device, queue, _)) = gpu() else {
        eprintln!("SKIPPED gpu_skipping_leaves_pixels_unchanged: no GPU adapter");
        return;
    };
    let n = 40;
    let renderer = renderer_n(&device, &queue, &sparse(n), n, false);
    let skipped = render_all(&device, &queue, &renderer);
    renderer.textures.disable_skipping(&queue);
    let full = render_all(&device, &queue, &renderer);
    for (i, (a, b)) in skipped.iter().zip(&full).enumerate() {
        let diff = a.iter().zip(b).filter(|(x, y)| x != y).count();
        assert_eq!(diff, 0, "case {i}: {diff} bytes differ with skipping");
    }
    assert!(
        skipped.iter().any(|img| img.iter().any(|&v| v > 0)),
        "something renders"
    );
}

/// Milliseconds per frame over `frames` renders submitted back to back (no
/// readback), each with a new camera so the frame cache cannot reuse it.
fn time_frames(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    r: &VolumeRenderer,
    algorithm: u32,
) -> f64 {
    let frames = 16;
    let size = [1024, 1024];
    let render = |rot_y: f32| {
        let p = VolumeUniformParams {
            rot_y,
            ..params(algorithm, 1.0, 0)
        };
        let mut encoder = device.create_command_encoder(&Default::default());
        r.render_frame(device, queue, &mut encoder, &p, size);
        queue.submit([encoder.finish()]);
    };
    // Warm up: builds the mode's pipeline and frame outside the timing.
    render(0.3);
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let start = Instant::now();
    for f in 0..frames {
        render(0.4 + f as f32 * 0.01);
    }
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    start.elapsed().as_secs_f64() * 1000.0 / f64::from(frames)
}

#[test]
#[ignore = "timing benchmark: cargo test --release --lib volume_skip_benchmark -- --ignored --nocapture"]
fn volume_skip_benchmark() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED volume_skip_benchmark: no GPU adapter");
        return;
    };
    let n = 256;
    let renderer = renderer_n(&device, &queue, &sparse(n), n, filterable);
    let names = [
        "DVR",
        "MIP",
        "MinIP",
        "Average",
        "Label",
        "Absorption",
        "Additive",
        "Indexed",
    ];
    let skipped: Vec<f64> = (0..8)
        .map(|a| time_frames(&device, &queue, &renderer, a))
        .collect();
    renderer.textures.disable_skipping(&queue);
    for (a, name) in names.iter().enumerate() {
        let full = time_frames(&device, &queue, &renderer, a as u32);
        eprintln!(
            "{name:>10}: {full:7.2} ms -> {:7.2} ms with skipping ({:.1}x)",
            skipped[a],
            full / skipped[a]
        );
    }
}
