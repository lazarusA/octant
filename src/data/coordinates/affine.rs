//! Dimension coordinates from a grid's affine (zarr-conventions `spatial:`
//! `spatial:transform`), for grids that hold no coordinate arrays.

use crate::data::CoordValues;

/// An affine `[a, b, c, d, e, f]` (`x = a*col + b*row + c`,
/// `y = d*col + e*row + f`) and the dimensions it places.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridAffine {
    pub transform: [f64; 6],
    /// Index of the x (column) dimension.
    pub x: usize,
    /// Index of the y (row) dimension.
    pub y: usize,
    /// `spatial:registration: "node"`: the affine locates cell centres, not corners.
    pub node: bool,
}

impl GridAffine {
    /// `(dimension name, coordinate)` of the x and y dimensions at cell
    /// centres, for a grid of shape `shape` with dimensions `dims`. Rotated
    /// or sheared affines place no coordinates, nor do degenerate steps.
    pub fn coordinates<'d>(
        &self,
        dims: &'d [String],
        shape: &[u64],
    ) -> Vec<(&'d str, CoordValues)> {
        let [a, b, c, d, e, f] = self.transform;
        if b != 0.0 || d != 0.0 {
            return Vec::new();
        }
        let half = if self.node { 0.0 } else { 0.5 };
        [(self.x, a, c), (self.y, e, f)]
            .into_iter()
            .filter_map(|(i, step, origin)| {
                let (dim, &len) = (dims.get(i)?, shape.get(i)?);
                let start = origin + half * step;
                let valid = step.is_finite() && step != 0.0 && start.is_finite() && len > 0;
                valid.then(|| {
                    let len = len as usize;
                    (dim.as_str(), CoordValues::Regular { start, step, len })
                })
            })
            .collect()
    }
}
