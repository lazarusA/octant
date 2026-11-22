//! A bricked volume's GPU types (`bricks.rs`): what an atlas texel holds,
//! the uniforms placing the window and the levels, their limits.

use bytemuck::{Pod, Zeroable};

/// Sub-blocks (8^3 cells) per brick, whose ranges rays skip by
/// (`brick_subs` in `sample_bricked.wgsl`).
pub const SUBS_PER_BRICK: u64 = 512;

/// Levels the shader places (`Bricked::levels`).
pub const MAX_LEVELS: usize = 16;

/// What an atlas texel holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtlasTexel {
    /// f16 of `(value - offset) / scale`, NaN where missing (`R16Float`).
    Half,
    /// 8-bit integer data, exact: `(value - offset) / 255` (`R8Unorm`).
    Byte,
    /// A composite's colors (`Rgba8Unorm`).
    Color,
}

impl AtlasTexel {
    /// The atlas's texture format and bytes per texel.
    pub fn format(self) -> (wgpu::TextureFormat, u32) {
        match self {
            Self::Half => (wgpu::TextureFormat::R16Float, 2),
            Self::Byte => (wgpu::TextureFormat::R8Unorm, 1),
            Self::Color => (wgpu::TextureFormat::Rgba8Unorm, 4),
        }
    }
}

/// Mirrors WGSL `BrickLevel` in `shaders/volume/sample_bricked.wgsl`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct BrickLevelGpu {
    pub factor: [f32; 4],
    pub offset: [f32; 4],
    pub count: [u32; 4],
}

/// Mirrors WGSL `Bricked` in `shaders/volume/sample_bricked.wgsl`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BrickUniforms {
    pub lo: [f32; 4],
    pub span: [f32; 4],
    pub flip: [u32; 4],
    pub grid: [u32; 4],
    /// Stored value = atlas value * `value[1]` + `value[0]`.
    pub value: [f32; 4],
    /// Atlas slots along x, y, z.
    pub slots: [u32; 4],
    pub levels: [BrickLevelGpu; MAX_LEVELS],
}
