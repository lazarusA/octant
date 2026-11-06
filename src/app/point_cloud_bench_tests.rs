//! What a point cloud costs, timed by hand:
//! `cargo test --release --lib point_cloud_timed -- --ignored --nocapture`.
//! A frame while orbiting (the camera turned each run, so never a cached
//! frame) and a repaint that changed nothing, opaque in 1920 x 1080, by
//! number of points up to a 45 x 720 x 1440 stack; then what showing that
//! stack first costs: projecting its block into the layer's volume, and
//! building and uploading the renderer. Then that stack thinned as each
//! Detail setting draws it on a 1920 x 1080 canvas.

use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui_wgpu::CallbackResources;

use super::channel_blend_bench_tests::{RUNS, device, median};
use super::frame_cache_bench_tests::time;
use super::surface_blend_bench_tests::orbit;
use crate::app::layers::{LayerId, VariableSelection};
use crate::app::{MeshDetail, OctantApp};
use crate::data::VolumeData;
use crate::data::octant_block::OctantBlock;
use crate::plots::oit::Transparency;
use crate::plots::{PlotType, PointCloudCallback, PointCloudRenderer};

const W: usize = 1440;
const H: usize = 720;
/// Planes of the stacks timed: 1 M to 46.7 M points.
const DEPTHS: [usize; 6] = [1, 4, 8, 16, 32, 45];
/// The default point size (`PointCloudPlotConfig`).
const POINT_SIZE: f32 = 1.05;

/// `n` values in 0..1000, none missing (every point drawn).
fn values(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| (i.wrapping_mul(2_654_435_761) % 1000) as f32)
        .collect()
}

#[test]
#[ignore = "benchmark"]
fn point_cloud_timed() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED point_cloud_timed: no GPU device");
        return;
    };
    crate::plots::limits::set_from_device(&device.limits());
    let mut resources = CallbackResources::default();
    eprintln!("point cloud frames, 1920 x 1080 (median of {RUNS} runs)");
    let max_bytes = device.limits().max_storage_buffer_binding_size;
    for d in DEPTHS {
        let n = W * H * d;
        if (n * 4) as u64 > max_bytes {
            eprintln!("{n:>9} points: past this device's storage binding");
            continue;
        }
        let (turned, still) = frames(&device, &queue, &mut resources, &values(n), (W, H));
        let m = n as f64 / 1e6;
        eprintln!("{m:>5.1} M points  orbiting {turned:>10.2?}   unchanged {still:>10.2?}");
    }
    thinned(&device, &queue, &mut resources);
    first_show(&device, &queue);
}

/// The 45 x 720 x 1440 stack thinned as a 1920 x 1080 canvas draws it at
/// each Detail setting (`OctantApp::point_grid`).
fn thinned(device: &wgpu::Device, queue: &wgpu::Queue, resources: &mut CallbackResources) {
    let d = 45;
    let volume = VolumeData::new(W, H, d, values(W * H * d), 0.0, 1000.0, String::new());
    eprintln!("the 45 x {H} x {W} stack thinned for a 1920 x 1080 canvas");
    for detail in MeshDetail::ALL {
        let mut app = OctantApp::default();
        app.layout.canvas_px = [1920, 1080];
        app.plot_configs.mesh.detail = detail;
        let Some(g) = app.point_grid([W, H, d]) else {
            eprintln!("{detail:?}: every voxel");
            continue;
        };
        let data = volume.thinned_planes(g.stride, 0..d);
        let (turned, _) = frames(device, queue, resources, &data, (g.width, g.height));
        let m = data.len() as f64 / 1e6;
        eprintln!(
            "{detail:?}: stride {}, {m:.2} M points, orbiting {turned:>10.2?}",
            g.stride
        );
    }
}

/// Median frames of `data` (`w` x `h` a plane) drawn as a point cloud while
/// orbiting and unchanged.
fn frames(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: &mut CallbackResources,
    data: &[f32],
    (w, h): (usize, usize),
) -> (Duration, Duration) {
    let points = Arc::new(PointCloudRenderer::new(
        device,
        wgpu::TextureFormat::Rgba8Unorm,
        data,
        w as u32,
        h as u32,
    ));
    let at = |step| PointCloudCallback {
        renderer: points.clone(),
        params: crate::plots::PointCloudUniformParams {
            color: crate::plots::common::PlotColorParams {
                cmax: 1000.0,
                ..Default::default()
            },
            camera: orbit(step),
            aspect_x: 1.0,
            aspect_y: 1.0,
            aspect_z: 1.0,
            point_size: POINT_SIZE,
            scale_by_value: false,
            width: w as u32,
            height: h as u32,
            shift_x: 0,
            shift_y: 0,
            shift_z: 0,
        },
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1080.0)),
        transparency: Transparency::Off,
        channels: None,
    };
    time(device, queue, resources, at)
}

/// Projecting a 45 x 720 x 1440 block into a point cloud layer's volume,
/// then building its renderer with that volume.
fn first_show(device: &wgpu::Device, queue: &wgpu::Queue) {
    let shape = [1, 45, H, W];
    let block = OctantBlock::new(
        "img".into(),
        shape.to_vec(),
        ["c", "z", "y", "x"].map(String::from).to_vec(),
        vec![0; 4],
        values(shape.iter().product()),
        Default::default(),
        std::collections::HashMap::new(),
    );
    let (mut project, mut upload) = (vec![], vec![]);
    let mut dims = (0, 0, 0);
    for _ in 0..RUNS {
        let mut app = OctantApp {
            selected: VariableSelection {
                plot_type: PlotType::PointCloud,
                ..Default::default()
            },
            ..Default::default()
        };
        let start = Instant::now();
        app.apply_3d_volume_projection(
            LayerId::BASE,
            &block,
            3,
            2,
            1,
            (0, W - 1),
            (0, H - 1),
            (0, 44),
            (0, W),
            (0, H),
            (0, 45),
            &[0, 0, 0, 0],
            false,
            true,
            0,
        );
        project.push(start.elapsed());
        let Some(volume) = app.layers.base.data.volume.as_ref() else {
            panic!("projected");
        };
        dims = (volume.width, volume.height, volume.depth);
        let start = Instant::now();
        let renderer = PointCloudRenderer::new(
            device,
            wgpu::TextureFormat::Rgba8Unorm,
            &volume.values,
            volume.width as u32,
            volume.height as u32,
        );
        queue.submit([]);
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        upload.push(start.elapsed());
        drop(renderer);
    }
    let (w, h, d) = dims;
    eprintln!("first show of 45 x {H} x {W}: a {w} x {h} x {d} volume");
    eprintln!(
        "projection {:>10.2?}   renderer and upload {:>10.2?}",
        median(project),
        median(upload)
    );
}
