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
            include_str!("../shaders/volume/skip.wgsl"),
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

/// `@group(0)`: uniforms, values, validity, transfer LUT, the shared sampler
/// and the brick grid.
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
            texture_entry(5, false, D3),
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

/// Format of the cached offscreen frame the raymarcher renders into.
pub const FRAME_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub fn create_module(device: &wgpu::Device, hardware_filter: bool) -> wgpu::ShaderModule {
    let source = if hardware_filter {
        SHADER_HARDWARE_FILTER
    } else {
        SHADER_MANUAL_FILTER
    };
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Volume Raymarching Shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

pub fn create_pipeline_layout(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Volume Pipeline Layout"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    })
}

/// Raymarching pipeline for mode `algorithm` (the WGSL `ALGORITHM` override),
/// drawing into a cleared [`FRAME_FORMAT`] frame: every pixel is written once,
/// already premultiplied, so neither blending nor depth is needed.
pub fn create_raymarch_pipeline(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    algorithm: u32,
) -> wgpu::RenderPipeline {
    let constants = [("ALGORITHM", f64::from(algorithm))];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Volume Raymarch Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: FRAME_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &constants,
                ..Default::default()
            },
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            // March from back faces only: one fragment per pixel, and the box
            // stays visible with the camera inside it.
            cull_mode: Some(wgpu::Face::Front),
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
