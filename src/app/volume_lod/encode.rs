//! A brick's stored cube from the block it was read as: f16 bits of
//! `(value - offset) / scale` in storage order, NaN where missing.

use super::Lod;
use crate::app::detail::LevelRead;
use crate::data::OctantBlock;
use crate::data::multiscale::bricks::{BrickKey, STORED};
use crate::plots::volume::bricks::{AtlasTexel, BrickTextures};

/// f16 bits of `(v - offset) / scale`, NaN where `v` is missing; values
/// past f16's range saturate (an infinity would read as no value, and
/// filtering it into its neighbours would make holes).
pub(super) fn encode_value(v: f32, offset: f32, scale: f32) -> u16 {
    if v.is_finite() {
        let max = half::f16::MAX.to_f32();
        half::f16::from_f32(((v - offset) / scale).clamp(-max, max)).to_bits()
    } else {
        half::f16::NAN.to_bits()
    }
}

/// How a brick's f16 bits decode into data values, as the GPU reads them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Encoding {
    pub offset: f32,
    pub scale: f32,
    /// A byte atlas: the GPU reads each value rounded to its byte
    /// (`to_unorm8`), exactly the 8-bit value stored.
    pub byte: bool,
}

impl Encoding {
    /// The value f16 bits `h` stand for on the GPU (NaN where missing).
    pub fn decode(self, h: u16) -> f32 {
        let x = half::f16::from_bits(h).to_f32();
        match self.byte {
            true => (x * 255.0).round() * (self.scale / 255.0) + self.offset,
            false => x * self.scale + self.offset,
        }
    }
}

impl Lod {
    /// How this plot's bricks decode.
    pub(super) fn encoding(&self) -> Encoding {
        Encoding {
            offset: self.offset,
            scale: self.scale,
            byte: matches!(self.texel, AtlasTexel::Byte),
        }
    }
}

/// A brick's stored cube: f16 bits of `(value - offset) / scale` (NaN
/// where missing), or a composite's RGBA8 colors (transparent where missing).
pub(super) enum Cube {
    Values(Vec<u16>),
    Colors(Vec<[u8; 4]>),
}

/// Whether a scalar cube holds a missing voxel (stored as NaN); a
/// composite's missing voxels are transparent colors, never flagged.
#[cfg(test)]
pub(super) fn has_missing(cube: &Cube) -> bool {
    match cube {
        Cube::Values(values) => values.iter().any(|&h| half::f16::from_bits(h).is_nan()),
        Cube::Colors(_) => false,
    }
}

/// `key`'s stored cube from its block (oriented as plotted, put back in
/// storage order), edge cells repeated past the level's ends.
pub(super) fn brick_values(
    lod: &Lod,
    key: BrickKey,
    read: &LevelRead,
    block: &OctantBlock,
) -> Option<Cube> {
    let n = STORED as usize;
    let whole = [0..n, 0..n, 0..n];
    match lod.composite {
        Some(_) => lod
            .region_colors(key, &whole, block, read)
            .map(Cube::Colors),
        None => lod
            .region_values(key, &whole, block, Some(read))
            .map(Cube::Values),
    }
}

impl Lod {
    /// Uploads f16 `values` (x fastest) over the box `at`, `size` of `slot`'s
    /// stored cube, as bytes in a byte atlas.
    pub(super) fn upload_values(
        &self,
        (gpu, queue): (&BrickTextures, &wgpu::Queue),
        slot: [u32; 3],
        (at, size): ([u32; 3], [u32; 3]),
        values: &[u16],
    ) {
        match self.texel {
            AtlasTexel::Byte => gpu.upload_region(queue, slot, at, size, &to_unorm8(values)),
            _ => gpu.upload_region(queue, slot, at, size, values),
        }
    }
}

/// f16 `(v - offset) / 255` as the byte `round(x * 255)` (exact for 8-bit
/// integer data), 0 where missing.
pub(super) fn to_unorm8(values: &[u16]) -> Vec<u8> {
    values
        .iter()
        .map(|&h| {
            let x = half::f16::from_bits(h).to_f32();
            if x.is_finite() {
                (x * 255.0).round().clamp(0.0, 255.0) as u8
            } else {
                0
            }
        })
        .collect()
}
