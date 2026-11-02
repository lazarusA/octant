//! A 2D matrix thinned for drawing as a mesh: a surface or sphere costs
//! time per cell (one instance each), not per screen pixel, so grids with
//! more cells than the canvas has pixels draw one cell in `stride` along
//! each axis, spread evenly over it as the full grid's cells are. Nearest
//! sampling, so packed composite colors and label ids stay exact.

use crate::data::coordinates::CoordinateGrid;
use crate::data::render::MatrixData;

/// The stride along each axis that leaves at most `budget` cells of a
/// `width` x `height` grid (1 when they fit), counting the short last
/// stride of each axis as `decimated` keeps it.
pub fn stride_for(width: usize, height: usize, budget: usize) -> usize {
    let budget = budget.max(1);
    let kept = |k: usize| width.div_ceil(k).saturating_mul(height.div_ceil(k));
    let ratio = width.saturating_mul(height).div_ceil(budget);
    let mut k = ((ratio as f64).sqrt() as usize).max(1);
    // Narrow grids thin along one axis only, so the square root falls short.
    while kept(k) > budget {
        k += 1;
    }
    k
}

/// The full-resolution index of thinned index `i` along an axis of `n`
/// cells thinned by `stride`: the cell under the middle of the span it is
/// drawn over, as the `n.div_ceil(stride)` drawn cells spread evenly over
/// the axis (`floor((2i + 1) n / 2w)`; `sample_index` in
/// `shaders/channel_bake.wgsl`).
pub fn sample_index(i: usize, stride: usize, n: usize) -> usize {
    let w = n.div_ceil(stride.max(1)).max(1) as u64;
    let x = (2 * i as u64 + 1) * n as u64 / (2 * w);
    (x as usize).min(n.saturating_sub(1))
}

/// The thinned index drawn over full-resolution index `x` of an axis of
/// `n` cells thinned by `stride`: the inverse of [`sample_index`].
pub fn drawn_index(x: usize, stride: usize, n: usize) -> usize {
    let w = n.div_ceil(stride.max(1)).max(1);
    let i = x as u64 * w as u64 / n.max(1) as u64;
    (i as usize).min(w - 1)
}

impl MatrixData {
    /// This matrix with one cell in `stride` along each axis (sampled at
    /// `sample_index`), its coordinates thinned alike; `None` when `stride`
    /// is 1 or the grid is HEALPix, whose cells are not laid out in rows.
    pub fn decimated(&self, stride: usize) -> Option<MatrixData> {
        if stride < 2 || self.grid.is_healpix() || self.width == 0 || self.height == 0 {
            return None;
        }
        let (w, h) = (self.width.div_ceil(stride), self.height.div_ceil(stride));
        let xs: Vec<usize> = (0..w)
            .map(|x| sample_index(x, stride, self.width))
            .collect();
        let ys: Vec<usize> = (0..h)
            .map(|y| sample_index(y, stride, self.height))
            .collect();
        let grid = thin_grid(&self.grid, self.width, &xs, &ys);
        let mut out = MatrixData::new_with_grid(
            w,
            h,
            thin(&self.values, self.width, &xs, &ys),
            self.min_val,
            self.max_val,
            self.dataset_name.clone(),
            self.max_timesteps,
            grid,
        );
        out.unique_values.clone_from(&self.unique_values);
        Some(out)
    }
}

/// `grid` with its per-cell coordinates thinned to the columns `xs` and
/// rows `ys` of a `width`-wide grid.
fn thin_grid(grid: &CoordinateGrid, width: usize, xs: &[usize], ys: &[usize]) -> CoordinateGrid {
    let at = |values: &[f32]| thin(values, width, xs, ys);
    let pick = |values: &[f32], idx: &[usize]| -> std::sync::Arc<[f32]> {
        idx.iter()
            .map(|&i| values.get(i).copied().unwrap_or(f32::NAN))
            .collect()
    };
    match grid {
        CoordinateGrid::Irregular1D {
            coords_x,
            coords_y,
            lon_bounds,
            lat_bounds,
        } => CoordinateGrid::Irregular1D {
            coords_x: pick(coords_x, xs),
            coords_y: pick(coords_y, ys),
            lon_bounds: *lon_bounds,
            lat_bounds: *lat_bounds,
        },
        CoordinateGrid::Curvilinear2D {
            lons,
            lats,
            lon_bounds,
            lat_bounds,
        } => CoordinateGrid::Curvilinear2D {
            lons: at(lons).into(),
            lats: at(lats).into(),
            lon_bounds: *lon_bounds,
            lat_bounds: *lat_bounds,
        },
        regular => regular.clone(),
    }
}

/// The cells `(x, y)` of `xs` x `ys` of a `width`-wide grid of `values`,
/// row by row; NaN past them.
fn thin(values: &[f32], width: usize, xs: &[usize], ys: &[usize]) -> Vec<f32> {
    let mut out = Vec::with_capacity(xs.len() * ys.len());
    for &y in ys {
        for &x in xs {
            out.push(values.get(y * width + x).copied().unwrap_or(f32::NAN));
        }
    }
    out
}

impl crate::data::VolumeData {
    /// Planes `z` with one voxel in `stride` along x and y (`sample_index`),
    /// plane after plane: what a thinned point cloud draws of them.
    pub fn thinned_planes(&self, stride: usize, z: std::ops::Range<usize>) -> Vec<f32> {
        let mut out = Vec::new();
        self.write_thinned_planes(stride, z, &mut out);
        out
    }

    /// `thinned_planes` into `out` (cleared first), reusing its memory.
    pub fn write_thinned_planes(
        &self,
        stride: usize,
        z: std::ops::Range<usize>,
        out: &mut Vec<f32>,
    ) {
        let (w, h) = (self.width.div_ceil(stride), self.height.div_ceil(stride));
        let plane = self.width * self.height;
        out.clear();
        out.reserve(z.len() * w * h);
        for zi in z {
            for y in 0..h {
                let row = zi * plane + sample_index(y, stride, self.height) * self.width;
                out.extend((0..w).map(|x| {
                    let at = row + sample_index(x, stride, self.width);
                    self.values.get(at).copied().unwrap_or(f32::NAN)
                }));
            }
        }
    }
}
