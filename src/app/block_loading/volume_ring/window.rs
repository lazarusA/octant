//! The ring window's geometry: which step each plane of the window shows,
//! and where the window starts in the ring.

use std::ops::Range;

/// A layer's ring window: window position `k` shows step
/// `(start + k) % extent` in plane `(front + k) % depth`.
#[derive(Clone, Copy)]
pub(super) struct Window {
    pub start: usize,
    pub extent: usize,
    pub depth: usize,
    pub front: usize,
}

impl Window {
    /// Every (plane, step) of the window, front to back.
    pub fn planes(self) -> impl Iterator<Item = (usize, usize)> {
        (0..self.depth).map(move |k| {
            (
                (self.front + k) % self.depth,
                (self.start + k) % self.extent,
            )
        })
    }

    /// The window's steps as at most two ranges, before and after the wrap.
    pub fn step_ranges(self) -> [Range<usize>; 2] {
        let end = self.start + self.depth;
        [
            self.start..end.min(self.extent),
            0..end.saturating_sub(self.extent),
        ]
    }
}

/// The plane the window starting at `start` begins in: the anchor's plane
/// turned by the shortest way from the anchor's step around the series, so
/// the step after the last one is a turn of one like any other.
pub(super) fn front_plane(
    anchor: Option<(usize, usize)>,
    start: usize,
    extent: usize,
    depth: usize,
) -> usize {
    let Some((step, plane)) = anchor else {
        return 0;
    };
    let (extent, depth) = (extent as isize, depth as isize);
    let mut turn = (start as isize - step as isize).rem_euclid(extent);
    if turn > extent / 2 {
        turn -= extent;
    }
    (plane as isize + turn).rem_euclid(depth) as usize
}

/// A plane a block holds that the ring lacks: its block-local stored index
/// along Z, the ring plane it goes in and its step.
pub(in crate::app::block_loading) struct RingPlane {
    pub local: usize,
    pub slot: usize,
    pub step: usize,
}
