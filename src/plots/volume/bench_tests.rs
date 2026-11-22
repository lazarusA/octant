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
        // The same volume with every block open: what skipping saves.
        let open = renderer_n(&device, &queue, &data, N, filterable);
        open.textures.blocks.open_all(&queue);
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
            let unskipped = timed(&device, &queue, &open, p, &resources);
            eprintln!("{name:<12} {mode:<11} {t:>10.2?}  (no skipping {unskipped:>10.2?})");
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

/// `data` (`N`^3) as one level of `N / 64` bricks a side, in slots of its
/// own, drawn bricked.
fn bricked_of(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[f32],
    spread: bool,
) -> super::VolumeRenderer {
    use super::bricks::{BrickLevelGpu, BrickUniforms, MAX_LEVELS};
    use crate::data::multiscale::bricks::{APRON, BRICK, STORED};
    let b = (N as u64 / BRICK) as u32;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let slots = if spread { [8, 8, 6] } else { [b; 3] };
    let renderer = super::VolumeRenderer::new_bricked(
        device,
        queue,
        format,
        (slots, [b; 3], super::bricks::AtlasTexel::Half),
    )
    .expect("fits");
    // Spread: each brick two slots apart in a 384-slot atlas, as the app's.
    let at = |c: u32| if spread { c * 2 } else { c };
    let bricks = renderer.bricks().expect("bricked");
    let stored = STORED as usize;
    let mut table = Vec::new();
    for bz in 0..b {
        for by in 0..b {
            for bx in 0..b {
                let cell = |c: u32, s: usize| {
                    (c as usize * BRICK as usize + s)
                        .saturating_sub(APRON as usize)
                        .min(N - 1)
                };
                let mut cube = Vec::with_capacity(stored * stored * stored);
                for sz in 0..stored {
                    for sy in 0..stored {
                        for sx in 0..stored {
                            let at = (cell(bz, sz) * N + cell(by, sy)) * N + cell(bx, sx);
                            cube.push(half::f16::from_f32(data[at]).to_bits());
                        }
                    }
                }
                let slot = [at(bx), at(by), bz];
                bricks.upload_brick(queue, slot, &cube);
                let (lo, hi) = cube
                    .iter()
                    .map(|&h| half::f16::from_bits(h).to_f32())
                    .fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)));
                bricks.write_range(queue, slot, [lo, hi], false);
                // Spread: neighbours alternate between two identical levels.
                let level = if spread { 1 + (bx + by + bz) % 2 } else { 1 };
                table.push([slot[0] as u8, slot[1] as u8, slot[2] as u8, level as u8]);
            }
        }
    }
    bricks.write_table(queue, &[0..b, 0..b, 0..b], &table);
    let mut levels = [BrickLevelGpu::default(); MAX_LEVELS];
    levels[0] = BrickLevelGpu {
        factor: [1.0; 4],
        offset: [0.0; 4],
        count: [b, b, b, 0],
    };
    levels[1] = levels[0];
    let n = N as f32;
    bricks.set_uniforms(
        queue,
        &BrickUniforms {
            lo: [0.0; 4],
            span: [n, n, n, 0.0],
            flip: [0; 4],
            grid: [b, b, b, 0],
            value: [0.0, 1.0, 0.0, 0.0],
            slots: [slots[0], slots[1], slots[2], 0],
            levels,
        },
    );
    bricks.set_dims([N as u32; 3]);
    renderer
}

#[test]
#[ignore = "benchmark"]
fn volume_bricked_timed() {
    let _registry = registry::test_lock();
    let Some((device, queue, filterable)) = gpu() else {
        eprintln!("SKIPPED volume_bricked_timed: no GPU adapter");
        return;
    };
    let resources = atlas(&device, &queue);
    eprintln!("{N}^3 volume, {SIZE}^2 frame, dense vs one level of bricks, DVR");
    for (name, data) in [("dense", dense()), ("sparse", sparse())] {
        let flat = renderer_n(&device, &queue, &data, N, filterable);
        let bricked = bricked_of(&device, &queue, &data, false);
        let spread = bricked_of(&device, &queue, &data, true);
        let mut p = params(0, 1.0, 0);
        p.color.cmin = if name == "sparse" { 0.1 } else { 0.0 };
        let t = timed(&device, &queue, &flat, p, &resources);
        let b = timed(&device, &queue, &bricked, p, &resources);
        let s = timed(&device, &queue, &spread, p, &resources);
        eprintln!(
            "{name:<8} dense {t:>10.2?}  bricked {b:>10.2?}  in a 384-slot atlas, two levels {s:>10.2?}"
        );
    }
}
