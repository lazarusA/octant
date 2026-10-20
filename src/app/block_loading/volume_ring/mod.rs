//! A volume whose Z axis is the animated one (time, lat, lon) shows the
//! window of `depth` steps from the current one, counted around the series
//! (after the last step come the first ones, looping or not): a ring buffer
//! the step turns by one plane (`get_volume_shifts` reads `ring_shift`), so
//! each step writes only the plane the window gained, from whichever resident
//! block holds it, and playback never replaces the whole volume at once.

use std::ops::Range;

use crate::app::layers::LayerId;
use crate::app::{OctantApp, SpatialRole};
use crate::data::DimensionSelection;
use crate::data::octant_block::OctantBlock;
use crate::plots::PlotType;

mod window;

pub(super) use window::RingPlane;
use window::{Window, front_plane};

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
        let layer = self.layers.get(id)?;
        let depth = layer.data.volume.as_ref()?.depth.max(1);
        let extent = self.layer_animated_dim_extent(id).max(1);
        let start = self.layer_step(id) % extent;
        let ring = &layer.data.ring;
        let anchor = ring.anchor.filter(|_| ring.steps.len() == depth);
        Some(Window {
            start,
            extent,
            depth,
            front: front_plane(anchor, start, extent, depth),
        })
    }

    /// Whether layer `id` is a ring volume.
    pub(super) fn has_ring(&self, id: LayerId) -> bool {
        self.ring_window(id).is_some()
    }

    /// Whether layer `id`'s ring window shows a step in `steps`, `None` for
    /// other layers.
    pub(super) fn ring_overlaps(&self, id: LayerId, steps: Range<usize>) -> Option<bool> {
        let w = self.ring_window(id)?;
        let [a, b] = w.step_ranges();
        Some(
            [a, b]
                .iter()
                .any(|r| r.start < steps.end && steps.start < r.end),
        )
    }

    /// The chunks of `cs` steps layer `id`'s ring window reads, front first.
    pub(super) fn ring_chunks(&self, id: LayerId, cs: usize) -> Vec<usize> {
        let Some(w) = self.ring_window(id) else {
            return Vec::new();
        };
        let cs = cs.max(1);
        let mut chunks: Vec<usize> = w
            .step_ranges()
            .iter()
            .filter(|r| !r.is_empty())
            .flat_map(|r| r.start / cs..=(r.end - 1) / cs)
            .collect();
        chunks.dedup();
        chunks
    }

    /// The plane the shader shows first: the current step's.
    pub(crate) fn ring_shift(&self, id: LayerId) -> Option<u32> {
        self.ring_window(id).map(|w| w.front as u32)
    }

    /// The step plane `slot` of layer `id`'s ring holds, for the hover.
    pub(crate) fn ring_plane_step(&self, id: LayerId, slot: usize) -> Option<usize> {
        self.ring_window(id)?;
        self.layers
            .get(id)?
            .data
            .ring
            .steps
            .get(slot)
            .copied()
            .flatten()
    }

    /// Steps past the shown one layer `id` must hold to show it: the rest of
    /// its ring window.
    pub(crate) fn ring_lead(&self, id: LayerId) -> usize {
        self.ring_window(id).map_or(0, |w| w.depth - 1)
    }

    /// Starts layer `id`'s ring record afresh after a new volume, and anchors
    /// it at window `w`, which the writes that follow fill.
    fn anchor_ring(&mut self, id: LayerId, w: Window) {
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let ring = &mut layer.data.ring;
        if ring.allocation != layer.data.volume_allocations || ring.steps.len() != w.depth {
            ring.allocation = layer.data.volume_allocations;
            ring.steps = vec![None; w.depth];
        }
        ring.anchor = Some((w.start, w.front));
    }

    /// The (plane, step) pairs of layer `id`'s ring window the ring lacks.
    fn ring_gaps(&self, id: LayerId, w: Window) -> Vec<(usize, usize)> {
        let Some(layer) = self.layers.get(id) else {
            return Vec::new();
        };
        let ring = &layer.data.ring;
        let fresh = ring.allocation == layer.data.volume_allocations;
        w.planes()
            .filter(|&(slot, step)| !fresh || ring.steps.get(slot).copied().flatten() != Some(step))
            .collect()
    }

    /// The steps of layer `id`'s ring window it does not hold yet.
    pub(super) fn ring_missing_steps(&self, id: LayerId) -> Vec<usize> {
        self.ring_window(id).map_or_else(Vec::new, |w| {
            self.ring_gaps(id, w).into_iter().map(|(_, t)| t).collect()
        })
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
        self.anchor_ring(id, w);
        let origin = block.origin.get(z_dim).copied().unwrap_or(0);
        let held = origin..origin + block.shape.get(z_dim).copied().unwrap_or(0);
        self.ring_gaps(id, w)
            .into_iter()
            .filter(|(_, step)| held.contains(step))
            .map(|(slot, step)| RingPlane {
                local: step - origin,
                slot,
                step,
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
    /// the planes still holding another step (not loaded yet).
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
    /// step, so the window never shows a step out of place.
    fn empty_stale_ring_planes(&mut self, id: LayerId) {
        let Some(w) = self.ring_window(id) else {
            return;
        };
        self.anchor_ring(id, w);
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let Some(volume) = &mut layer.data.volume else {
            return;
        };
        let mut emptied = false;
        for (slot, step) in w.planes() {
            let Some(held) = layer.data.ring.steps.get_mut(slot) else {
                continue;
            };
            if held.is_some_and(|h| h != step) {
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
