//! Cached frames of meshes and point clouds drawn without OIT: the plot is
//! drawn offscreen (its own pipelines, color and depth) only when its frame
//! key changes (`oit::FrameInputs::key`), and every paint draws that frame
//! into egui's pass with its depth (`shaders/frame_cache_blit.wgsl`), so 3D
//! coastlines drawn after it are still hidden behind the plot. Repaints that
//! change nothing the plot shows (hover, panels, toasts) skip its geometry.

use std::sync::Mutex;

use super::oit::{DEPTH_FORMAT, OitSlot, Transparency, Variant};

/// The offscreen targets of one frame.
struct Targets {
    color: wgpu::TextureView,
    depth: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    size: [u32; 2],
}

/// The blit pipeline and the last frame with its key.
struct FrameState {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    targets: Option<Targets>,
    key: Option<u64>,
}

/// A renderer's cached frame, built on first use: the callback's `prepare`
/// renders into it, `paint` draws it.
pub struct FrameSlot {
    target_format: wgpu::TextureFormat,
    state: Mutex<Option<FrameState>>,
}

impl FrameSlot {
    /// `target_format` is egui's, which the plot's pipelines draw into.
    pub fn new(target_format: wgpu::TextureFormat) -> Self {
        Self {
            target_format,
            state: Mutex::new(None),
        }
    }

    /// Draws the plot with `pipeline` (built for egui's pass) at `size`
    /// pixels into the frame, unless it was drawn there from `key`; `draw`
    /// binds the plot's group 0 and buffers and issues its draw call, the
    /// colormap `atlas` (group 1) is set. Returns whether it drew.
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        (size, key): ([u32; 2], u64),
        (pipeline, atlas): (&wgpu::RenderPipeline, &wgpu::BindGroup),
        draw: impl Fn(&mut wgpu::RenderPass<'_>),
    ) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let state = state.get_or_insert_with(|| FrameState::new(device, self.target_format));
        let held = (state.targets.as_ref().map(|t| t.size), state.key);
        let Some((size, resize)) = super::oit::frame_update(device, held, (size, key)) else {
            return false;
        };
        let (layout, format) = (&state.layout, self.target_format);
        let targets = match &mut state.targets {
            Some(t) if !resize => t,
            slot => slot.insert(Targets::new(device, layout, format, size)),
        };
        state.key = Some(key);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Cached Plot Frame"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &targets.color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &targets.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(1, atlas, &[]);
        draw(&mut pass);
        true
    }

    /// Draws the last frame into the current viewport. Returns `false` when
    /// none was drawn yet.
    pub fn paint(&self, rpass: &mut wgpu::RenderPass<'static>) -> bool {
        let state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let Some((state, targets)) = state.as_ref().and_then(|s| Some((s, s.targets.as_ref()?)))
        else {
            return false;
        };
        rpass.set_pipeline(&state.pipeline);
        rpass.set_bind_group(0, &targets.bind_group, &[]);
        rpass.draw(0..3, 0..1);
        true
    }

    /// Frees the frame (8 bytes per pixel) while the plot is not drawn this
    /// way; the next `render` draws a new one.
    pub fn release(&self) {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(s) = state.as_mut() {
            s.targets = None;
            s.key = None;
        }
    }
}

impl FrameState {
    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let texture = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cached Plot Frame Layout"),
            entries: &[
                texture(0, wgpu::TextureSampleType::Float { filterable: false }),
                texture(1, wgpu::TextureSampleType::Float { filterable: false }),
            ],
        });
        let pipeline = super::fullscreen::frame_pipeline(
            device,
            "Cached Plot Frame Blit",
            include_str!("shaders/frame_cache_blit.wgsl"),
            (&layout, true),
            target_format,
        );
        Self {
            pipeline,
            layout,
            targets: None,
            key: None,
        }
    }
}

impl Targets {
    fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
        size: [u32; 2],
    ) -> Self {
        let texture = |label, format| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let color = texture("Cached Plot Frame", target_format);
        let depth = texture("Cached Plot Frame Depth", DEPTH_FORMAT);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cached Plot Frame"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth),
                },
            ],
        });
        Self {
            color,
            depth,
            bind_group,
            size,
        }
    }
}

/// A mesh's or point cloud's frames: its OIT frame and its cached frame.
pub type Slots<'a> = (&'a OitSlot, &'a FrameSlot);

/// Renders the plot (`draw`) into the frame `transparency` draws through,
/// with its egui `pipeline` or the OIT variants `build` makes, freeing the
/// other frame; frees both without a colormap `atlas`, so `paint_frame`
/// draws no older frame.
pub fn render_frame(
    (oit, frame): Slots<'_>,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    (transparency, size, key): (Transparency, [u32; 2], u64),
    (atlas, cull_mode): (Option<&wgpu::BindGroup>, Option<wgpu::Face>),
    (pipeline, build): (
        &wgpu::RenderPipeline,
        impl Fn(&Variant<'_>) -> wgpu::RenderPipeline,
    ),
    draw: impl Fn(&mut wgpu::RenderPass<'_>),
) {
    let Some(atlas) = atlas else {
        frame.release();
        oit.release();
        return;
    };
    if transparency == Transparency::Oit {
        frame.release();
        oit.render(
            device,
            encoder,
            (size, key),
            (atlas, cull_mode),
            build,
            draw,
        );
    } else {
        oit.release();
        frame.render(device, encoder, (size, key), (pipeline, atlas), draw);
    }
}

/// Draws the frame `transparency` renders through; without one yet (no
/// colormap atlas in `prepare`) draws the plot directly with its egui
/// `pipeline` (`draw` binds group 0 and buffers and issues the draw call),
/// once the atlas is there.
pub fn paint_frame(
    (oit, frame): Slots<'_>,
    (transparency, pipeline): (Transparency, &wgpu::RenderPipeline),
    rpass: &mut wgpu::RenderPass<'static>,
    resources: &eframe::egui_wgpu::CallbackResources,
    draw: impl FnOnce(&mut wgpu::RenderPass<'static>),
) {
    let drawn = match transparency {
        Transparency::Oit => oit.paint(rpass),
        _ => frame.paint(rpass),
    };
    if drawn || !super::colormap_atlas::bind(rpass, resources) {
        return;
    }
    rpass.set_pipeline(pipeline);
    draw(rpass);
}
