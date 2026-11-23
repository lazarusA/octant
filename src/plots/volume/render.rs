//! Frame rendering: raymarches into the cached offscreen frame when its key
//! changed, and blits the frame in egui's pass. A frame can be built in
//! bands of rows over several egui frames (a bricked volume at rest, whose
//! full frame can take most of a second on an integrated GPU): the bands
//! go into a second frame shown once complete, the last one staying up
//! meanwhile, so the UI never waits on the whole raymarch.

use super::frame::{Frame, FrameKey};
use super::{VolumeRenderer, VolumeUniformParams, pipeline};
use crate::utils::colormap::registry;
use eframe::egui_wgpu::CallbackResources;

/// Rendering modes (WGSL `ALGORITHM`); higher values select the last one.
pub const ALGORITHMS: usize = 8;

/// Per-renderer GPU state built on demand.
#[derive(Default)]
pub struct RenderCache {
    pipelines: [Option<wgpu::RenderPipeline>; ALGORITHMS],
    frame: Option<Frame>,
    key: Option<FrameKey>,
    /// The frame being built in bands, its key and the next band.
    next: Option<Frame>,
    next_key: Option<FrameKey>,
    band: u32,
}

impl VolumeRenderer {
    /// Brings the cached frame up to date for `params` at `size` pixels,
    /// recording the raymarch into `encoder` only when the frame's inputs
    /// changed (colors from the atlas in `resources`, which `prepare` keeps
    /// current). Returns whether it re-rendered.
    pub fn render_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        params: &VolumeUniformParams,
        size: [u32; 2],
        resources: &CallbackResources,
    ) -> bool {
        let target = (params, size, 1);
        self.render_frame_banded(device, queue, encoder, target, resources)
    }

    /// [`Self::render_frame`] in `bands` bands of rows, one per call, the
    /// shown frame swapped once the last is done. Returns whether it
    /// rendered a band.
    pub fn render_frame_banded(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        (params, size, bands): (&VolumeUniformParams, [u32; 2], u32),
        resources: &CallbackResources,
    ) -> bool {
        let Some(atlas) = crate::plots::colormap_atlas::bind_group(resources) else {
            return false;
        };
        let max = device.limits().max_texture_dimension_2d;
        let size = size.map(|s| s.clamp(1, max));
        let mut cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        // Nothing shown yet: one whole frame now beats bands over an empty
        // canvas (the banded build follows on the next frame).
        let bands = match cache.frame {
            Some(_) => bands.clamp(1, size[1]),
            None => 1,
        };
        let key = self.frame_key(params, size, bands > 1);
        if cache.key == Some(key) && cache.frame.is_some() {
            return false;
        }
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&key.uniforms));
        let algorithm = (params.algorithm as usize).min(ALGORITHMS - 1);
        self.ensure_pipeline(&mut cache, device, algorithm);
        if bands == 1 {
            self.render_whole(&mut cache, device, encoder, (key, algorithm), atlas);
            return true;
        }
        self.start_build(&mut cache, device, key);
        let band = cache.band;
        if let (0, Some(b)) = (band, &self.bricks) {
            b.feedback.clear(encoder);
        }
        let rows = (size[1] * band / bands)..(size[1] * (band + 1) / bands);
        if let (Some(p), Some(f)) = (&cache.pipelines[algorithm], &cache.next) {
            self.raymarch(encoder, p, atlas, &f.view, (rows, size[0], band == 0));
        }
        self.finish_band(&mut cache, encoder, (params, bands));
        true
    }

    /// Counts the band drawn; after the last, records the frame's ray
    /// feedback and shows the frame.
    fn finish_band(
        &self,
        cache: &mut RenderCache,
        encoder: &mut wgpu::CommandEncoder,
        (params, bands): (&VolumeUniformParams, u32),
    ) {
        cache.band += 1;
        if cache.band < bands {
            return;
        }
        if let Some(b) = &self.bricks {
            b.feedback.record(encoder, *params);
        }
        // The finished frame shows; the old one is reused for the next.
        let done = cache.next.take();
        cache.next = std::mem::replace(&mut cache.frame, done);
        cache.key = cache.next_key.take();
    }

    /// Builds the raymarch pipeline of `algorithm` unless built.
    fn ensure_pipeline(&self, cache: &mut RenderCache, device: &wgpu::Device, algorithm: usize) {
        if cache.pipelines[algorithm].is_none() {
            cache.pipelines[algorithm] = Some(pipeline::create_raymarch_pipeline(
                device,
                &self.module,
                &self.pipeline_layout,
                algorithm as u32,
                self.bricks.is_some(),
            ));
        }
    }

    /// Renders the frame of `key` whole, straight into the shown frame.
    fn render_whole(
        &self,
        cache: &mut RenderCache,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        (key, algorithm): (FrameKey, usize),
        atlas: &wgpu::BindGroup,
    ) {
        let size = key.size;
        let shown = cache.frame.take().filter(|f| f.size == size);
        cache.frame = Some(shown.unwrap_or_else(|| self.blit.create_frame(device, size)));
        cache.next_key = None;
        if let (Some(p), Some(f)) = (&cache.pipelines[algorithm], &cache.frame) {
            self.raymarch(encoder, p, atlas, &f.view, (0..size[1], size[0], true));
        }
        cache.key = Some(key);
    }

    /// Starts a band build of `key` unless one of the same view is under
    /// way. Bricks arriving mid-build (every batch, faster than 16 bands)
    /// join its remaining bands instead of restarting it, else the frame
    /// never completes while a view streams in; the build keeps the data
    /// version it started with, so the finished frame is stale and the next
    /// build draws the new bricks in every band.
    fn start_build(&self, cache: &mut RenderCache, device: &wgpu::Device, key: FrameKey) {
        if cache.next_key.map(without_data) == Some(without_data(key)) {
            return;
        }
        let reuse = cache.next.take().filter(|f| f.size == key.size);
        cache.next = Some(reuse.unwrap_or_else(|| self.blit.create_frame(device, key.size)));
        cache.next_key = Some(key);
        cache.band = 0;
    }

    /// Everything the frame of `params` at `size` depends on.
    /// A `banded` bricked frame records ray feedback.
    pub(super) fn frame_key(
        &self,
        params: &VolumeUniformParams,
        size: [u32; 2],
        banded: bool,
    ) -> FrameKey {
        let mut uniforms = params.to_uniforms(self.state());
        uniforms.record_seen = u32::from(banded && self.bricks.is_some());
        FrameKey {
            uniforms,
            data_version: self.textures.as_ref().map_or(0, |t| t.version())
                + self.bricks.as_ref().map_or(0, |b| b.version()),
            generation: registry::generation(),
            size,
        }
    }

    /// Raymarches the rows `rows` (all `width` pixels) into `target`,
    /// clearing it first when `clear`.
    fn raymarch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        atlas: &wgpu::BindGroup,
        target: &wgpu::TextureView,
        (rows, width, clear): (std::ops::Range<u32>, u32, bool),
    ) {
        let load = match clear {
            true => wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            false => wgpu::LoadOp::Load,
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Volume Raymarch Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_scissor_rect(
            0,
            rows.start,
            width,
            rows.end.saturating_sub(rows.start).max(1),
        );
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_bind_group(1, atlas, &[]);
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
}

#[path = "render_queries.rs"]
mod queries;

/// `key` with its data version cleared: what a band build restarts on.
fn without_data(key: FrameKey) -> FrameKey {
    FrameKey {
        data_version: 0,
        ..key
    }
}
