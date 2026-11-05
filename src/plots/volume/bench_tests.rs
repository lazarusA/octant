//! What one raymarched frame costs per mode, timed by hand: `cargo test
//! --release --lib volume_mode_timed -- --ignored --nocapture`. Three
//! 256^3 fields (dense, sparse, categorical filling the box), a 1024^2 frame,
//! the camera turned every run so the frame cache never answers.

use std::time::{Duration, Instant};

use super::VolumeUniformParams;
use super::gpu_render::{atlas, camera, gpu, params, renderer_n};
use crate::utils::colormap::registry;

const N: usize = 256;
const SIZE: u32 = 1024;
const RUNS: usize = 9;

/// Smooth noise filling the box, in [0, 1].
fn dense() -> Vec<f32> {
    field(|p| 0.5 + 0.25 * ((p[0] * 9.0).sin() + (p[1] * 7.0 + p[2] * 5.0).cos()))
}

/// Mostly empty: a few small balls, values in [0, 1].
fn sparse() -> Vec<f32> {
    const BALLS: [[f32; 3]; 4] = [
        [0.3, 0.3, 0.3],
        [0.7, 0.4, 0.6],
        [0.4, 0.7, 0.5],
        [0.6, 0.6, 0.2],
    ];
    field(|p| {
        BALLS
            .iter()
            .map(|c| {
                let d = (0..3).map(|i| (p[i] - c[i]).powi(2)).sum::<f32>().sqrt();
                (1.0 - d / 0.06).max(0.0)
            })
            .fold(0.0, f32::max)
    })
}

/// Classes 1..=4 in slabs and a ball, filling the box (land-cover like).
fn categorical() -> Vec<f32> {
    field(|p| {
        let r = (0..3).map(|i| (p[i] - 0.5).powi(2)).sum::<f32>().sqrt();
        if r < 0.25 {
            4.0
        } else {
            1.0 + (p[1] * 3.0).floor().min(2.0)
        }
    })
}

fn field(f: impl Fn([f32; 3]) -> f32) -> Vec<f32> {
    let mut v = Vec::with_capacity(N * N * N);
    for z in 0..N {
        for y in 0..N {
            for x in 0..N {
                v.push(f([x, y, z].map(|c| (c as f32 + 0.5) / N as f32)));
            }
        }
    }
    v
}

#[test]
#[ignore = "benchmark"]
fn volume_mode_timed() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED volume_mode_timed: no GPU adapter");
        return;
    };
    let resources = atlas(&device, &queue);
    eprintln!("{N}^3 volume, {SIZE}^2 frame, median of {RUNS} turned frames");
    for (name, data, cmax) in [
        ("dense", dense(), 1.0),
        ("sparse", sparse(), 1.0),
        ("categorical", categorical(), 4.0),
    ] {
        let renderer = renderer_n(&device, &queue, &data, N, filterable);
        for (mode, algorithm, transparency) in [
            ("dvr", 0, true),
            ("dvr opaque", 0, false),
            ("mip", 1, true),
            ("label", 4, false),
        ] {
            let mut p = params(algorithm, 1.0, 0);
            p.transparency = transparency;
            p.color.cmin = if name == "sparse" { 0.1 } else { 0.0 };
            p.color.cmax = cmax;
            let t = timed(&device, &queue, &renderer, p, &resources);
            eprintln!("{name:<12} {mode:<11} {t:>10.2?}");
        }
    }
}

fn timed(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &super::VolumeRenderer,
    base: VolumeUniformParams,
    resources: &eframe::egui_wgpu::CallbackResources,
) -> Duration {
    let mut times = Vec::with_capacity(RUNS);
    for run in 0..=RUNS {
        let p = VolumeUniformParams {
            camera: camera(0.6 + run as f32 * 0.37, [0.0; 3]),
            ..base
        };
        let start = Instant::now();
        let mut encoder = device.create_command_encoder(&Default::default());
        renderer.render_frame(device, queue, &mut encoder, &p, [SIZE, SIZE], resources);
        queue.submit([encoder.finish()]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        // The first run builds the pipeline and the frame.
        if run > 0 {
            times.push(start.elapsed());
        }
    }
    times.sort();
    times[RUNS / 2]
}
