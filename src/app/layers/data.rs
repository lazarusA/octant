//! The CPU data a layer shows: its current 2D slice or 3D volume.

use std::sync::Arc;

use crate::data::matrix_data::MatrixData;
use crate::data::{MatrixPyramid, ViewportResampler, VolumeData};

#[derive(Default)]
pub struct LayerData {
    pub matrix: Option<MatrixData>,
    pub pyramid: Option<Arc<MatrixPyramid>>,
    pub resampler: ViewportResampler,
    pub volume: Option<VolumeData>,
    /// Counts volume resets (new selections), so cache hits know when the
    /// other resident blocks must be projected again.
    pub volume_allocations: u64,
    /// Block window behind the plotted 2D composite, for raw per-channel hover values.
    pub composite_probe: Option<crate::data::slicing::CompositeProbe>,
    /// Dimensions the plotted data runs opposite to storage order along
    /// (`OctantBlock::flipped_dims`), so the hover can map screen indices back.
    pub flipped_dims: Vec<String>,
    /// `store:variable:2d|3d` of the data last built, to tell a new variable
    /// from another step of the same one.
    pub var_key: Option<String>,
    /// Bumped (`touch`) whenever `matrix` or `volume` change, so caches of
    /// data derived from them (the line plot's GPU payload) know to rebuild.
    pub version: u64,
}

impl LayerData {
    /// Records that `matrix` or `volume` changed.
    pub fn touch(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    /// Full-resolution `(width, height)` of the 2D data (before any pyramid
    /// resampling), or 1024 x 1024 when there is none.
    pub fn dimensions_2d(&self) -> (usize, usize) {
        if let Some(pyr) = &self.pyramid {
            (pyr.original_width, pyr.original_height)
        } else if let Some(m) = &self.matrix {
            (m.width, m.height)
        } else {
            (1024, 1024)
        }
    }
}
