//! A 2D matrix thinned for drawing as a mesh: a surface or sphere costs
//! time per cell (one instance each), not per screen pixel, so grids with
//! more cells than the canvas has pixels draw one cell in `stride` along
//! each axis. Nearest sampling, so packed composite colors and label ids
//! stay exact.

use crate::data::coordinates::CoordinateGrid;
use crate::data::render::MatrixData;

/// The stride along each axis that leaves at most `budget` of `cells`
/// cells (1 when they fit).
pub fn stride_for(cells: usize, budget: usize) -> usize {
    let ratio = cells.div_ceil(budget.max(1));
    let mut k = (ratio as f64).sqrt().ceil() as usize;
    // `sqrt` of a large ratio can round down by one.
    while k.saturating_mul(k) < ratio {
        k += 1;
    }
    k.max(1)
}

/// The full-resolution index of thinned index `i` along an axis of `n`
/// cells: the middle cell of its `stride` cells.
pub fn sample_index(i: usize, stride: usize, n: usize) -> usize {
    (i * stride + stride / 2).min(n.saturating_sub(1))
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
        let at = |values: &[f32]| thin(values, self.width, &xs, &ys);
        let pick = |values: &[f32], idx: &[usize]| -> std::sync::Arc<[f32]> {
            idx.iter().map(|&i| values.get(i).copied().unwrap_or(f32::NAN)).collect()
        };
        let grid = match &self.grid {
            CoordinateGrid::Irregular1D {
                coords_x,
                coords_y,
                lon_bounds,
                lat_bounds,
            } => CoordinateGrid::Irregular1D {
                coords_x: pick(coords_x, &xs),
                coords_y: pick(coords_y, &ys),
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
        };
        let mut out = MatrixData::new_with_grid(
            w,
            h,
            at(&self.values),
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
