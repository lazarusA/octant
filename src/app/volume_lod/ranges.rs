//! Value ranges of coarse bricks made true bounds as finer ones arrive: a
//! mean pyramid's coarse levels understate their finer cells' extremes
//! (max pyramids, Roerdink 2001, avoid this), so each placed brick
//! widens the recorded ranges of its resident ancestors, which the selection
//! (`Lod::draws_nothing`) and the shader's cell skipping read.

use std::ops::Range;

use super::Lod;
use super::encode::value_range;
use super::subblocks::widen_subs;
use crate::data::multiscale::bricks::BrickKey;
use crate::plots::volume::bricks::BrickTextures;

impl Lod {
    /// Widens the ranges of `key`'s resident ancestors to cover `range`,
    /// uploading the ones that changed.
    pub(super) fn widen_ancestors(
        &mut self,
        key: BrickKey,
        range: (f32, f32),
        gpu: Option<(&BrickTextures, &wgpu::Queue)>,
    ) {
        if range.0 > range.1 {
            return;
        }
        let bricks = self.bricks();
        // The ancestors of its corner cells: in a pyramid offset by half a
        // cell, a brick's cells may belong to two parents along an axis.
        let cells = bricks.cells(key);
        let corners = (0..8).map(|c: usize| -> [u32; 3] {
            std::array::from_fn(|i| match c >> i & 1 {
                0 => cells[i].start,
                _ => cells[i].end.saturating_sub(1).max(cells[i].start),
            })
        });
        let levels = usize::from(key.level) + 1..self.set.len();
        let mut ancestors: Vec<BrickKey> = levels
            .flat_map(|l| corners.clone().map(move |c| (l, c)))
            .map(|(l, c)| bricks.cell_brick(l, c))
            .collect();
        ancestors.sort_unstable();
        ancestors.dedup();
        for parent in ancestors {
            let Some(placed) = self.residency.get_mut(&parent) else {
                continue;
            };
            let (lo, hi) = placed.range;
            let wider = (lo.min(range.0), hi.max(range.1));
            if wider == (lo, hi) {
                continue;
            }
            placed.range = wider;
            let missing = placed.missing;
            if let (Some((gpu, queue)), Some(slot)) = (gpu, self.residency.slot_of(&parent)) {
                gpu.write_range(queue, slot, [wider.0, wider.1], missing);
            }
        }
    }

    /// Widens resident `key`'s brick and sub-block ranges with the f16
    /// `values` just written over the stored box `region` of its apron, and
    /// uploads them to `slot`.
    pub(super) fn widen_apron(
        &mut self,
        key: BrickKey,
        region: &[Range<usize>; 3],
        values: &[u16],
        (gpu, queue, slot): (&BrickTextures, &wgpu::Queue, [u32; 3]),
    ) {
        let encoding = self.encoding();
        let (lo, hi) = value_range(values, encoding);
        let Some(placed) = self.residency.get_mut(&key) else {
            return;
        };
        placed.range = (placed.range.0.min(lo), placed.range.1.max(hi));
        placed.missing |= values.iter().any(|&h| half::f16::from_bits(h).is_nan());
        widen_subs(&mut placed.subs, region, values, encoding);
        gpu.write_range(
            queue,
            slot,
            [placed.range.0, placed.range.1],
            placed.missing,
        );
        gpu.write_subs(queue, slot, &placed.subs);
    }

    /// Widens resident `key`'s range with those of the finer bricks already
    /// resident under it (reads run in parallel, so a child often arrives
    /// before its parent), so a parent that averaged a bright spot away
    /// still counts as drawing it.
    pub(super) fn absorb_descendants(&mut self, key: BrickKey) {
        let bricks = self.bricks();
        let cells = bricks.cells(key);
        let mut wider: Option<(f32, f32)> = None;
        for z in cells[2].clone() {
            for y in cells[1].clone() {
                for x in cells[0].clone() {
                    let Some((k, p)) = self.residency.lookup(&bricks, [x, y, z]) else {
                        continue;
                    };
                    if k.level < key.level && p.range.0 <= p.range.1 {
                        let (lo, hi) = wider.unwrap_or(p.range);
                        wider = Some((lo.min(p.range.0), hi.max(p.range.1)));
                    }
                }
            }
        }
        if let (Some((lo, hi)), Some(placed)) = (wider, self.residency.get_mut(&key)) {
            placed.range = (placed.range.0.min(lo), placed.range.1.max(hi));
        }
    }
}
