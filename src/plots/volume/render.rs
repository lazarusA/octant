//! Frame rendering: raymarches into the cached offscreen frame when its key
//! changed, and blits the frame in egui's pass.

use super::frame::{Frame, FrameKey};
use super::{VolumeRenderer, VolumeUniformParams, pipeline};
use crate::utils::colormap::registry;

/// Rendering modes (WGSL `ALGORITHM`); higher values select the last one.
pub const ALGORITHMS: usize = 8;

/// Per-renderer GPU state built on demand.
#[derive(Default)]
pub struct RenderCache {
    pipelines: [Option<wgpu::RenderPipeline>; ALGORITHMS],
    frame: Option<Frame>,
    key: Option<FrameKey>,
}

impl VolumeRenderer {
    /// Brings the cached frame up to date for `params` at `size` pixels,
    /// recording the raymarch into `encoder` only when the frame's inputs
    /// changed. Returns whether it re-rendered.
    pub fn render_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        params: &VolumeUniformParams,
        size: [u32; 2],
    ) -> bool {
        let max = device.limits().max_texture_dimension_2d;
        let size = size.map(|s| s.clamp(1, max));
        let key = FrameKey {
            uniforms: params.to_uniforms(self.state()),
            data_version: self.textures.version(),
            generation: registry::generation(),
            size,
        };
        let mut cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        if cache.key == Some(key) && cache.frame.is_some() {
            return false;
        }
        self.transfer
            .sync(queue, &params.color, params.opacity_scale);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&key.uniforms));
        if cache.frame.as_ref().is_none_or(|f| f.size != size) {
            cache.frame = Some(self.blit.create_frame(device, size));
        }
        let algorithm = (params.algorithm as usize).min(ALGORITHMS - 1);
        let pipeline = cache.pipelines[algorithm]
            .get_or_insert_with(|| {
                pipeline::create_raymarch_pipeline(
                    device,
                    &self.module,
                    &self.pipeline_layout,
                    algorithm as u32,
                )
            })
            .clone();
        if let Some(frame) = &cache.frame {
            self.raymarch(encoder, &pipeline, &frame.view);
        }
        cache.key = Some(key);
        true
    }

    fn raymarch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        target: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Volume Raymarch Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        // The bounding box's 36 vertices come from `vs_main`'s vertex index.
        pass.draw(0..36, 0..1);
    }

    /// Draws the cached frame into the current viewport. Returns `false` when
    /// no frame was rendered yet.
    pub fn paint_frame(&self, rpass: &mut wgpu::RenderPass<'static>) -> bool {
        let cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        let Some(frame) = &cache.frame else {
            return false;
        };
        self.blit.draw(rpass, &frame.bind_group);
        true
    }

    /// The cached frame's texture (tests read it back).
    #[cfg(test)]
    pub(super) fn frame_texture(&self) -> Option<wgpu::Texture> {
        let cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        cache.frame.as_ref().map(|f| f.texture.clone())
    }
}
