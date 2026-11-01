//! What one frame of a surface costs by its number of cells, timed by hand:
//! `cargo test --release --lib surface_frame_timed -- --ignored --nocapture`.
//! Packed composite colors (no blend) in a 1920 x 1080 frame, smooth and
//! cube modes, from TESSERA's coarsest RGB level (25 M cells) down.

use std::sync::Arc;
use std::time::Instant;

use eframe::egui_wgpu::CallbackResources;

use super::channel_blend_bench_tests::{RUNS, device, median};
use super::surface_blend_bench_tests::{draw_surface, surface_callback};
use crate::plots::Mesh3DRenderer;

/// Grids of 25 M, 6 M, 2 M and 0.5 M cells at TESSERA's 2:1 aspect.
const SIZES: [(usize, usize); 4] = [(7031, 3515), (3500, 1750), (2000, 1000), (1000, 500)];

#[test]
#[ignore = "benchmark"]
fn surface_frame_timed() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED surface_frame_timed: no GPU device");
        return;
    };
    crate::plots::limits::set_from_device(&device.limits());
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut resources = CallbackResources::default();
    eprintln!("surface frame by cells (median of {RUNS} runs)");
    for (w, h) in SIZES {
        let values: Vec<f32> = (0..w * h)
            .map(|i| (i.wrapping_mul(2_654_435_761) % 0xFF_FFFF) as f32)
            .collect();
        let surface = Arc::new(Mesh3DRenderer::new_surface(&device, format, &values, w, h));
        for (name, mode) in [("smooth", 0), ("cubes", 2)] {
            let callback = surface_callback(&surface, mode, None);
            draw_surface(&device, &queue, &mut resources, &callback);
            let times = (0..RUNS)
                .map(|_| {
                    let start = Instant::now();
                    draw_surface(&device, &queue, &mut resources, &callback);
                    start.elapsed()
                })
                .collect();
            let cells = (w * h) as f64 / 1e6;
            eprintln!(
                "{w:>5} x {h:<5} {cells:>5.1} M cells {name:<6} {:>10.2?}",
                median(times)
            );
        }
    }
}
