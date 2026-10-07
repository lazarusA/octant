//! Per-renderer OIT pipelines and frame, built on first use.

use super::frame::{Compositor, OitFrame};
use super::{ACCUMULATE, OPAQUE, Variant};
use std::sync::Mutex;

pub struct OitState {
    opaque: wgpu::RenderPipeline,
    accumulate: wgpu::RenderPipeline,
    compositor: Compositor,
    frame: Option<OitFrame>,
}

impl OitState {
    /// `build` creates the renderer's pipeline for a [`Variant`] (its own
    /// vertex stage and layout, no culling).
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        build: impl Fn(&Variant<'_>) -> wgpu::RenderPipeline,
    ) -> Self {
        Self {
            opaque: build(&OPAQUE),
            accumulate: build(&ACCUMULATE),
            compositor: Compositor::new(device, target_format),
            frame: None,
        }
    }

    /// Records the opaque and translucent passes at `size` pixels into
    /// `encoder`. `draw` binds the renderer's group 0 and buffers and issues
    /// its draw call; the pipeline and the colormap `atlas` (group 1) are set.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        size: [u32; 2],
        atlas: &wgpu::BindGroup,
        draw: impl Fn(&mut wgpu::RenderPass<'_>),
    ) {
        let max = device.limits().max_texture_dimension_2d;
        let size = size.map(|s| s.clamp(1, max));
        if self.frame.as_ref().is_none_or(|f| f.size != size) {
            self.frame = Some(self.compositor.create_frame(device, size));
        }
        let Some(frame) = &self.frame else {
            return;
        };
        let clear = |view, color| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })
        };
        let depth = |load| {
            Some(wgpu::RenderPassDepthStencilAttachment {
                view: &frame.depth,
                depth_ops: Some(wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            })
        };
        let passes = [
            (
                &self.opaque,
                [clear(&frame.opaque, wgpu::Color::TRANSPARENT), None],
                wgpu::LoadOp::Clear(1.0),
            ),
            (
                &self.accumulate,
                [
                    clear(&frame.accum, wgpu::Color::TRANSPARENT),
                    clear(&frame.reveal, wgpu::Color::WHITE),
                ],
                wgpu::LoadOp::Load,
            ),
        ];
        for (pipeline, colors, depth_load) in passes {
            let targets = if colors[1].is_some() {
                &colors[..]
            } else {
                &colors[..1]
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("OIT Pass"),
                color_attachments: targets,
                depth_stencil_attachment: depth(depth_load),
                ..Default::default()
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, atlas, &[]);
            draw(&mut pass);
        }
    }

    /// Frees the frame's targets (about 18 bytes per pixel) while OIT is not
    /// used; the pipelines stay, and the next [`render`](Self::render)
    /// allocates a new frame.
    pub fn release_frame(&mut self) {
        self.frame = None;
    }

    /// Composites the last rendered frame into the current viewport. Returns
    /// `false` when no frame was rendered yet.
    pub fn paint(&self, rpass: &mut wgpu::RenderPass<'_>) -> bool {
        let Some(frame) = &self.frame else {
            return false;
        };
        self.compositor.draw(rpass, frame);
        true
    }
}

/// A renderer's [`OitState`], built on first use: the callback's `prepare`
/// renders into it, `paint` composites it.
pub struct OitSlot {
    target_format: wgpu::TextureFormat,
    state: Mutex<Option<OitState>>,
}

impl OitSlot {
    /// `target_format` is egui's, which the composite draws into.
    pub fn new(target_format: wgpu::TextureFormat) -> Self {
        Self {
            target_format,
            state: Mutex::new(None),
        }
    }

    /// [`OitState::render`], building the state first with `build` (the
    /// renderer's pipeline for a [`Variant`]).
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        size: [u32; 2],
        atlas: &wgpu::BindGroup,
        build: impl Fn(&Variant<'_>) -> wgpu::RenderPipeline,
        draw: impl Fn(&mut wgpu::RenderPass<'_>),
    ) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        state
            .get_or_insert_with(|| OitState::new(device, self.target_format, build))
            .render(device, encoder, size, atlas, draw);
    }

    /// [`OitState::paint`]; `false` when nothing was rendered yet.
    pub fn paint(&self, rpass: &mut wgpu::RenderPass<'_>) -> bool {
        let state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        state.as_ref().is_some_and(|s| s.paint(rpass))
    }

    /// Frees the frame when the renderer draws without OIT (see
    /// [`OitState::release_frame`]).
    pub fn release(&self) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(s) = state.as_mut() {
            s.release_frame();
        }
    }
}
