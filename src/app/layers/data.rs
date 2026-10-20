//! The CPU data a layer shows: its current 2D slice or 3D volume.

use std::sync::Arc;

use crate::data::{MatrixData, MatrixPyramid, ViewportResampler, VolumeData};

#[derive(Default)]
pub struct LayerData {
    pub matrix: Option<MatrixData>,
    pub pyramid: Option<Arc<MatrixPyramid>>,
    pub resampler: ViewportResampler,
    pub volume: Option<VolumeData>,
    /// Counts volume resets (new selections), so cache hits know when the
    /// other resident blocks must be projected again.
    pub volume_allocations: u64,
    /// The volume allocation and block origin last written into `volume`, so
    /// playback steps over that same block skip writing it again.
    pub last_volume_block: Option<(u64, Vec<usize>)>,
    /// Block window behind the plotted 2D composite, for raw per-channel hover values.
    pub composite_probe: Option<crate::data::slicing::CompositeProbe>,
    /// How the plotted 2D composite stretched its channels, which its detail
    /// tiles reuse.
    pub composite_stretch: Option<crate::data::slicing::CompositeStretch>,
    /// Dimensions the plotted data runs opposite to storage order along
    /// (`OctantBlock::flipped_dims`), so the hover can map screen indices back.
    pub flipped_dims: Vec<String>,
    /// `store:variable:2d|3d` of the data last built, to tell a new variable
    /// from another step of the same one.
    pub var_key: Option<String>,
    /// The greyscale image channel whose window the color style shows.
    pub greyscale_channel: Option<usize>,
    /// Bumped (`touch_matrix`) whenever `matrix` changes, so caches of data
    /// derived from it (the line plot's GPU payload) know to rebuild.
    pub matrix_version: u64,
    /// Bumped (`touch_volume`) whenever `volume` changes.
    pub volume_version: u64,
    /// The finer level's tile drawn over a multiscale heatmap (base layer).
    pub detail: crate::app::detail::DetailState,
}

impl LayerData {
    /// Records that `matrix` changed.
    pub fn touch_matrix(&mut self) {
        self.matrix_version = self.matrix_version.wrapping_add(1);
    }

    /// Records that `volume` changed.
    pub fn touch_volume(&mut self) {
        self.volume_version = self.volume_version.wrapping_add(1);
    }

    /// Full-resolution `(width, height)` of the 2D data (before any pyramid
    /// resampling), or 1024 x 1024 when there is none.
    pub fn dimensions_2d(&self) -> (usize, usize) {
        self.shown_dimensions_2d().unwrap_or((1024, 1024))
    }

    /// The width and height of the 2D data shown, `None` before any arrives.
    pub fn shown_dimensions_2d(&self) -> Option<(usize, usize)> {
        if let Some(pyr) = &self.pyramid {
            Some((pyr.original_width, pyr.original_height))
        } else {
            self.matrix.as_ref().map(|m| (m.width, m.height))
        }
    }
}
