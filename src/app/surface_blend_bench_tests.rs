//! What a multi-channel composite surface costs, CPU blend against GPU
//! blend, timed by hand: `cargo test --release --lib surface_blend_timed --
//! --ignored --nocapture`. The channels and sizes of `channel_blend_bench_tests`,
//! drawn as a smooth surface (one instance per cell) in a 1920 x 1080 frame.

use std::sync::Arc;
use std::time::Instant;

use eframe::egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};

use super::channel_blend_bench_tests::{CHANNELS, H, RUNS, W, block, device, frame, median};
use crate::data::slicing::{ChannelColorConfig, CompositeStretch};
use crate::plots::{ChannelDraw, ChannelParams, ChannelRenderer, Mesh3DCallback, Mesh3DRenderer};

const FRAME: [u32; 2] = [1920, 1080];

/// Prepares and paints `callback` into one frame, as egui would.
pub(super) fn draw_surface(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: &mut CallbackResources,
    callback: &Mesh3DCallback,
) {
    let screen = ScreenDescriptor {
        size_in_pixels: FRAME,
        pixels_per_point: 1.0,
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    let extra = callback.prepare(device, queue, &screen, &mut encoder, resources);
    queue.submit(extra.into_iter().chain([encoder.finish()]));
    let info = egui::PaintCallbackInfo {
        viewport: callback.rect,
        clip_rect: callback.rect,
        pixels_per_point: 1.0,
        screen_size_px: FRAME,
    };
    frame(device, queue, |rpass| {
        callback.paint(info, rpass, resources)
    });
}

/// `surface` drawn in `mode` (0 smooth, 1 steps, 2 cubes) as an RGB
/// composite, blended from `channels` when given, in a 1920 x 1080 frame.
pub(super) fn surface_callback(
    surface: &Arc<Mesh3DRenderer>,
    mode: u32,
    channels: Option<ChannelDraw>,
) -> Mesh3DCallback {
    let color = crate::plots::common::PlotColorParams {
        colormap: crate::utils::colormap::COLORMAP_RGB_COMPOSITE,
        ..Default::default()
    };
    Mesh3DCallback {
        renderer: surface.clone(),
        params: crate::plots::common::Mesh3DUniformParams {
            color,
            camera: crate::plots::camera::OrbitCamera::default().uniform(),
            displacement_strength: 0.5,
            mode,
            coord_mode: 0,
            has_reference_globe: false,
            lon_bounds: [-std::f32::consts::PI, std::f32::consts::PI],
            lat_bounds: [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            flip_height: false,
        },
        cube_mode_idx: 2,
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1080.0)),
        transparency: Default::default(),
        channels,
        thinning: crate::plots::Thinning::NONE,
    }
}

#[test]
#[ignore = "benchmark"]
fn surface_blend_timed() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED surface_blend_timed: no GPU device");
        return;
    };
    crate::plots::limits::set_from_device(&device.limits());
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let block = block();
    let mut configs: Vec<_> = (0..CHANNELS)
        .map(|i| ChannelColorConfig::new(i, format!("b{i}"), [255, (i * 60) as u8, 0]))
        .collect();
    let blend = |configs: &[ChannelColorConfig], stretch: &mut CompositeStretch| {
        crate::data::slicing::slice_multichannel_composite_nd(
            &block,
            0,
            2,
            1,
            (0, W),
            (0, H),
            &[0, 0, 0],
            configs,
            1,
            stretch,
        )
        .expect("blend")
    };
    let mut stretch = CompositeStretch::default();
    let packed = blend(&configs, &mut stretch);
    let surface = Arc::new(Mesh3DRenderer::new_surface(
        &device,
        format,
        &packed.values,
        W,
        H,
    ));
    let planes = Arc::new(ChannelRenderer::new(&device, format, CHANNELS, W * H));
    let window = crate::app::block_loading::projection_2d::PlaneWindow {
        x_dim: 2,
        y_dim: 1,
        x_range: (0, W),
        y_range: (0, H),
        fixed: &[0, 0, 0],
    };
    crate::app::block_loading::projection_channels::each_plane(&block, 0, &window, |c, p| {
        planes.write_plane(&queue, c, &p.values);
    })
    .expect("planes");
    let callback = |channels| surface_callback(&surface, 0, channels);
    let gpu_channels = |configs: &[ChannelColorConfig]| ChannelDraw {
        renderer: planes.clone(),
        params: ChannelParams::of(configs, &stretch, (0, CHANNELS), (W * H) as u32, None),
    };
    let mut resources = CallbackResources::default();
    // Warm up both paths (pipelines, colormap atlas).
    draw_surface(&device, &queue, &mut resources, &callback(None));
    draw_surface(
        &device,
        &queue,
        &mut resources,
        &callback(Some(gpu_channels(&configs))),
    );

    let (mut cpu_change, mut gpu_change, mut cpu_draw, mut gpu_draw) =
        (vec![], vec![], vec![], vec![]);
    for run in 0..RUNS {
        configs[1].color_rgb = [0, (run * 40) as u8, 255];
        // Before: a display change re-blends on the CPU, uploads, draws.
        let start = Instant::now();
        let mut fresh = stretch.clone();
        let packed = blend(&configs, &mut fresh);
        surface.update_data(&queue, &packed.values);
        draw_surface(&device, &queue, &mut resources, &callback(None));
        cpu_change.push(start.elapsed());
        // After: new uniforms, one bake into the surface's data, a frame.
        let start = Instant::now();
        let gpu = callback(Some(gpu_channels(&configs)));
        draw_surface(&device, &queue, &mut resources, &gpu);
        gpu_change.push(start.elapsed());
        // Every frame either path draws (orbiting, hovering).
        let start = Instant::now();
        draw_surface(&device, &queue, &mut resources, &callback(None));
        cpu_draw.push(start.elapsed());
        let start = Instant::now();
        draw_surface(&device, &queue, &mut resources, &gpu);
        gpu_draw.push(start.elapsed());
    }
    eprintln!("surface, {CHANNELS} channels of {W} x {H} (median of {RUNS} runs)");
    eprintln!(
        "display change, CPU blend + upload + frame {:>10.2?}",
        median(cpu_change)
    );
    eprintln!(
        "display change, GPU bake + frame          {:>10.2?}",
        median(gpu_change)
    );
    eprintln!(
        "one frame, packed colors (CPU path)        {:>10.2?}",
        median(cpu_draw)
    );
    eprintln!(
        "one frame, baked colors (GPU path)         {:>10.2?}",
        median(gpu_draw)
    );
}
