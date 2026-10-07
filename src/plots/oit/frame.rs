//! Offscreen OIT targets and the composite pipeline resolving them in egui's pass.

use super::{ACCUM_FORMAT, DEPTH_FORMAT, OPAQUE_FORMAT, REVEAL_FORMAT};

/// The four targets of one OIT frame, sized to the plot viewport in pixels.
pub struct OitFrame {
    pub opaque: wgpu::TextureView,
    pub depth: wgpu::TextureView,
    pub accum: wgpu::TextureView,
    pub reveal: wgpu::TextureView,
    /// The composited targets, bound for [`Compositor::draw`].
    pub bind_group: wgpu::BindGroup,
    pub size: [u32; 2],
}

/// Pipeline drawing an [`OitFrame`] over the plot viewport.
pub struct Compositor {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn target(
    device: &wgpu::Device,
    label: &str,
    format: wgpu::TextureFormat,
    size: [u32; 2],
) -> wgpu::TextureView {
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

impl Compositor {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("OIT Composite Layout"),
            entries: &[texture_entry(0), texture_entry(1), texture_entry(2)],
        });
        Self {
            pipeline: crate::plots::fullscreen::frame_pipeline(
                device,
                "OIT Composite Pipeline",
                include_str!("../shaders/oit/composite.wgsl"),
                &layout,
                target_format,
            ),
            layout,
        }
    }

    pub fn create_frame(&self, device: &wgpu::Device, size: [u32; 2]) -> OitFrame {
        let opaque = target(device, "OIT Opaque", OPAQUE_FORMAT, size);
        let depth = target(device, "OIT Depth", DEPTH_FORMAT, size);
        let accum = target(device, "OIT Accum", ACCUM_FORMAT, size);
        let reveal = target(device, "OIT Reveal", REVEAL_FORMAT, size);
        let entry = |binding, view| wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("OIT Composite Bind Group"),
            layout: &self.layout,
            entries: &[entry(0, &opaque), entry(1, &accum), entry(2, &reveal)],
        });
        OitFrame {
            opaque,
            depth,
            accum,
            reveal,
            bind_group,
            size,
        }
    }

    pub fn draw(&self, rpass: &mut wgpu::RenderPass<'_>, frame: &OitFrame) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &frame.bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}
