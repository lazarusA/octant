//! Pipelines drawing an offscreen frame over the plot viewport in egui's pass
//! (volume blit, OIT composite): the shared full-viewport triangle
//! (`shaders/common/fullscreen.wgsl`) and a premultiplied-alpha fragment stage.

/// Full-viewport triangle WGSL, prepended to each frame shader.
pub const FULLSCREEN_WGSL: &str = include_str!("shaders/common/fullscreen.wgsl");

/// Pipeline running `fs_main` of `fragment_wgsl` (which reads `FullscreenOutput`)
/// over the viewport, with its frame bound at group 0 through `layout`.
pub fn frame_pipeline(
    device: &wgpu::Device,
    label: &str,
    fragment_wgsl: &str,
    layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(format!("{FULLSCREEN_WGSL}\n{fragment_wgsl}").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_fullscreen"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        // egui's pass carries a depth attachment; the frame ignores it.
        depth_stencil: Some(super::common::default_depth_stencil_state(
            false,
            wgpu::CompareFunction::Always,
        )),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
