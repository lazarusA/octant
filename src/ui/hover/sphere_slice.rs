//! The globe under the slicer: which cells its box keeps.

use std::f32::consts::PI;

use super::raycast_sphere::SphereField;
use super::smooth::value_at;

impl SphereField<'_> {
    /// Whether the slicer keeps `cell`, met toward `(lon, lat)`: by its
    /// center (HEALPix pixels, which have no rows, by where they lie on the
    /// globe) and its value's height, as sphere.wgsl cuts blocks.
    pub(super) fn kept(&self, cell: (usize, usize), lon: f32, lat: f32) -> bool {
        let placed = match self.matrix.grid.is_healpix() {
            true => self.slice.keeps_uv(lon / (2.0 * PI) + 0.5, 0.5 - lat / PI),
            false => {
                let grid = (self.matrix.width, self.matrix.height);
                self.slice.keeps_cell(cell, grid)
            }
        };
        let h = self
            .scale
            .normalized(value_at(self.layer, self.matrix, cell.0, cell.1));
        placed && self.slice.keeps_height(h, &self.scale, self.relief)
    }
}
