//! The CPU data a layer shows: its current 2D slice or 3D volume.

use std::sync::Arc;

use crate::data::{MatrixData, MatrixPyramid, ViewportResampler, VolumeData};

#[derive(Default)]
pub struct LayerData {
    pub matrix: Option<MatrixData>,
    pub pyramid: Option<Arc<MatrixPyramid>>,
    /// The thinned grid the surface, sphere and 3D coastlines draw when
    /// the 2D data has more cells than the canvas budget allows.
    pub mesh: Option<MeshGrid>,
    /// How the point cloud thins the volume when it has more voxels than
    /// the canvas budget allows.
    pub points: Option<PointGrid>,
    pub resampler: ViewportResampler,
    pub volume: Option<VolumeData>,
    /// Counts volume resets (new selections), so cache hits know when the
    /// other resident blocks must be projected again.
    pub volume_allocations: u64,
    /// The volume allocation and block origin last written into `volume`, so
    /// playback steps over that same block skip writing it again.
    pub last_volume_block: Option<(u64, Vec<usize>)>,
    /// Which step each Z plane of `volume` holds while Z is the animated axis.
    pub ring: RingPlanes,
    /// A 3D composite's raw channels when the GPU blends them (`volume`
    /// then holds the first channel's values, for its grid).
    pub channel_volume: Option<ChannelVolume>,
    /// Block window behind the plotted 2D composite, for raw per-channel hover values.
    pub composite_probe: Option<crate::data::slicing::CompositeProbe>,
    /// How the plotted composite stretched its channels, which its detail
    /// tiles (2D) and the GPU blend reuse.
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

/// A 2D matrix with one cell in `stride` along each axis
/// (`MatrixData::decimated`): what meshes draw, cost growing per cell.
pub struct MeshGrid {
    pub matrix: MatrixData,
    pub stride: usize,
}

/// A volume thinned for the point cloud: one voxel in `stride` along x and
/// y (`VolumeData::thinned_planes`), every Z plane, `width` x `height` a
/// plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointGrid {
    pub stride: usize,
    pub width: usize,
    pub height: usize,
}

/// The channels of a 3D composite blended on the GPU, stacked along Z:
/// channel `c_start + c`'s plane `z` is plane `c * depth + z` of `volume`
/// (`depth` the layer volume's), as the GPU holds them
/// (`plots::ChannelRenderer`, one `w * h * depth` plane per channel).
pub struct ChannelVolume {
    pub volume: VolumeData,
    pub c_start: usize,
    pub channels: usize,
}

impl ChannelVolume {
    /// Writes each channel's slab of `slabs` at `[x, y, z]` of its volume
    /// (`depth` planes a channel), as deep as the slab or the volume's end.
    pub fn write_slabs(
        &mut self,
        [x, y, z]: [usize; 3],
        depth: usize,
        slab_depth: usize,
        slabs: &[VolumeData],
    ) {
        let planes = slab_depth.min(depth.saturating_sub(z));
        for (c, s) in slabs.iter().enumerate() {
            let at = [x, y, c * depth + z];
            self.volume
                .update_subvolume(at, [s.width, s.height, planes], &s.values);
        }
    }
}

/// The steps held by a volume whose Z axis is the animated one: a ring
/// buffer the step turns (`get_volume_shifts`), plane `slot` holding
/// `steps[slot]`, for volume allocation `allocation`; `anchor` is the last
/// written window's (first step, its plane), from which the next window turns.
#[derive(Default)]
pub struct RingPlanes {
    pub allocation: u64,
    pub steps: Vec<Option<usize>>,
    pub anchor: Option<(usize, usize)>,
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

    /// Empties Z plane `slot` of `volume` and, when the GPU blends its
    /// channels, that plane of every channel, which is what it draws.
    pub fn empty_volume_plane(&mut self, slot: usize) {
        let Some(volume) = &mut self.volume else {
            return;
        };
        let depth = volume.depth;
        let range = volume.plane_range(slot, 1);
        if let Some(plane) = volume.values.get_mut(range) {
            plane.fill(f32::NAN);
        }
        if let Some(cv) = &mut self.channel_volume {
            for c in 0..cv.channels {
                let range = cv.volume.plane_range(c * depth + slot, 1);
                if let Some(plane) = cv.volume.values.get_mut(range) {
                    plane.fill(f32::NAN);
                }
            }
        }
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
