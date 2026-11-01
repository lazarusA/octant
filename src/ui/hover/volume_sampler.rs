//! What the 3D hover samples: the plotted volume (or a 2D slice as one
//! plane), its circular shifts and, for a thinned point cloud, the grid it
//! draws.

use crate::app::OctantApp;
use crate::app::layers::{Layer, PointGrid};
use crate::data::MatrixData;
use crate::data::render::decimate::{drawn_index, sample_index};
use crate::plots::PlotType;

/// Helper for volumetric sampling, circular dataset shifts, and 3D ray marching.
pub struct VolumeSampler<'a> {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub values: &'a [f32],
    pub shift_x: usize,
    pub shift_y: usize,
    pub shift_z: usize,
    /// The layer whose volume this is, read through its GPU blend
    /// (`Layer::gpu_blended_voxel`) when it has one.
    pub layer: Option<&'a Layer>,
    /// The grid a thinned point cloud draws (`PointGrid`): rays pick its
    /// cells, read at the voxels they draw.
    pub drawn: Option<PointGrid>,
}

impl<'a> VolumeSampler<'a> {
    pub fn from_app(app: &'a OctantApp, matrix: Option<&'a MatrixData>) -> Option<Self> {
        let (shift_x, shift_y, shift_z) = app.get_volume_shifts();
        if let Some(v) = &app.layers.base.data.volume {
            if v.width == 0 || v.height == 0 || v.depth == 0 || v.values.is_empty() {
                return None;
            }
            Some(Self {
                width: v.width,
                height: v.height,
                depth: v.depth,
                values: &v.values,
                shift_x: shift_x as usize,
                shift_y: shift_y as usize,
                shift_z: shift_z as usize,
                layer: Some(&app.layers.base),
                drawn: app
                    .layers
                    .base
                    .data
                    .points
                    .filter(|_| app.effective_canvas_plot_type() == PlotType::PointCloud),
            })
        } else if let Some(m) = matrix {
            if m.width == 0 || m.height == 0 || m.values.is_empty() {
                return None;
            }
            Some(Self {
                width: m.width,
                height: m.height,
                depth: 1,
                values: &m.values,
                shift_x: shift_x as usize,
                shift_y: shift_y as usize,
                shift_z: shift_z as usize,
                layer: None,
                drawn: None,
            })
        } else {
            None
        }
    }

    /// The voxel the drawn grid's cell `(cx, cy)` shows (`sample_index`).
    pub(super) fn drawn_voxel(&self, cx: usize, cy: usize) -> (usize, usize) {
        match self.drawn {
            Some(g) => (
                sample_index(cx, g.stride, self.width),
                sample_index(cy, g.stride, self.height),
            ),
            None => (cx, cy),
        }
    }

    /// The drawn grid's cell showing voxel `(x, y)`, and the grid's size.
    pub fn drawn_cell(&self, x: usize, y: usize) -> ((usize, usize), (usize, usize)) {
        match self.drawn {
            Some(g) => (
                (
                    drawn_index(x, g.stride, self.width),
                    drawn_index(y, g.stride, self.height),
                ),
                (g.width, g.height),
            ),
            None => ((x, y), (self.width, self.height)),
        }
    }

    pub fn sample_cell(&self, cx: usize, cy: usize, cz: usize) -> f32 {
        let shifted_x = (cx + self.shift_x) % self.width;
        let shifted_y = (cy + self.shift_y) % self.height;
        let shifted_z = (cz + self.shift_z) % self.depth;
        let idx = shifted_z * (self.width * self.height) + shifted_y * self.width + shifted_x;
        let blended = self.layer.and_then(|l| l.gpu_blended_voxel(idx));
        blended.unwrap_or_else(|| self.values.get(idx).copied().unwrap_or(f32::NAN))
    }

    pub fn is_visible(&self, app: &OctantApp, val: f32) -> bool {
        let is_nan = val.is_nan() || val.abs() > 1e30;
        if is_nan {
            app.layers.base.color.use_nan_color
        } else if app.layers.base.composite.enabled {
            (val.to_bits() & 0x00FF_FFFF) != 0
        } else {
            let in_low =
                app.layers.base.color.use_lowclip || val >= app.layers.base.color.range_min;
            let in_high =
                app.layers.base.color.use_highclip || val <= app.layers.base.color.range_max;
            in_low && in_high
        }
    }
}
