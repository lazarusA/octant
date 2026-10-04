//! Volume render pipeline: shader assembly, bind group layout and sampler.

/// Assembles the volume shader around `$filter`, the file defining `fetch_trilinear`.
/// Only the colormap prelude is needed (`ColorUniforms`, `evaluate_scaled_norm`);
/// colors come from the transfer LUT, so the atlas at `@group(1)` stays unbound.
macro_rules! volume_shader {
    ($filter:expr) => {
        concat!(
            include_str!("../shaders/colormaps/mod.wgsl"),
            "\n",
            include_str!("../shaders/volume/bindings.wgsl"),
            "\n",
            $filter,
            "\n",
            include_str!("../shaders/volume/sample.wgsl"),
            "\n",
            include_str!("../shaders/volume/shading.wgsl"),
            "\n",
            include_str!("../shaders/volume/dvr.wgsl"),
            "\n",
            include_str!("../shaders/volume/projections.wgsl"),
            "\n",
            include_str!("../shaders/volume/rgba.wgsl"),
            "\n",
            include_str!("../shaders/volume/main.wgsl"),
        )
    };
}

/// Shader for devices that filter `R32Float` in hardware.
pub const SHADER_HARDWARE_FILTER: &str =
    volume_shader!(include_str!("../shaders/volume/filter_hardware.wgsl"));
/// Shader blending eight texel loads where `R32Float` is not filterable.
pub const SHADER_MANUAL_FILTER: &str =
    volume_shader!(include_str!("../shaders/volume/filter_manual.wgsl"));

/// Whether the device filters `R32Float` textures (`FLOAT32_FILTERABLE`).
pub fn hardware_filtering(device: &wgpu::Device) -> bool {
    device
        .features()
        .contains(wgpu::Features::FLOAT32_FILTERABLE)
}

fn texture_entry(
    binding: u32,
    filterable: bool,
    dim: wgpu::TextureViewDimension,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable },
            view_dimension: dim,
            multisampled: false,
        },
        count: None,
    }
}

/// `@group(0)`: uniforms, values, validity, transfer LUT and the shared sampler.
pub fn bind_group_layout(device: &wgpu::Device, hardware_filter: bool) -> wgpu::BindGroupLayout {
    use wgpu::TextureViewDimension::{D2, D3};
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Volume Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            texture_entry(1, hardware_filter, D3),
            texture_entry(2, true, D3),
            texture_entry(3, true, D2),
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Linear sampler repeating on every axis: the shader clamps logical
/// coordinates to the edge texel centers, so only ring-buffer seams wrap.
pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Volume Sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    })
}

pub fn create_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    layout: &wgpu::BindGroupLayout,
    hardware_filter: bool,
) -> wgpu::RenderPipeline {
    let source = if hardware_filter {
        SHADER_HARDWARE_FILTER
    } else {
        SHADER_MANUAL_FILTER
    };
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Volume Raymarching Shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Volume Pipeline Layout"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Volume Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                // The raymarcher composites front to back, so it emits premultiplied color.
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            // March from back faces only: one fragment per pixel, and the box
            // stays visible with the camera inside it.
            cull_mode: Some(wgpu::Face::Front),
            ..Default::default()
        },
        depth_stencil: Some(crate::plots::common::default_depth_stencil_state(
            false,
            wgpu::CompareFunction::Always,
        )),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
