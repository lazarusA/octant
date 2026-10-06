//! Cached offscreen frame of the volume and the blit drawing it into egui's
//! pass: the raymarch reruns only when something it depends on changes, not
//! on every repaint of the UI around it.

use super::pipeline::FRAME_FORMAT;
use super::uniforms::VolumeUniforms;

/// Everything a frame's pixels depend on.
#[derive(Copy, Clone)]
pub struct FrameKey {
    pub uniforms: VolumeUniforms,
    /// Upload counter of the volume textures.
    pub data_version: u64,
    /// Colormap registry generation (custom map edits).
    pub generation: u64,
    pub size: [u32; 2],
}

impl PartialEq for FrameKey {
    fn eq(&self, other: &Self) -> bool {
        bytemuck::bytes_of(&self.uniforms) == bytemuck::bytes_of(&other.uniforms)
            && self.data_version == other.data_version
            && self.generation == other.generation
            && self.size == other.size
    }
}

/// Offscreen target of the raymarcher, bound for the blit.
pub struct Frame {
    /// Owned here; read back only by tests.
    #[cfg_attr(not(test), allow(dead_code))]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub size: [u32; 2],
}

/// Pipeline drawing a [`Frame`] over the plot viewport in egui's pass.
pub struct Blit {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl Blit {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Volume Blit Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Volume Blit Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            pipeline: blit_pipeline(device, target_format, &layout),
            layout,
            sampler,
        }
    }

    pub fn create_frame(&self, device: &wgpu::Device, size: [u32; 2]) -> Frame {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Volume Frame"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FRAME_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Volume Blit Bind Group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        Frame {
            texture,
            view,
            bind_group,
            size,
        }
    }

    pub fn draw(&self, rpass: &mut wgpu::RenderPass<'static>, frame: &wgpu::BindGroup) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, frame, &[]);
        rpass.draw(0..3, 0..1);
    }
}

fn blit_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Volume Blit Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/volume/blit.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Volume Blit Pipeline Layout"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Volume Blit Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_blit"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_blit"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        // egui's pass carries a depth attachment; the blit ignores it.
        depth_stencil: Some(crate::plots::common::default_depth_stencil_state(
            false,
            wgpu::CompareFunction::Always,
        )),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
