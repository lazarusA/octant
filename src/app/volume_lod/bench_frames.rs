//! GPU frame timing and image comparison of the Beechnut benchmark
//! (`bench_tests.rs`).

use std::time::{Duration, Instant};

use super::{PX, RUNS};
use crate::app::OctantApp;
use crate::plots::{PlotType, VolumeRenderer, VolumeUniformParams};

/// Median GPU time of a `PX` frame of `renderer` with `params`, the camera
/// nudged each run so the frame cache renders again.
fn frame_time(
    app: &OctantApp,
    renderer: &VolumeRenderer,
    params: VolumeUniformParams,
) -> Option<Duration> {
    let rs = app.wgpu_render_state.as_ref()?;
    let (device, queue) = (&rs.device, &rs.queue);
    let mut resources = eframe::egui_wgpu::CallbackResources::default();
    crate::plots::colormap_atlas::prepare(device, queue, &mut resources);
    let mut times = Vec::with_capacity(RUNS);
    for run in 0..=RUNS {
        let mut camera = app
            .nav
            .orbit_camera(PlotType::Volume, PX[0] as f32 / PX[1] as f32);
        camera.rotation_y += run as f32 * 1e-4;
        let p = VolumeUniformParams {
            camera: camera.uniform(),
            ..params
        };
        let start = Instant::now();
        let mut encoder = device.create_command_encoder(&Default::default());
        renderer.render_frame(device, queue, &mut encoder, &p, PX, &resources);
        queue.submit([encoder.finish()]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        if run > 0 {
            times.push(start.elapsed());
        }
    }
    times.sort();
    times.get(RUNS / 2).copied()
}

/// The slowest band of a resting frame built in 16 (what one UI frame
/// waits for, `paint_layer` `RESTING_BANDS`).
fn band_time(
    app: &OctantApp,
    renderer: &VolumeRenderer,
    params: VolumeUniformParams,
) -> Option<Duration> {
    let rs = app.wgpu_render_state.as_ref()?;
    let (device, queue) = (&rs.device, &rs.queue);
    let mut resources = eframe::egui_wgpu::CallbackResources::default();
    crate::plots::colormap_atlas::prepare(device, queue, &mut resources);
    let mut camera = app
        .nav
        .orbit_camera(PlotType::Volume, PX[0] as f32 / PX[1] as f32);
    camera.rotation_y += 0.37;
    let p = VolumeUniformParams {
        camera: camera.uniform(),
        ..params
    };
    // A whole frame first: with nothing shown the renderer draws whole.
    let mut encoder = device.create_command_encoder(&Default::default());
    renderer.render_frame_banded(device, queue, &mut encoder, (&p, PX, 1), &resources);
    queue.submit([encoder.finish()]);
    let mut slowest = Duration::ZERO;
    for _ in 0..16 {
        let start = Instant::now();
        let mut encoder = device.create_command_encoder(&Default::default());
        renderer.render_frame_banded(device, queue, &mut encoder, (&p, PX, 16), &resources);
        queue.submit([encoder.finish()]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        slowest = slowest.max(start.elapsed());
    }
    Some(slowest)
}

/// Prints the GPU frame times of the current view.
pub(super) fn report_frames(app: &mut OctantApp, label: &str) {
    let layer = &app.layers.base;
    let params = app.get_volume_uniform_params(layer, PX[0] as f32 / PX[1] as f32);
    let ms = |d: Option<Duration>| d.map_or(f64::NAN, |d| d.as_secs_f64() * 1e3);
    let dense = layer.renderers.volume.clone();
    let bricked = layer.renderers.volume_bricked.clone();
    let (Some(dense), Some(bricked), Some(lod)) =
        (dense, bricked, layer.data.volume_lod.lod.as_ref())
    else {
        eprintln!("{label}: no GPU, frames not timed");
        return;
    };
    let bricks = bricked.bricks().expect("bricked");
    let dense_ms = ms(frame_time(app, &dense, params));
    bricks.set_dims(lod.level_dims(lod.finest));
    let rest_ms = ms(frame_time(app, &bricked, params));
    let band_ms = ms(band_time(app, &bricked, params));
    bricks.set_dims(lod.level_dims(lod.root_level));
    let moving_ms = ms(frame_time(app, &bricked, params));
    for (mode, name) in [(1, "MIP"), (4, "label surfaces")] {
        report_mode(app, (&dense, &bricked), (mode, name), label);
    }
    // Every slot open: no empty skipping.
    let rs = app.wgpu_render_state.as_ref().expect("gpu");
    let open = vec![[f32::MIN, f32::MAX, 0.0, 0.0]; lod.residency.slot_count()];
    rs.queue
        .write_buffer(&bricks.ranges, 0, bytemuck::cast_slice(&open));
    bricks.set_dims(lod.level_dims(lod.finest));
    let unskipped_ms = ms(frame_time(app, &bricked, params));
    eprintln!(
        "{label}: GPU frame {}x{}: dense level {} {dense_ms:.1} ms; bricked at rest {rest_ms:.1} ms \
         (no empty skipping {unskipped_ms:.1} ms, slowest of 16 bands {band_ms:.1} ms); \
         bricked moving {moving_ms:.1} ms",
        PX[0], PX[1], lod.root_level
    );
}

/// Prints the GPU frame times of mode `mode` (walking voxels) over the
/// bricks the DVR view settled on.
fn report_mode(
    app: &OctantApp,
    (dense, bricked): (&VolumeRenderer, &VolumeRenderer),
    (mode, name): (u32, &str),
    label: &str,
) {
    let (Some(lod), Some(bricks)) = (
        app.layers.base.data.volume_lod.lod.as_ref(),
        bricked.bricks(),
    ) else {
        return;
    };
    let params = VolumeUniformParams {
        algorithm: mode,
        ..app.get_volume_uniform_params(&app.layers.base, PX[0] as f32 / PX[1] as f32)
    };
    let ms = |d: Option<Duration>| d.map_or(f64::NAN, |d| d.as_secs_f64() * 1e3);
    let dense_ms = ms(frame_time(app, dense, params));
    bricks.set_dims(lod.level_dims(lod.finest));
    let rest_ms = ms(frame_time(app, bricked, params));
    let band_ms = ms(band_time(app, bricked, params));
    bricks.set_dims(lod.level_dims(lod.root_level));
    let moving_ms = ms(frame_time(app, bricked, params));
    eprintln!(
        "{label}: {name}: dense {dense_ms:.1} ms; bricked at rest {rest_ms:.1} ms \
         (slowest of 16 bands {band_ms:.1} ms); bricked moving {moving_ms:.1} ms"
    );
}

/// Mean byte difference and opaque pixels of the dense and bricked frames
/// of the current view (square, the GPU tests' readback).
pub(super) fn compare_images(app: &OctantApp, label: &str) {
    let Some(rs) = app.wgpu_render_state.as_ref() else {
        return;
    };
    let layer = &app.layers.base;
    let (Some(dense), Some(bricked)) = (&layer.renderers.volume, &layer.renderers.volume_bricked)
    else {
        return;
    };
    let params = app.get_volume_uniform_params(layer, 1.0);
    let render = |r: &VolumeRenderer| {
        crate::plots::volume::gpu_render::render_sized(&rs.device, &rs.queue, r, &params, 512)
    };
    let (a, b) = (render(dense), render(bricked));
    let diff: u64 = a
        .iter()
        .zip(&b)
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum();
    let opaque = |f: &[u8]| f.chunks(4).filter(|p| p[3] > 250).count();
    eprintln!(
        "{label}: dense vs bricked image: mean byte difference {:.2}, opaque pixels {} vs {}",
        diff as f64 / a.len().max(1) as f64,
        opaque(&a),
        opaque(&b)
    );
}

/// CPU time of one visibility pass over the plotted volume.
pub(super) fn time_visibility(app: &OctantApp) {
    let lod = app
        .layers
        .base
        .data
        .volume_lod
        .lod
        .as_ref()
        .expect("bricked");
    let Some(volume) = app.layers.base.data.volume.as_ref() else {
        return;
    };
    let camera = app
        .nav
        .orbit_camera(PlotType::Volume, PX[0] as f32 / PX[1] as f32);
    let c = app.get_color_params(&app.layers.base);
    let opacity = super::super::occlusion::Opacity {
        color: c,
        density: 3.0,
        opaque: false,
    };
    let start = Instant::now();
    let seen = super::super::occlusion::seen_cells(
        volume,
        &camera,
        &lod.place,
        lod.flip,
        lod.bricks().grid(),
        PX,
        opacity,
    );
    let marked = seen.count();
    let cells = crate::data::multiscale::bricks::table_cells(lod.bricks().grid());
    eprintln!(
        "visibility rays: {:.1} ms, {marked} of {cells:?} table cells seen",
        start.elapsed().as_secs_f64() * 1e3,
    );
}
