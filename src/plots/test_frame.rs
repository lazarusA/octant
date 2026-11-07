//! Test-only offscreen frames for the plot GPU tests: a paint callback
//! drawn on a small transparent target through its own `prepare` and
//! `paint`, and its pixels read back.

use eframe::egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};

pub(crate) const SIZE: u32 = 64;
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(crate) fn rect() -> egui::Rect {
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(SIZE as f32, SIZE as f32))
}

/// The RGBA8 pixels `callback` draws on a transparent `SIZE`² target.
pub(crate) fn pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    callback: &impl CallbackTrait,
) -> Vec<u8> {
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
pub(crate) fn coverage(pixels: &[u8]) -> (usize, usize) {
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
