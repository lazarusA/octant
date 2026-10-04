//! Empty-space skipping grid: for every 8³ brick of the volume, a superset of
//! the valid values the shader can read while sampling inside it (min, max)
//! and whether missing voxels are near. Rays cross bricks that cannot
//! contribute to the current mode in one jump.
//!
//! Built in two steps, both cheap enough to run on every upload: each brick's
//! own voxels in one linear pass (`cores`, kept on the CPU so partial uploads
//! only recompute their layers), then the union with the 26 neighboring bricks
//! (wrapping on every axis). Sampling reads at most two voxels past a brick
//! (trilinear filtering plus missing-voxel fills), well inside the neighbors,
//! and wrapping covers ring-buffer seams.

use super::encode::{self, Dims};
use std::ops::Range;
use std::sync::Mutex;

/// Brick edge in voxels; matches `BRICK` in `shaders/volume/skip.wgsl`.
pub const BRICK: usize = 8;

/// Stats of a brick without valid voxels: min above max.
const EMPTY: [f32; 4] = [f32::MAX, f32::MIN, 0.0, 0.0];

/// Brick grid extent for a volume of `dims`.
pub fn brick_dims(dims: Dims) -> Dims {
    Dims {
        w: dims.w.div_ceil(BRICK),
        h: dims.h.div_ceil(BRICK),
        d: dims.d.div_ceil(BRICK),
    }
}

/// Scalar a brick's range is taken over: the value, or a composite's
/// brightness (brightest channel, as its alpha), in [0, 1].
pub fn decode(v: f32, composite: bool) -> Option<f32> {
    if !encode::is_valid(v) {
        return None;
    }
    if !composite {
        return Some(v);
    }
    let packed = v as u32;
    let brightest = (packed & 0xFF)
        .max((packed >> 8) & 0xFF)
        .max((packed >> 16) & 0xFF);
    Some(brightest as f32 / 255.0)
}

fn merge(a: &mut [f32; 4], b: &[f32; 4]) {
    a[0] = a[0].min(b[0]);
    a[1] = a[1].max(b[1]);
    a[2] = a[2].max(b[2]);
}

/// Own-voxel stats of brick row `by` in brick layer `bz` (one entry per bx).
fn core_row(values: &[f32], dims: Dims, composite: bool, bz: usize, by: usize) -> Vec<[f32; 4]> {
    let mut row_stats = vec![EMPTY; dims.w.div_ceil(BRICK)];
    let planes = bz * BRICK..((bz + 1) * BRICK).min(dims.d);
    let rows = by * BRICK..((by + 1) * BRICK).min(dims.h);
    for z in planes {
        for y in rows.clone() {
            let start = (z * dims.h + y) * dims.w;
            let row = &values[start..start + dims.w];
            for (stats, segment) in row_stats.iter_mut().zip(row.chunks(BRICK)) {
                for &v in segment {
                    match decode(v, composite) {
                        Some(v) => {
                            stats[0] = stats[0].min(v);
                            stats[1] = stats[1].max(v);
                        }
                        None => stats[2] = 1.0,
                    }
                }
            }
        }
    }
    row_stats
}

/// Brick layer `bz` of the grid: each brick merged with its 26 neighbors.
fn dilate_layer(cores: &[[f32; 4]], grid: Dims, bz: usize) -> Vec<[f32; 4]> {
    let wrap = |i: usize, d: isize, n: usize| (i as isize + d).rem_euclid(n as isize) as usize;
    let mut out = Vec::with_capacity(grid.w * grid.h);
    for by in 0..grid.h {
        for bx in 0..grid.w {
            let mut stats = EMPTY;
            for dz in -1..=1 {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (x, y, z) = (
                            wrap(bx, dx, grid.w),
                            wrap(by, dy, grid.h),
                            wrap(bz, dz, grid.d),
                        );
                        merge(&mut stats, &cores[(z * grid.h + y) * grid.w + x]);
                    }
                }
            }
            out.push(stats);
        }
    }
    out
}

/// Brick layers holding planes `z`, then those layers and their neighbors
/// (wrapping): the cores to recompute and the grid layers to rewrite.
fn layers_for(z: &Range<usize>, layers: usize) -> (Range<usize>, Vec<usize>) {
    let cores = (z.start / BRICK).min(layers)..z.end.div_ceil(BRICK).min(layers);
    let mut grid: Vec<usize> = Vec::new();
    for c in cores.clone() {
        for d in [layers - 1, 0, 1] {
            let layer = (c + d) % layers;
            if !grid.contains(&layer) {
                grid.push(layer);
            }
        }
    }
    (cores, grid)
}

/// The grid texture (`Rgba32Float`, one texel per brick) and its CPU cores.
pub struct BrickGrid {
    grid: Dims,
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    cores: Mutex<Vec<[f32; 4]>>,
}

impl BrickGrid {
    pub fn new(texture: wgpu::Texture, dims: Dims) -> Self {
        let grid = brick_dims(dims);
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
            cores: Mutex::new(vec![EMPTY; grid.w * grid.h * grid.d]),
            grid,
        }
    }

    /// Recomputes the bricks affected by changed planes `z` of `values`.
    pub fn update(
        &self,
        queue: &wgpu::Queue,
        values: &[f32],
        dims: Dims,
        composite: bool,
        z: Range<usize>,
    ) {
        let grid = self.grid;
        if z.is_empty() || grid.d == 0 || values.len() < dims.w * dims.h * dims.d {
            return;
        }
        let (core_layers, grid_layers) = layers_for(&z, grid.d);
        let rows: Vec<(usize, usize)> = core_layers
            .flat_map(|bz| (0..grid.h).map(move |by| (bz, by)))
            .collect();
        let compute = |&(bz, by): &(usize, usize)| core_row(values, dims, composite, bz, by);
        #[cfg(not(target_arch = "wasm32"))]
        let computed: Vec<Vec<[f32; 4]>> = {
            use rayon::prelude::*;
            rows.par_iter().map(compute).collect()
        };
        #[cfg(target_arch = "wasm32")]
        let computed: Vec<Vec<[f32; 4]>> = rows.iter().map(compute).collect();

        let mut cores = self.cores.lock().unwrap_or_else(|p| p.into_inner());
        for (&(bz, by), row) in rows.iter().zip(computed) {
            let start = (bz * grid.h + by) * grid.w;
            cores[start..start + grid.w].copy_from_slice(&row);
        }
        for layer in grid_layers {
            let stats = dilate_layer(&cores, grid, layer);
            self.write_layer(queue, layer, &stats);
        }
    }

    fn write_layer(&self, queue: &wgpu::Queue, layer: usize, stats: &[[f32; 4]]) {
        let extent = wgpu::Extent3d {
            width: self.grid.w as u32,
            height: self.grid.h as u32,
            depth_or_array_layers: 1,
        };
        let origin = wgpu::Origin3d {
            x: 0,
            y: 0,
            z: layer as u32,
        };
        super::textures::write_region(
            queue,
            &self.texture,
            origin,
            extent,
            bytemuck::cast_slice(stats),
            self.grid.w as u32 * 16,
        );
    }

    /// Marks every brick as mattering, turning skipping off (tests compare
    /// against it).
    #[cfg(test)]
    pub fn disable(&self, queue: &wgpu::Queue) {
        let all = vec![[f32::MIN, f32::MAX, 1.0, 0.0]; self.grid.w * self.grid.h];
        for layer in 0..self.grid.d {
            self.write_layer(queue, layer, &all);
        }
    }
}
