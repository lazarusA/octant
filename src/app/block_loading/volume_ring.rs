//! A volume whose Z axis is the animated one (time, lat, lon) shows the
//! window of `depth` steps from the current one: a ring buffer the step turns
//! (`get_volume_shifts` reads `ring_shift`), step `t` in plane
//! `(t - range start) % depth`. Each step writes only the planes the window
//! gained, from whichever resident block holds them, so playback never
//! replaces the whole volume at once.

use std::ops::Range;

use crate::app::layers::LayerId;
use crate::app::{OctantApp, SpatialRole};
use crate::data::DimensionSelection;
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
    fn steps(self) -> Range<usize> {
        self.start..self.start + self.depth
    }

    /// The plane step `t` (at least `origin`) goes in: the one place the ring
    /// order is decided, for the writes and the shader's turn alike.
    fn slot(self, t: usize) -> usize {
        (t - self.origin) % self.depth
    }

    /// What plane `slot(t)` holds once settled: step `t`, or nothing past the
    /// variable's end.
    fn expected(self, t: usize) -> Option<usize> {
        (t < self.extent).then_some(t)
    }
}

/// A plane a block holds that the ring lacks: its block-local stored index
/// along Z, the ring plane it goes in and its step.
pub(super) struct RingPlane {
    pub local: usize,
    pub slot: usize,
    pub step: usize,
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
            // Before the range the shift stays 0.
            start: self.layer_step(id).max(origin),
            origin,
            extent: self.layer_animated_dim_extent(id),
            depth,
        })
    }

    /// The steps layer `id`'s ring volume shows, `None` for other layers.
    pub(super) fn ring_steps(&self, id: LayerId) -> Option<Range<usize>> {
        self.ring_window(id).map(Window::steps)
    }

    /// The plane the shader shows first: the current step's.
    pub(crate) fn ring_shift(&self, id: LayerId) -> Option<u32> {
        self.ring_window(id).map(|w| w.slot(w.start) as u32)
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

    /// The steps of layer `id`'s ring window it does not hold yet.
    pub(super) fn ring_missing_steps(&self, id: LayerId) -> Vec<usize> {
        let (Some(w), Some(layer)) = (self.ring_window(id), self.layers.get(id)) else {
            return Vec::new();
        };
        let ring = &layer.data.ring;
        let fresh = ring.allocation == layer.data.volume_allocations;
        w.steps()
            .filter(|&t| t < w.extent)
            .filter(|&t| !fresh || ring.steps.get(w.slot(t)).copied().flatten() != Some(t))
            .collect()
    }

    /// The planes of layer `id`'s ring window that `block` holds and the ring
    /// lacks.
    pub(super) fn ring_planes_to_write(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        z_dim: usize,
    ) -> Vec<RingPlane> {
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
            .map(|t| RingPlane {
                local: t - origin,
                slot: w.slot(t),
                step: t,
            })
            .collect()
    }

    /// Records that plane `slot` of layer `id`'s ring now holds `step`.
    pub(super) fn record_ring_plane(&mut self, id: LayerId, slot: usize, step: usize) {
        if let Some(layer) = self.layers.get_mut(id)
            && let Some(held) = layer.data.ring.steps.get_mut(slot)
        {
            *held = Some(step);
        }
    }

    /// Writes the planes layer `id`'s ring window lacks from the resident
    /// blocks holding them, one block per run of missing steps, then empties
    /// the planes still holding another step.
    pub(super) fn fill_ring(
        &mut self,
        id: LayerId,
        (source_id, var_name): (&str, &str),
        selections: &[DimensionSelection],
        anim_dim: Option<usize>,
    ) {
        let mut covered = 0..0;
        for t in self.ring_missing_steps(id) {
            if covered.contains(&t) {
                continue;
            }
            let Some(block) = self
                .block_cache
                .find_covering_block(source_id, var_name, selections, anim_dim, t)
            else {
                continue;
            };
            let dim = anim_dim.unwrap_or(0);
            let origin = block.origin.get(dim).copied().unwrap_or(0);
            covered = origin..origin + block.shape.get(dim).copied().unwrap_or(0);
            self.apply_block_projection(id, &block);
        }
        self.empty_stale_ring_planes(id);
    }

    /// Empties the planes of layer `id`'s ring window still holding another
    /// step (not loaded yet, or past the variable's end), so the window never
    /// shows old steps wrapped around.
    fn empty_stale_ring_planes(&mut self, id: LayerId) {
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
