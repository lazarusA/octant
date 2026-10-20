//! A volume whose Z axis is the animated one (time, lat, lon) shows the
//! window of `depth` steps from the current one: a ring buffer the step turns
//! (`get_volume_shifts`), step `t` in plane `(t - range start) % depth`. Each
//! step writes only the planes the window gained, from whichever resident
//! block holds them, so playback never replaces the whole volume at once.

use crate::app::layers::LayerId;
use crate::app::{OctantApp, SpatialRole};
use crate::data::octant_block::OctantBlock;
use crate::plots::PlotType;

/// A layer's ring window: steps `start..start + depth`, the selected range
/// starting at `origin`, the variable `extent` steps long.
#[derive(Clone, Copy)]
struct Window {
    start: usize,
    origin: usize,
    extent: usize,
    depth: usize,
}

impl Window {
    fn steps(self) -> std::ops::Range<usize> {
        self.start..self.start + self.depth
    }

    /// The plane step `t` (at least `origin`) goes in.
    fn slot(self, t: usize) -> usize {
        (t - self.origin) % self.depth
    }

    /// What plane `slot(t)` holds once settled: step `t`, or nothing past the
    /// variable's end.
    fn expected(self, t: usize) -> Option<usize> {
        (t < self.extent).then_some(t)
    }
}

impl OctantApp {
    /// Whether layer `id` is a volume or point cloud animated along its Z axis.
    pub(crate) fn animates_along_z(&self, id: LayerId) -> bool {
        let Some(dim) = self.layer_animated_dim(id) else {
            return false;
        };
        let is_3d = self.layer_selections(id).is_some_and(|(_, staged)| {
            matches!(staged.plot_type, PlotType::Volume | PlotType::PointCloud)
        });
        is_3d
            && self
                .layer_dim_config(id)
                .get(dim)
                .is_some_and(|c| c.spatial == SpatialRole::Z)
    }

    fn ring_window(&self, id: LayerId) -> Option<Window> {
        if !self.animates_along_z(id) {
            return None;
        }
        let dim = self.layer_animated_dim(id)?;
        let depth = self.layers.get(id)?.data.volume.as_ref()?.depth.max(1);
        let origin = self.layer_dim_ranges(id).get(dim).map_or(0, |r| r.0);
        Some(Window {
            // Before the range the shift stays 0 (`get_volume_shifts`).
            start: self.layer_step(id).max(origin),
            origin,
            extent: self.layer_animated_dim_extent(id),
            depth,
        })
    }

    /// Steps past the shown one layer `id` must hold to show it: the rest of
    /// its ring window.
    pub(crate) fn ring_lead(&self, id: LayerId) -> usize {
        self.ring_window(id).map_or(0, |w| w.depth - 1)
    }

    /// Starts layer `id`'s ring record afresh after a new volume.
    fn reset_ring(&mut self, id: LayerId, w: Window) {
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let ring = &mut layer.data.ring;
        if ring.allocation != layer.data.volume_allocations || ring.steps.len() != w.depth {
            ring.allocation = layer.data.volume_allocations;
            ring.steps = vec![None; w.depth];
        }
    }

    /// The planes of layer `id`'s ring window that `block` holds and the ring
    /// lacks, as (block-local stored index along `z_dim`, step).
    pub(super) fn ring_planes_to_write(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        z_dim: usize,
    ) -> Vec<(usize, usize)> {
        let Some(w) = self.ring_window(id) else {
            return Vec::new();
        };
        self.reset_ring(id, w);
        let Some(layer) = self.layers.get(id) else {
            return Vec::new();
        };
        let origin = block.origin.get(z_dim).copied().unwrap_or(0);
        let held = origin..origin + block.shape.get(z_dim).copied().unwrap_or(0);
        let steps = &layer.data.ring.steps;
        w.steps()
            .filter(|&t| t < w.extent && held.contains(&t))
            .filter(|&t| steps.get(w.slot(t)).copied().flatten() != Some(t))
            .map(|t| (t - origin, t))
            .collect()
    }

    /// Records that layer `id`'s ring now holds `step`.
    pub(super) fn record_ring_plane(&mut self, id: LayerId, step: usize) {
        let Some(w) = self.ring_window(id) else {
            return;
        };
        if let Some(layer) = self.layers.get_mut(id)
            && let Some(slot) = layer.data.ring.steps.get_mut(w.slot(step))
        {
            *slot = Some(step);
        }
    }

    /// Whether a plane of layer `id`'s ring window holds anything but its step.
    pub(super) fn ring_missing(&self, id: LayerId) -> bool {
        let (Some(w), Some(layer)) = (self.ring_window(id), self.layers.get(id)) else {
            return false;
        };
        let ring = &layer.data.ring;
        ring.allocation != layer.data.volume_allocations
            || w.steps()
                .any(|t| ring.steps.get(w.slot(t)).copied().flatten() != w.expected(t))
    }

    /// Empties the planes of layer `id`'s ring window still holding another
    /// step (not loaded yet, or past the variable's end), so the window never
    /// shows old steps wrapped around.
    pub(super) fn empty_stale_ring_planes(&mut self, id: LayerId) {
        let Some(w) = self.ring_window(id) else {
            return;
        };
        self.reset_ring(id, w);
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let Some(volume) = &mut layer.data.volume else {
            return;
        };
        let mut emptied = false;
        for t in w.steps() {
            let slot = w.slot(t);
            let Some(held) = layer.data.ring.steps.get_mut(slot) else {
                continue;
            };
            if held.is_some() && *held != w.expected(t) {
                *held = None;
                let range = volume.plane_range(slot, 1);
                if let Some(plane) = volume.values.get_mut(range) {
                    plane.fill(f32::NAN);
                }
                layer.renderers.mark_volume_dirty(slot..slot + 1);
                emptied = true;
            }
        }
        if emptied {
            layer.data.touch_volume();
        }
    }
}
