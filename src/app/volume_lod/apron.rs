//! Aprons from neighbours: bricks are read without their apron, so a read stays inside the
//! store's chunks wherever chunk edges fall on brick edges (64, 128, ...
//! cells: one chunk instead of eight). A placed brick's apron is filled
//! from its resident same-level neighbours whose blocks are still cached,
//! and an arriving brick fills its resident neighbours' aprons facing it,
//! so trilinear filtering across bricks reads the same values as before;
//! where no neighbour is resident yet the brick's edge repeats. A
//! composite's apron is blended from the neighbour's channels
//! (`apron_color.rs`).

use std::ops::Range;

use super::Lod;
use super::encode::Cube;
use super::encode::encode_value;
use crate::app::detail::LevelRead;
use crate::data::OctantBlock;
use crate::data::blocks::BlockCache;
use crate::data::multiscale::bricks::{BrickKey, STORED};
use crate::plots::volume::bricks::BrickTextures;

/// The 26 neighbour offsets.
fn offsets() -> impl Iterator<Item = [i32; 3]> {
    (0..27)
        .map(|n| [n % 3 - 1, n / 3 % 3 - 1, n / 9 - 1])
        .filter(|d| *d != [0; 3])
}

/// Stored indices along one axis of the part of a cube facing offset `d`:
/// its first cell, its interior, its last cell.
fn span(d: i32) -> Range<usize> {
    let last = STORED as usize - 1;
    match d {
        -1 => 0..1,
        0 => 1..last,
        _ => last..last + 1,
    }
}

impl Lod {
    /// The stored box of `key`'s cube facing offset `d`: along an axis
    /// where `key` sits at its level's start or end, also the apron there
    /// (no brick lies beyond, so the facing brick's edge repeats into it).
    fn facing(&self, key: BrickKey, d: [i32; 3]) -> [Range<usize>; 3] {
        let counts = self.bricks().counts(usize::from(key.level));
        std::array::from_fn(|i| {
            let r = span(d[i]);
            if d[i] != 0 {
                return r;
            }
            let start = if key.b[i] == 0 { 0 } else { r.start };
            let end = if key.b[i] + 1 >= counts[i] {
                r.end + 1
            } else {
                r.end
            };
            start..end
        })
    }

    /// The brick at offset `d` from `key` on its level, if the level has it.
    fn neighbour(&self, key: BrickKey, d: [i32; 3]) -> Option<BrickKey> {
        let counts = self.bricks().counts(usize::from(key.level));
        let mut b = [0; 3];
        for i in 0..3 {
            let v = i64::from(key.b[i]) + i64::from(d[i]);
            b[i] = u32::try_from(v).ok().filter(|&v| v < counts[i])?;
        }
        Some(BrickKey {
            level: key.level,
            b,
        })
    }

    /// The f16 values of `block` over the stored box `region` of brick
    /// `key`'s cube (x fastest), reading directly from `block` with column
    /// offsets precomputed.
    pub(super) fn region_values(
        &self,
        key: BrickKey,
        region: &[Range<usize>; 3],
        block: &OctantBlock,
        read: Option<&LevelRead>,
    ) -> Option<Vec<u16>> {
        self.bricks().window(key)?;
        let axes = self.block_axes_for(block, read)?;
        let len = region[0].len() * region[1].len() * region[2].len();
        let mut out = Vec::with_capacity(len);
        let cols: Vec<usize> = region[0]
            .clone()
            .map(|x| axes.stored_offset(block, key, 0, x))
            .collect();
        let (offset, scale) = (self.offset, self.scale);
        for z in region[2].clone() {
            let oz = axes.base + axes.stored_offset(block, key, 2, z);
            for y in region[1].clone() {
                let row = oz + axes.stored_offset(block, key, 1, y);
                out.extend(cols.iter().map(|&ox| {
                    let v = block.values.get(row + ox).copied().unwrap_or(f32::NAN);
                    encode_value(v, offset, scale)
                }));
            }
        }
        Some(out)
    }

    /// Fills `cube` (`key`'s stored cube) where its apron faces a resident
    /// neighbour whose block is cached; a composite's from `read`'s
    /// dimensions (`apron_color.rs`).
    pub(super) fn stitch_own(
        &self,
        key: BrickKey,
        cube: &mut Cube,
        cache: &BlockCache,
        read: Option<&LevelRead>,
    ) {
        for d in offsets() {
            let Some(other) = self.neighbour(key, d) else {
                continue;
            };
            let block = self
                .residency
                .get(&other)
                .and_then(|p| cache.peek(&p.block));
            let Some(block) = block else {
                continue;
            };
            let region = self.facing(key, d);
            match cube {
                Cube::Values(values) => {
                    if let Some(found) = self.region_values(key, &region, block, read) {
                        fill(values, &region, found);
                    }
                }
                Cube::Colors(colors) => {
                    let found = read.and_then(|r| self.region_colors(key, &region, block, r));
                    if let Some(found) = found {
                        fill(colors, &region, found);
                    }
                }
            }
        }
    }

    /// Uploads, into each resident neighbour of `key`, its apron facing
    /// `key` from `key`'s `block` (read as `read`), widening the
    /// neighbour's ranges to cover it.
    pub(super) fn stitch_neighbours(
        &mut self,
        key: BrickKey,
        (block, read): (&OctantBlock, &LevelRead),
        gpu: &BrickTextures,
        queue: &wgpu::Queue,
    ) {
        for d in offsets() {
            let Some(other) = self.neighbour(key, d) else {
                continue;
            };
            let Some(slot) = self.residency.slot_of(&other) else {
                continue;
            };
            // Seen from `other`, `key` lies at offset `-d`.
            let region = self.facing(other, d.map(|v| -v));
            let at = region.clone().map(|r| r.start as u32);
            let size = region.clone().map(|r| r.len() as u32);
            if self.composite.is_some() {
                if let Some(colors) = self.region_colors(other, &region, block, read) {
                    gpu.upload_region(queue, slot, at, size, &colors);
                }
            } else if let Some(values) = self.region_values(other, &region, block, Some(read)) {
                self.upload_values((gpu, queue), slot, (at, size), &values);
                self.widen_apron(other, &region, &values, (gpu, queue, slot));
            }
        }
    }
}

/// Writes `src` (x fastest) over the box `region` of a stored cube `dst`.
fn fill<T: Copy>(dst: &mut [T], region: &[Range<usize>; 3], src: Vec<T>) {
    let n = STORED as usize;
    let mut src = src.into_iter();
    for z in region[2].clone() {
        for y in region[1].clone() {
            for x in region[0].clone() {
                if let (Some(slot), Some(v)) = (dst.get_mut((z * n + y) * n + x), src.next()) {
                    *slot = v;
                }
            }
        }
    }
}
