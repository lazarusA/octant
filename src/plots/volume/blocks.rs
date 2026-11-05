//! Empty-space skipping: per block of `BLOCK`^3 stored voxels, the smallest
//! and largest valid value and whether any voxel is missing, over the block
//! and its 26 neighbors (so trilinear taps, missing-voxel fills and gradients
//! near a block stay within what it reports). Rays jump over blocks that
//! cannot show anything (`blocks.wgsl`). One `Rgba32Float` texel per block:
//! min, max, missing (0 or 1), unused.

use super::encode::{Dims, VolumeEncoding, is_valid};
use super::texture_writes::{create_texture, write_region};
use std::ops::Range;
use std::sync::Mutex;

/// Block edge in voxels (`BLOCK_SHIFT` in `blocks.wgsl`).
pub const BLOCK: usize = 8;

/// A block without valid values: no range intersects it.
const EMPTY: [f32; 3] = [f32::MAX, -f32::MAX, 0.0];
/// A block that may hold anything (composites, whose colors the CPU does
/// not know).
const OPEN: [f32; 3] = [-f32::MAX, f32::MAX, 1.0];

pub struct Blocks {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    grid: Dims,
    /// Each block's own min, max and missing flag, before taking neighbors.
    own: Mutex<Vec<[f32; 3]>>,
}

/// Blocks covering `dims` voxels.
pub fn grid_of(dims: Dims) -> Dims {
    Dims {
        w: dims.w.div_ceil(BLOCK),
        h: dims.h.div_ceil(BLOCK),
        d: dims.d.div_ceil(BLOCK),
    }
}

impl Blocks {
    /// Blocks of a volume nothing was uploaded to yet: every voxel missing,
    /// or open for composites.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        dims: Dims,
        encoding: VolumeEncoding,
    ) -> Self {
        let grid = grid_of(dims);
        let texture = create_texture(
            device,
            "Volume Blocks",
            grid,
            wgpu::TextureFormat::Rgba32Float,
        );
        let start = match encoding {
            VolumeEncoding::Scalar => [EMPTY[0], EMPTY[1], 1.0],
            VolumeEncoding::PackedRgb => OPEN,
        };
        let blocks = Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
            grid,
            own: Mutex::new(vec![start; grid.w * grid.h * grid.d]),
        };
        blocks.write_rows(queue, 0..grid.d);
        blocks
    }

    /// Opens every block, so rays skip nothing (tests compare against it).
    #[cfg(test)]
    pub fn open_all(&self, queue: &wgpu::Queue) {
        self.own
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .fill(OPEN);
        self.write_rows(queue, 0..self.grid.d);
    }

    /// Takes scalar planes `z` of the full volume `values` into the blocks.
    pub fn update(&self, queue: &wgpu::Queue, values: &[f32], dims: Dims, z: Range<usize>) {
        if z.is_empty() || values.len() != dims.w * dims.h * dims.d {
            return;
        }
        let rows = z.start / BLOCK..z.end.div_ceil(BLOCK).min(self.grid.d);
        {
            let mut own = self.own.lock().unwrap_or_else(|p| p.into_inner());
            for bz in rows.clone() {
                own_row(values, dims, self.grid, bz, &mut own);
            }
        }
        // Neighbors take each changed row too.
        let start = rows.start.saturating_sub(1);
        let end = (rows.end + 1).min(self.grid.d);
        self.write_rows(queue, start..end);
    }

    /// Writes block rows `rows`, each block widened by its neighbors.
    fn write_rows(&self, queue: &wgpu::Queue, rows: Range<usize>) {
        if rows.is_empty() {
            return;
        }
        let own = self.own.lock().unwrap_or_else(|p| p.into_inner());
        let texels: Vec<[f32; 4]> = rows
            .clone()
            .flat_map(|bz| (0..self.grid.h).map(move |by| (bz, by)))
            .flat_map(|(bz, by)| (0..self.grid.w).map(move |bx| [bx, by, bz]))
            .map(|b| widened(&own, self.grid, b))
            .collect();
        drop(own);
        let origin = wgpu::Origin3d {
            x: 0,
            y: 0,
            z: rows.start as u32,
        };
        let extent = wgpu::Extent3d {
            width: self.grid.w as u32,
            height: self.grid.h as u32,
            depth_or_array_layers: rows.len() as u32,
        };
        let bytes = bytemuck::cast_slice(&texels);
        write_region(
            queue,
            &self.texture,
            origin,
            extent,
            bytes,
            self.grid.w as u32 * 16,
        );
    }
}

/// Recomputes every block of row `bz` from the voxels it holds.
fn own_row(values: &[f32], dims: Dims, grid: Dims, bz: usize, own: &mut [[f32; 3]]) {
    let row = bz * grid.h * grid.w;
    own[row..row + grid.h * grid.w].fill(EMPTY);
    for z in bz * BLOCK..((bz + 1) * BLOCK).min(dims.d) {
        for y in 0..dims.h {
            let line = &values[(z * dims.h + y) * dims.w..][..dims.w];
            let blocks = &mut own[row + (y / BLOCK) * grid.w..][..grid.w];
            for (bx, chunk) in line.chunks(BLOCK).enumerate() {
                let b = &mut blocks[bx];
                for &v in chunk {
                    if is_valid(v) {
                        b[0] = b[0].min(v);
                        b[1] = b[1].max(v);
                    } else {
                        b[2] = 1.0;
                    }
                }
            }
        }
    }
}

/// Block `b` with its 26 neighbors: the smallest min, largest max, any missing.
fn widened(own: &[[f32; 3]], grid: Dims, b: [usize; 3]) -> [f32; 4] {
    let near = |c: usize, n: usize| c.saturating_sub(1)..(c + 2).min(n);
    let mut out = [EMPTY[0], EMPTY[1], 0.0, 0.0];
    for z in near(b[2], grid.d) {
        for y in near(b[1], grid.h) {
            for x in near(b[0], grid.w) {
                let o = own[(z * grid.h + y) * grid.w + x];
                out[0] = out[0].min(o[0]);
                out[1] = out[1].max(o[1]);
                out[2] = out[2].max(o[2]);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIMS: Dims = Dims { w: 20, h: 9, d: 17 };

    fn own_of(values: &[f32]) -> Vec<[f32; 3]> {
        let grid = grid_of(DIMS);
        let mut own = vec![EMPTY; grid.w * grid.h * grid.d];
        for bz in 0..grid.d {
            own_row(values, DIMS, grid, bz, &mut own);
        }
        own
    }

    #[test]
    fn blocks_hold_their_voxels_range() {
        let n = DIMS.w * DIMS.h * DIMS.d;
        let mut values = vec![0.0f32; n];
        // Voxel (17, 8, 16): block (2, 1, 2), the partial last one on each axis.
        values[(16 * DIMS.h + 8) * DIMS.w + 17] = 5.0;
        values[0] = f32::NAN;
        let grid = grid_of(DIMS);
        assert_eq!((grid.w, grid.h, grid.d), (3, 2, 3));
        let own = own_of(&values);
        let at = |b: [usize; 3]| own[(b[2] * grid.h + b[1]) * grid.w + b[0]];
        assert_eq!(at([2, 1, 2]), [0.0, 5.0, 0.0]);
        assert_eq!(at([0, 0, 0]), [0.0, 0.0, 1.0]);
        // Neighbors see the 5 and the missing voxel; far blocks do not.
        assert_eq!(widened(&own, grid, [1, 0, 1])[..3], [0.0, 5.0, 1.0]);
        assert_eq!(widened(&own, grid, [0, 0, 2])[1], 0.0);
        assert_eq!(widened(&own, grid, [0, 1, 1])[2], 1.0);
        assert_eq!(widened(&own, grid, [2, 1, 2])[2], 0.0);
    }

    #[test]
    fn a_block_without_values_is_empty() {
        let own = own_of(&vec![f32::NAN; DIMS.w * DIMS.h * DIMS.d]);
        assert!(own.iter().all(|b| b[0] > b[1] && b[2] == 1.0));
    }
}
