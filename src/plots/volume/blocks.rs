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
    stats: Mutex<Stats>,
}

/// Min, max and missing flag per plane and block column (`planes`, so an
/// upload scans only its own planes), and per block (`own`, the reduction of
/// its planes), before taking neighbors.
struct Stats {
    planes: Vec<[f32; 3]>,
    own: Vec<[f32; 3]>,
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
        let columns = grid.w * grid.h;
        let blocks = Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
            grid,
            stats: Mutex::new(Stats {
                planes: vec![start; dims.d * columns],
                own: vec![start; grid.d * columns],
            }),
        };
        blocks.write_rows(queue, 0..grid.d);
        blocks
    }

    /// Opens every block, so rays skip nothing (tests compare against it).
    #[cfg(test)]
    pub fn open_all(&self, queue: &wgpu::Queue) {
        self.lock().own.fill(OPEN);
        self.write_rows(queue, 0..self.grid.d);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Stats> {
        self.stats.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Takes scalar planes `z` of the full volume `values` into the blocks:
    /// scans those planes only, then reduces the block rows holding them.
    pub fn update(&self, queue: &wgpu::Queue, values: &[f32], dims: Dims, z: Range<usize>) {
        let z = z.start..z.end.min(dims.d);
        if z.is_empty() || values.len() != dims.w * dims.h * dims.d {
            return;
        }
        let rows = z.start / BLOCK..z.end.div_ceil(BLOCK).min(self.grid.d);
        {
            let mut stats = self.lock();
            let Stats { planes, own } = &mut *stats;
            let (grid, columns) = (self.grid, self.grid.w * self.grid.h);
            let changed = &mut planes[z.start * columns..z.end * columns];
            let scan = |(i, plane): (usize, &mut [[f32; 3]])| {
                plane_stats(values, dims, grid, z.start + i, plane);
            };
            #[cfg(not(target_arch = "wasm32"))]
            {
                use rayon::prelude::*;
                changed.par_chunks_mut(columns).enumerate().for_each(scan);
            }
            #[cfg(target_arch = "wasm32")]
            changed.chunks_mut(columns).enumerate().for_each(scan);
            for bz in rows.clone() {
                own_row(planes, dims, self.grid, bz, own);
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
        let stats = self.lock();
        let texels: Vec<[f32; 4]> = rows
            .clone()
            .flat_map(|bz| (0..self.grid.h).map(move |by| (bz, by)))
            .flat_map(|(bz, by)| (0..self.grid.w).map(move |bx| [bx, by, bz]))
            .map(|b| widened(&stats.own, self.grid, b))
            .collect();
        drop(stats);
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

/// Recomputes plane `z`'s block columns (`plane`) from its voxels.
fn plane_stats(values: &[f32], dims: Dims, grid: Dims, z: usize, plane: &mut [[f32; 3]]) {
    plane.fill(EMPTY);
    for y in 0..dims.h {
        let line = &values[(z * dims.h + y) * dims.w..][..dims.w];
        let columns = &mut plane[(y / BLOCK) * grid.w..][..grid.w];
        for (b, chunk) in columns.iter_mut().zip(line.chunks(BLOCK)) {
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

/// Recomputes block row `bz` (`own`) from the planes it holds.
fn own_row(planes: &[[f32; 3]], dims: Dims, grid: Dims, bz: usize, own: &mut [[f32; 3]]) {
    let columns = grid.w * grid.h;
    let row = &mut own[bz * columns..(bz + 1) * columns];
    row.fill(EMPTY);
    for z in bz * BLOCK..((bz + 1) * BLOCK).min(dims.d) {
        for (b, p) in row.iter_mut().zip(&planes[z * columns..(z + 1) * columns]) {
            *b = [b[0].min(p[0]), b[1].max(p[1]), b[2].max(p[2])];
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
#[path = "blocks_tests.rs"]
mod tests;
