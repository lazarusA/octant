//! Pipeline variants of the 3D plot renderers and the one builder for them:
//! egui-pass pipelines (`fs_main`) and the OIT passes ([`OPAQUE`], [`ACCUMULATE`]).

use super::{ACCUM_FORMAT, OPAQUE_FORMAT, REVEAL_FORMAT};
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites,
};

/// Fragment entry point, color targets and depth writes of one pipeline.
pub struct Variant<'a> {
    pub label: &'a str,
    pub entry: &'a str,
    pub targets: &'a [Option<ColorTargetState>],
    pub depth_write: bool,
}

impl<'a> Variant<'a> {
    /// Pipeline drawing `fs_main` into egui's pass ([`egui_target`]).
    pub fn egui(
        label: &'a str,
        targets: &'a [Option<ColorTargetState>],
        depth_write: bool,
    ) -> Self {
        Self {
            label,
            entry: "fs_main",
            targets,
            depth_write,
        }
    }
}

/// egui's color target with straight-alpha blending, for [`Variant::egui`].
pub fn egui_target(format: wgpu::TextureFormat) -> [Option<ColorTargetState>; 1] {
    [Some(ColorTargetState {
        format,
        blend: Some(BlendState::ALPHA_BLENDING),
        write_mask: ColorWrites::ALL,
    })]
}

const ADD: BlendComponent = BlendComponent {
    src_factor: BlendFactor::One,
    dst_factor: BlendFactor::One,
    operation: BlendOperation::Add,
};

const REVEAL: BlendComponent = BlendComponent {
    src_factor: BlendFactor::Zero,
    dst_factor: BlendFactor::OneMinusSrc,
    operation: BlendOperation::Add,
};

/// Opaque pass: nearly opaque fragments, replacing color and writing depth.
pub const OPAQUE: Variant<'static> = Variant {
    label: "OIT Opaque Pipeline",
    entry: "fs_opaque",
    targets: &[Some(ColorTargetState {
        format: OPAQUE_FORMAT,
        blend: None,
        write_mask: ColorWrites::ALL,
    })],
    depth_write: true,
};

/// Translucent pass: additive color sum and multiplicative revealage.
pub const ACCUMULATE: Variant<'static> = Variant {
    label: "OIT Accumulate Pipeline",
    entry: "fs_oit",
    targets: &[
        Some(ColorTargetState {
            format: ACCUM_FORMAT,
            blend: Some(BlendState {
                color: ADD,
                alpha: ADD,
            }),
            write_mask: ColorWrites::ALL,
        }),
        Some(ColorTargetState {
            format: REVEAL_FORMAT,
            blend: Some(BlendState {
                color: REVEAL,
                alpha: REVEAL,
            }),
            write_mask: ColorWrites::ALL,
        }),
    ],
    depth_write: false,
};

/// Triangle-list pipeline of a 3D plot shader (`vs_main` reading `buffers`)
/// for one [`Variant`], depth-tested `LessEqual` against `Depth32Float`.
pub fn build_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    buffers: &[Option<wgpu::VertexBufferLayout<'_>>],
    cull_mode: Option<wgpu::Face>,
    variant: &Variant<'_>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(variant.label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(variant.entry),
            targets: variant.targets,
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode,
            front_face: wgpu::FrontFace::Ccw,
            ..Default::default()
        },
        depth_stencil: Some(crate::plots::common::default_depth_stencil_state(
            variant.depth_write,
            wgpu::CompareFunction::LessEqual,
        )),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
