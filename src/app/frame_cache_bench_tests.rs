//! What the cached frames of meshes and point clouds save, timed by hand:
//! `cargo test --release --lib frame_cache_timed -- --ignored --nocapture`.
//! A repaint whose view changed draws the plot (into its cached frame, then
//! onto the screen); one that changed nothing the plot shows (the hover, a
//! panel, a toast) only draws the frame. Opaque and OIT, 1920 x 1080.

use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui_wgpu::{CallbackResources, CallbackTrait};

use super::channel_blend_bench_tests::{RUNS, device, median};
use super::surface_blend_bench_tests::{draw_surface, orbit, surface_callback};
use crate::plots::oit::Transparency;
use crate::plots::{Mesh3DRenderer, PointCloudCallback, PointCloudRenderer};

/// Median times of `RUNS` repaints that turned the camera, then of `RUNS`
/// that changed nothing, of the callbacks `at(step)` makes.
pub(super) fn time<C: CallbackTrait>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: &mut CallbackResources,
    at: impl Fn(usize) -> C,
) -> (Duration, Duration) {
    draw_surface(device, queue, resources, &at(0));
    let mut timed = |step: usize| {
        let callback = at(step);
        let start = Instant::now();
        draw_surface(device, queue, resources, &callback);
        start.elapsed()
    };
    let turned = (1..=RUNS).map(&mut timed).collect();
    let still = (0..RUNS).map(|_| timed(RUNS)).collect();
    (median(turned), median(still))
}

fn values(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| (i.wrapping_mul(2_654_435_761) % 1000) as f32)
        .collect()
}

fn report(name: &str, transparency: Transparency, (turned, still): (Duration, Duration)) {
    let mode = match transparency {
        Transparency::Oit => "OIT",
        _ => "opaque",
    };
    eprintln!("{name:<26} {mode:<7} view changed {turned:>10.2?}   unchanged {still:>10.2?}");
}

#[test]
#[ignore = "benchmark"]
fn frame_cache_timed() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED frame_cache_timed: no GPU device");
        return;
    };
    crate::plots::limits::set_from_device(&device.limits());
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut resources = CallbackResources::default();
    eprintln!("cached frames, 1920 x 1080 (median of {RUNS} runs)");
    let modes = [Transparency::Off, Transparency::Oit];
    // 1 M cells: a 1920 x 1080 canvas's thinned surface; 24.7 M: none.
    for (w, h) in [(1414, 707), (7031, 3515)] {
        let surface = Arc::new(Mesh3DRenderer::new_surface(
            &device,
            format,
            &values(w * h),
            w,
            h,
        ));
        for transparency in modes {
            let at = |step| {
                let mut callback = surface_callback(&surface, 0, None);
                callback.params.camera = orbit(step);
                callback.params.color.colormap = 0;
                callback.params.color.cmax = 1000.0;
                callback.transparency = transparency;
                callback
            };
            let cells = (w * h) as f64 / 1e6;
            let name = format!("surface {cells:.1} M cells");
            report(
                &name,
                transparency,
                time(&device, &queue, &mut resources, at),
            );
        }
    }
    for (w, h, d) in [(128, 128, 128), (256, 256, 128)] {
        let points = Arc::new(PointCloudRenderer::new(
            &device,
            format,
            &values(w * h * d),
            w as u32,
            h as u32,
        ));
        for transparency in modes {
            let at = |step| PointCloudCallback {
                renderer: points.clone(),
                params: crate::plots::PointCloudUniformParams {
                    slice: crate::plots::slice::SliceUniform::OFF,
                    color: crate::plots::common::PlotColorParams {
                        cmax: 1000.0,
                        ..Default::default()
                    },
                    camera: orbit(step),
                    aspect_x: 1.0,
                    aspect_y: 1.0,
                    aspect_z: 1.0,
                    point_size: crate::app::state::plot_configs::PointCloudPlotConfig::default()
                        .point_size,
                    scale_by_value: false,
                    width: w as u32,
                    height: h as u32,
                    shift_x: 0,
                    shift_y: 0,
                    shift_z: 0,
                },
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1080.0)),
                transparency,
                channels: None,
            };
            let points = (w * h * d) as f64 / 1e6;
            let name = format!("point cloud {points:.1} M points");
            report(
                &name,
                transparency,
                time(&device, &queue, &mut resources, at),
            );
        }
    }
}
