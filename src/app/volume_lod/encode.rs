//! A brick's stored cube from the block it was read as: f16 bits of
//! `(value - offset) / scale` in storage order, NaN where missing.

use super::Lod;
use crate::app::OctantApp;
use crate::app::block_loading::block_axes;
use crate::app::detail::LevelRead;
use crate::data::OctantBlock;
use crate::data::multiscale::bricks::{APRON, BRICK, BrickKey, STORED};
use crate::plots::volume::bricks::{AtlasTexel, BrickTextures};
use crate::plots::volume::encode::packed_rgba;

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

/// The range of the valid values a stored cube encodes (apron included:
/// rays near its faces read it).
pub(super) fn value_range(values: &[u16], encoding: Encoding) -> (f32, f32) {
    values
        .iter()
        .map(|&h| encoding.decode(h))
        .filter(|v| v.is_finite())
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        })
}

/// A brick's stored cube: f16 bits of `(value - offset) / scale` (NaN
/// where missing), or a composite's RGBA8 colors (transparent where missing).
pub(super) enum Cube {
    Values(Vec<u16>),
    Colors(Vec<[u8; 4]>),
}

/// `key`'s stored cube from its block (oriented as plotted, put back in
/// storage order), edge cells repeated past the level's ends, and whether
/// any voxel is missing.
pub(super) fn brick_values(
    lod: &Lod,
    key: BrickKey,
    read: &LevelRead,
    block: &OctantBlock,
) -> Option<(Cube, bool)> {
    let window = lod.bricks().window(key)?;
    let (dims, config) = (&read.dims, &read.sel.dim_config);
    if block.dimension_names != *dims {
        return None;
    }
    let (bx, by, bz) =
        OctantApp::resolve_spatial_axes(block.rank(), &block.dimension_names, dims, config);
    let fixed = block_axes::fixed_indices(block, dims, read.sel.animated_dim(), config, read.step);
    let range =
        |d: usize| block.oriented_range(d, block_axes::dim_bounds(block, dims, d, config).1);
    let ranges = (range(bx), range(by), range(bz));
    let volume = match &lod.composite {
        Some(c) => c.slice(block, (bx, by, bz), ranges, &fixed)?,
        None => {
            let (x, y, z) = ranges;
            block.volume_with_ranges(bx, by, bz, x, y, z, &fixed, "", false)?
        }
    };
    let n = [volume.width, volume.height, volume.depth];
    let axes = [bx, by, bz];
    // The volume's (oriented) index of the brick's stored cell `s` along
    // axis `i`: the volume spans the whole block, so the block's.
    let read_cell = |i: usize, s: usize| {
        let cell = (u64::from(key.b[i]) * BRICK + s as u64).saturating_sub(APRON);
        let local = cell.clamp(window[i].start, window[i].end.saturating_sub(1)) - window[i].start;
        let local = (local as usize).min(n[i].saturating_sub(1));
        block.oriented_index(axes[i], local)
    };
    let (offset, scale) = (lod.offset, lod.scale);
    Some(match lod.composite {
        Some(_) => {
            let (cube, missing) = encode_cube(&volume.values, n, read_cell, packed_rgba);
            (Cube::Colors(cube), missing)
        }
        None => {
            let encode = |v| encode_value(v, offset, scale);
            let (cube, missing) = encode_cube(&volume.values, n, read_cell, encode);
            (Cube::Values(cube), missing)
        }
    })
}

/// The stored cube read from `values` (`n` x, y, z, x fastest) at
/// `read_cell(axis, stored index)`, each value `encode`d, and whether any
/// is missing.
fn encode_cube<T>(
    values: &[f32],
    n: [usize; 3],
    read_cell: impl Fn(usize, usize) -> usize,
    encode: impl Fn(f32) -> T,
) -> (Vec<T>, bool) {
    let stored = STORED as usize;
    let columns: Vec<usize> = (0..stored).map(|s| read_cell(0, s)).collect();
    let mut out = Vec::with_capacity(stored * stored * stored);
    let mut missing = false;
    for sz in 0..stored {
        let z = read_cell(2, sz);
        for sy in 0..stored {
            let row = (z * n[1] + read_cell(1, sy)) * n[0];
            out.extend(columns.iter().map(|&x| {
                let v = values.get(row + x).copied().unwrap_or(f32::NAN);
                missing |= !v.is_finite();
                encode(v)
            }));
        }
    }
    (out, missing)
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
