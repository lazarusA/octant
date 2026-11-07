//! The surface under the slicer: where fractions of its cells lie on it
//! (`ui::slice_gizmo` places its box through this), its drawn heights, and
//! which cells the box keeps.

use super::raycast_surface::SurfaceField;
use super::smooth::value_at;
use crate::plots::slice::has_relief;

impl SurfaceField<'_> {
    /// Surface coordinate along x or y of `f`, a fraction of the grid's
    /// cells (cell `k` of `n` spans `[k/n, (k+1)/n)`): within cell `k`
    /// between its edges, midway between neighbouring centers (the outer
    /// ones `sample`'s), so irregular grids place it where their cells are.
    pub fn uv_of_fraction(&self, f: f32, along_x: bool) -> f32 {
        let n = if along_x {
            self.matrix.width
        } else {
            self.matrix.height
        };
        if n < 2 || self.matrix.grid.is_healpix() {
            return f;
        }
        let x = f.clamp(0.0, 1.0) * n as f32;
        let k = (x as usize).min(n - 1);
        let c = |j| self.center(j, along_x);
        let left = if k == 0 {
            self.sample(0, along_x)
        } else {
            0.5 * (c(k - 1) + c(k))
        };
        let right = if k + 1 == n {
            self.sample(n - 1, along_x)
        } else {
            0.5 * (c(k) + c(k + 1))
        };
        left + (right - left) * (x - k as f32)
    }

    /// World heights the drawn surface spans, the zero plane included.
    pub fn height_span(&self) -> (f32, f32) {
        let (low, high) = self.scale.bounds();
        let k = crate::utils::height::height_factor(false, self.mode) * self.disp;
        (k * low.min(0.0), k * high.max(0.0))
    }

    /// Whether the slicer keeps cell `(px, py)`: by its center, and by its
    /// value's height, as blocks are cut (smooth and step surfaces, cut
    /// between their vertices, closely).
    pub(super) fn kept(&self, cell: (usize, usize)) -> bool {
        let grid = (self.matrix.width, self.matrix.height);
        let h = self
            .scale
            .normalized(value_at(self.layer, self.matrix, cell.0, cell.1));
        let relief = has_relief(&self.scale, false, self.mode, self.disp);
        self.slice.keeps_cell(cell, grid) && self.slice.keeps_height(h, &self.scale, relief)
    }
}
