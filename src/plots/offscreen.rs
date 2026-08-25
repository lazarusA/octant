use std::sync::Arc;

/// An offscreen GPU render target for high-throughput headless animation rendering.
pub struct OffscreenTarget {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub staging_buffer: wgpu::Buffer,
    pub padded_bytes_per_row: u32,
    pub unpadded_bytes_per_row: u32,
}

impl OffscreenTarget {
    /// Creates a new offscreen render target with matching staging buffer and depth texture.
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Octant Offscreen Render Texture"),
            size: wgpu::Extent3d {
                width: width.max(2),
                height: height.max(2),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Octant Offscreen Depth Texture"),
            size: wgpu::Extent3d {
                width: width.max(2),
                height: height.max(2),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let unpadded_bytes_per_row = width.max(2) * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) & !(align - 1);
        let buffer_size = (padded_bytes_per_row * height.max(2)) as u64;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Octant Offscreen Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            width: width.max(2),
            height: height.max(2),
            format,
            texture,
            view,
            depth_texture,
            depth_view,
            staging_buffer,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
        }
    }

    /// Executes a render pass on the offscreen texture, copies to staging, and extracts RGBA pixels.
    pub fn render_frame<F>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        clear_color: wgpu::Color,
        draw_fn: F,
    ) -> Result<Arc<[u8]>, String>
    where
        F: FnOnce(&mut wgpu::RenderPass<'_>),
    {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Octant Offscreen Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Octant Offscreen Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rpass.set_viewport(0.0, 0.0, self.width as f32, self.height as f32, 0.0, 1.0);
            rpass.set_scissor_rect(0, 0, self.width, self.height);
            draw_fn(&mut rpass);
        }

        self.copy_to_staging(&mut encoder);
        queue.submit(Some(encoder.finish()));

        self.read_pixels(device)
    }

    /// Enqueues copying the offscreen texture into the staging buffer.
    pub fn copy_to_staging(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Synchronously maps the staging buffer (for fast headless rendering) and extracts RGBA pixels.
    pub fn read_pixels(&self, device: &wgpu::Device) -> Result<Arc<[u8]>, String> {
        let buffer_slice = self.staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        rx.recv()
            .map_err(|e| format!("Channel receive error: {}", e))?
            .map_err(|e| format!("Failed to map staging buffer: {:?}", e))?;

        let mapped_view = buffer_slice
            .get_mapped_range()
            .map_err(|e| format!("Failed to get mapped range: {:?}", e))?;
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);

        let is_bgra = matches!(
            self.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );

        let padded_row = self.padded_bytes_per_row as usize;
        let unpadded_row = self.unpadded_bytes_per_row as usize;

        for row in 0..(self.height as usize) {
            let start = row * padded_row;
            let end = start + unpadded_row;
            if end <= mapped_view.len() {
                let row_bytes = &mapped_view[start..end];
                if is_bgra {
                    for chunk in row_bytes.chunks_exact(4) {
                        rgba.push(chunk[2]); // R
                        rgba.push(chunk[1]); // G
                        rgba.push(chunk[0]); // B
                        rgba.push(chunk[3]); // A
                    }
                } else {
                    rgba.extend_from_slice(row_bytes);
                }
            }
        }

        drop(mapped_view);
        self.staging_buffer.unmap();

        Ok(Arc::from(rgba.into_boxed_slice()))
    }
}
