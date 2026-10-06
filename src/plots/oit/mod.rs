//! Weighted blended order-independent transparency (McGuire & Bavoil 2013)
//! for 3D meshes and point clouds with translucent colors.
//!
//! The callback's `prepare` draws the plot offscreen twice ([`OitState::render`]):
//! nearly opaque fragments into an opaque color and depth target (`fs_opaque`),
//! then translucent fragments, depth-tested against them without writing
//! depth, into a weighted color sum and a revealage product (`fs_oit`, see
//! `shaders/common/oit.wgsl`). Sums and products do not depend on draw order,
//! so no sorting is needed and no translucent layer hides another. `paint`
//! resolves the frame over the plot viewport ([`OitState::paint`]).
//!
//! Needs blendable float targets and per-target blending ([`supported`]);
//! without them plots fall back to [`Transparency::NoDepthWrite`].

mod frame;
mod state;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

pub use state::OitState;

use std::sync::OnceLock;
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites,
};

pub const OPAQUE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const ACCUM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const REVEAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;
/// Same format as egui's depth buffer, so one depth state serves every pass.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// How a 3D mesh or point cloud draws its colors.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Transparency {
    /// Depth writes on: the nearest fragment hides those behind it.
    #[default]
    Off,
    /// Depth writes off, blended in draw order (fallback without OIT support).
    NoDepthWrite,
    /// Weighted blended order-independent transparency.
    Oit,
}

/// Fragment entry point, color targets and depth writes of one pipeline.
pub struct Variant<'a> {
    pub label: &'a str,
    pub entry: &'a str,
    pub targets: &'a [Option<ColorTargetState>],
    pub depth_write: bool,
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

/// Frame size in pixels of a plot viewport, rounded as
/// `setup_viewport_and_scissor` rounds the viewport.
pub fn frame_size(rect: &egui::Rect, pixels_per_point: f32) -> [u32; 2] {
    let pixels = rect.size() * pixels_per_point;
    [pixels.x, pixels.y].map(|p| p.round().max(1.0) as u32)
}

/// Whether the adapter can render OIT frames. Checked once per process.
pub fn supported(adapter: &wgpu::Adapter) -> bool {
    static SUPPORTED: OnceLock<bool> = OnceLock::new();
    *SUPPORTED.get_or_init(|| {
        let blendable = |format| {
            let features = adapter.get_texture_format_features(format);
            features
                .allowed_usages
                .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
                && features
                    .flags
                    .contains(wgpu::TextureFormatFeatureFlags::BLENDABLE)
        };
        adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::INDEPENDENT_BLEND)
            && blendable(ACCUM_FORMAT)
            && blendable(REVEAL_FORMAT)
    })
}
