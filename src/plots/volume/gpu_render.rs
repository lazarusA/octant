//! Test-only offscreen rendering of volume renderers (shared by the GPU tests
//! and the contact sheet).

use super::{VolumeEncoding, VolumeRenderer, VolumeUniformParams};
use crate::plots::common::PlotColorParams;
use crate::utils::colormap::registry;

/// Stand-in for egui's target format (only the blit draws into it).
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Device with `FLOAT32_FILTERABLE` when the adapter has it, and whether it does.
pub(super) fn gpu() -> Option<(wgpu::Device, wgpu::Queue, bool)> {
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok()?;
        let filterable = adapter.features() & wgpu::Features::FLOAT32_FILTERABLE;
        let descriptor = wgpu::DeviceDescriptor {
            required_features: filterable,
            ..Default::default()
        };
        let (device, queue) = adapter.request_device(&descriptor).await.ok()?;
        Some((device, queue, !filterable.is_empty()))
    })
}

pub(super) fn params(algorithm: u32, quality: f32, shift_x: u32) -> VolumeUniformParams {
    VolumeUniformParams {
        color: PlotColorParams {
            colormap: registry::default_id(),
            cmin: 0.0,
            cmax: 1.0,
            ..Default::default()
        },
        rot_y: 0.6,
        rot_x: 0.4,
        aspect_x: 1.0,
        aspect_y: 1.0,
        aspect_z: 1.0,
        zoom: 2.5,
        opacity_scale: 3.0,
        quality,
        algorithm,
        isovalue: 0.5,
        isorange: 0.1,
        attenuation: 0.0,
        screen_aspect: 1.0,
        shift_x,
        shift_y: 0,
        shift_z: 0,
        transparency: true,
        lighting: true,
    }
}

/// Premultiplied RGBA8 pixels of one `size`² frame of `renderer` with `p`.
pub(super) fn render_sized(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &VolumeRenderer,
    p: &VolumeUniformParams,
    size: u32,
) -> Vec<u8> {
    let mut encoder = device.create_command_encoder(&Default::default());
    renderer.render_frame(device, queue, &mut encoder, p, [size, size]);
    let color = renderer.frame_texture().expect("rendered frame");
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(size * size * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        color.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size * 4),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    readback
        .slice(..)
        .get_mapped_range()
        .map(|v| v.to_vec())
        .unwrap_or_default()
}

/// Renderer for an `n`³ scalar volume.
pub(super) fn renderer_n(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[f32],
    n: usize,
    hardware: bool,
) -> VolumeRenderer {
    renderer_wh(device, queue, data, [n, n], hardware)
}

/// Renderer for a scalar volume of `w`×`h` planes (depth from `data`).
pub(super) fn renderer_wh(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[f32],
    [w, h]: [usize; 2],
    hardware: bool,
) -> VolumeRenderer {
    let geometry = (data, w as u32, h as u32);
    VolumeRenderer::build(
        device,
        queue,
        FORMAT,
        geometry,
        VolumeEncoding::Scalar,
        hardware,
    )
    .expect("small volumes fit any device")
}
