//! The slicer's cut through the heatmap, surface, sphere and point cloud on
//! a GPU, each drawn through its own paint callback: a cut keeps less, and
//! what it keeps is where the box says. Skipped (with a note) without one.

use std::sync::Arc;

use eframe::egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};

use super::camera::OrbitCamera;
use super::common::{Mesh3DUniformParams, PlotColorParams};
use super::slice::SliceUniform;
use super::{
    HeatmapCallback, HeatmapRenderer, Mesh3DCallback, Mesh3DRenderer, PointCloudCallback,
    PointCloudRenderer, PointCloudUniformParams, QuadView,
};

const SIZE: u32 = 64;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn rect() -> egui::Rect {
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(SIZE as f32, SIZE as f32))
}

fn boxed(lo: [f32; 3], hi: [f32; 3]) -> SliceUniform {
    SliceUniform {
        lo: [lo[0], lo[1], lo[2], SliceUniform::BOX],
        hi: [hi[0], hi[1], hi[2], 0.0],
    }
}

fn color() -> PlotColorParams {
    PlotColorParams {
        cmax: 100.0,
        ..Default::default()
    }
}

/// The RGBA8 pixels `callback` draws on a transparent `SIZE`² target.
fn pixels(device: &wgpu::Device, queue: &wgpu::Queue, callback: &impl CallbackTrait) -> Vec<u8> {
    let mut resources = CallbackResources::default();
    let screen = ScreenDescriptor {
        size_in_pixels: [SIZE; 2],
        pixels_per_point: 1.0,
    };
    let mut encoder = device.create_command_encoder(&Default::default());
    let extra = callback.prepare(device, queue, &screen, &mut encoder, &mut resources);
    queue.submit(extra.into_iter().chain([encoder.finish()]));
    let texture = |format, usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: SIZE,
                height: SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let target = wgpu::TextureUsages::RENDER_ATTACHMENT;
    let color = texture(FORMAT, target | wgpu::TextureUsages::COPY_SRC);
    let depth = texture(wgpu::TextureFormat::Depth32Float, target);
    let (color_view, depth_view) = (
        color.create_view(&Default::default()),
        depth.create_view(&Default::default()),
    );
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut rpass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            })
            .forget_lifetime();
        let info = egui::PaintCallbackInfo {
            viewport: rect(),
            clip_rect: rect(),
            pixels_per_point: 1.0,
            screen_size_px: [SIZE; 2],
        };
        callback.paint(info, &mut rpass, &resources);
    }
    // Rows of SIZE * 4 = 256 bytes, as copies require.
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(SIZE * SIZE * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        color.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(SIZE * 4),
                rows_per_image: None,
            },
        },
        color.size(),
    );
    queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    buffer
        .slice(..)
        .get_mapped_range()
        .map(|v| v.to_vec())
        .unwrap_or_default()
}

/// Pixels drawn (alpha above zero), and how many in the left half.
fn coverage(pixels: &[u8]) -> (usize, usize) {
    let drawn = |(i, p): &(usize, &[u8; 4])| p[3] > 0 && (i % SIZE as usize) < SIZE as usize / 2;
    let all = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 0)
        .count();
    let left = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(drawn)
        .count();
    (all, left)
}

#[test]
fn gpu_slicer_cuts_the_heatmap_quad() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED gpu_slicer_cuts_the_heatmap_quad: no GPU device");
        return;
    };
    let heatmap = Arc::new(HeatmapRenderer::new(&device, FORMAT, &[50.0; 16], 4, 4));
    let draw = |slice| HeatmapCallback {
        renderer: heatmap.clone(),
        color_params: color(),
        rect: rect(),
        view: QuadView {
            slice,
            ..Default::default()
        },
        labels: None,
        channels: None,
    };
    let (whole, _) = coverage(&pixels(&device, &queue, &draw(SliceUniform::OFF)));
    let (kept, left) = coverage(&pixels(
        &device,
        &queue,
        &draw(boxed([0.0; 3], [0.5, 1.0, 1.0])),
    ));
    assert_eq!(whole, (SIZE * SIZE) as usize, "the quad fills the target");
    assert_eq!(
        (kept, left),
        (whole / 2, whole / 2),
        "only the left half stays"
    );
}

fn mesh(
    sphere: bool,
    mode: u32,
    slice: SliceUniform,
    renderer: &Arc<Mesh3DRenderer>,
) -> Mesh3DCallback {
    let camera = OrbitCamera {
        rotation_x: 0.6,
        ..OrbitCamera::default()
    };
    Mesh3DCallback {
        renderer: renderer.clone(),
        params: Mesh3DUniformParams {
            slice,
            color: color(),
            camera: camera.uniform(),
            displacement_strength: 0.5,
            mode,
            coord_mode: 0,
            has_reference_globe: false,
            lon_bounds: [-std::f32::consts::PI, std::f32::consts::PI],
            lat_bounds: [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            flip_height: false,
        },
        cube_mode_idx: if sphere { 3 } else { 2 },
        rect: rect(),
        transparency: Default::default(),
        channels: None,
        thinning: super::Thinning::NONE,
    }
}

#[test]
fn gpu_slicer_cuts_surfaces_and_spheres() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED gpu_slicer_cuts_surfaces_and_spheres: no GPU device");
        return;
    };
    let values = [50.0; 16 * 8];
    let surface = Arc::new(Mesh3DRenderer::new_surface(&device, FORMAT, &values, 16, 8));
    let sphere = Arc::new(Mesh3DRenderer::new_sphere(&device, FORMAT, &values, 16, 8));
    // The app hands meshes w as normalized heights (`Slicer::height_range`).
    let half = boxed([0.0, 0.0, f32::MIN], [0.5, 1.0, f32::MAX]);
    for (renderer, is_sphere, modes) in [(&surface, false, 0..3), (&sphere, true, 0..4)] {
        for mode in modes {
            let draw = |slice| pixels(&device, &queue, &mesh(is_sphere, mode, slice, renderer));
            let (whole, _) = coverage(&draw(SliceUniform::OFF));
            let (kept, _) = coverage(&draw(half));
            let what = format!("sphere {is_sphere} mode {mode}");
            assert!(whole > 100, "{what} draws: {whole}");
            assert!(
                kept > 0 && kept < whole,
                "{what}: half kept {kept} of {whole}"
            );
        }
    }
    // Every block holds 50 (height 0.5): a box above it keeps no block, whole.
    let above = boxed([0.0, 0.0, 0.6], [1.0, 1.0, f32::MAX]);
    let blocks = pixels(&device, &queue, &mesh(false, 2, above, &surface));
    assert_eq!(
        coverage(&blocks).0,
        0,
        "blocks are cut whole by their value"
    );
}

#[test]
fn gpu_slicer_cuts_point_clouds() {
    let Some((device, queue)) = super::test_gpu::device() else {
        eprintln!("SKIPPED gpu_slicer_cuts_point_clouds: no GPU device");
        return;
    };
    let n = 8u32;
    let points = Arc::new(PointCloudRenderer::new(&device, FORMAT, &[50.0; 512], n, n));
    let draw = |slice| {
        let callback = PointCloudCallback {
            renderer: points.clone(),
            params: PointCloudUniformParams {
                slice,
                color: color(),
                camera: OrbitCamera::default().uniform(),
                aspect_x: 1.0,
                aspect_y: 1.0,
                aspect_z: 1.0,
                point_size: 1.0,
                scale_by_value: false,
                width: n,
                height: n,
                shift_x: 0,
                shift_y: 0,
                shift_z: 0,
            },
            rect: rect(),
            transparency: Default::default(),
            channels: None,
        };
        coverage(&pixels(&device, &queue, &callback))
    };
    let (whole, _) = draw(SliceUniform::OFF);
    let (half, left) = draw(boxed([0.0; 3], [0.5, 1.0, 1.0]));
    assert!(half > 0 && half < whole, "half kept {half} of {whole}");
    assert!(
        left as f32 > 0.9 * half as f32,
        "kept on the left: {left} of {half}"
    );
    let plane = |at: [f32; 3]| SliceUniform {
        lo: [at[0], at[1], at[2], SliceUniform::PLANES],
        hi: [1.0, 1.0, 1.0, 0.0],
    };
    assert_eq!(draw(plane([-1.0; 3])).0, 0, "no plane shown draws nothing");
    let (one, _) = draw(plane([-1.0, -1.0, 0.0]));
    assert!(one > 0 && one < whole, "the front plane: {one} of {whole}");
}
