//! Value ranges of coarse bricks made true bounds as finer ones arrive: a
//! mean pyramid's coarse levels understate their finer cells' extremes
//! (max pyramids, Roerdink 2001, avoid this), so each placed brick
//! widens the recorded ranges of its resident ancestors, which the selection
//! (`Lod::draws_nothing`) and the shader's cell skipping read.

use std::ops::Range;

use super::Lod;
use super::subblocks::widen_subs;
use crate::data::multiscale::bricks::BrickKey;
use crate::plots::volume::bricks::BrickTextures;

impl Lod {
    /// Widens the ranges of `key`'s resident ancestors to cover `range` and
    /// `missing`, uploading the ones that changed.
    pub(super) fn widen_ancestors(
        &mut self,
        key: BrickKey,
        range: (f32, f32),
        missing: bool,
        gpu: Option<(&BrickTextures, &wgpu::Queue)>,
    ) {
        if range.0 > range.1 && !missing {
            return;
        }
        let bricks = self.bricks();
        let cells = bricks.cells(key);
        let levels = usize::from(key.level) + 1..self.set.len();
        let ancestors: Vec<BrickKey> = levels.flat_map(|l| bricks.over(l, &cells)).collect();
        for parent in ancestors {
            let Some(placed) = self.residency.get_mut(&parent) else {
                continue;
            };
            let (lo, hi) = placed.range;
            let wider = (lo.min(range.0), hi.max(range.1));
            let had_missing = placed.missing;
            let now_missing = had_missing || missing;
            if wider == (lo, hi) && now_missing == had_missing {
                continue;
            }
            placed.range = wider;
            placed.missing = now_missing;
            if let (Some((gpu, queue)), Some(slot)) = (gpu, self.residency.slot_of(&parent)) {
                gpu.write_range(queue, slot, [wider.0, wider.1], now_missing);
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
        let Some(placed) = self.residency.get_mut(&key) else {
            return;
        };
        let ((lo, hi), missing) = widen_subs(&mut placed.subs, region, values, encoding);
        placed.range = (placed.range.0.min(lo), placed.range.1.max(hi));
        placed.missing |= missing;
        gpu.write_range(
            queue,
            slot,
            [placed.range.0, placed.range.1],
            placed.missing,
        );
        gpu.write_subs(queue, slot, &placed.subs);
    }

    /// Widens resident `key`'s range and missing flag with those of the finer
    /// bricks already resident under it (reads run in parallel, so a child
    /// often arrives before its parent), so a parent that averaged a bright
    /// spot away still counts as drawing it.
    pub(super) fn absorb_descendants(&mut self, key: BrickKey) {
        let bricks = self.bricks();
        let cells = bricks.cells(key);
        let mut wider: Option<(f32, f32)> = None;
        let mut missing = false;
        for z in cells[2].clone() {
            for y in cells[1].clone() {
                for x in cells[0].clone() {
                    let Some((k, p)) = self.residency.lookup(&bricks, [x, y, z]) else {
                        continue;
                    };
                    if k.level < key.level {
                        missing |= p.missing;
                        if p.range.0 <= p.range.1 {
                            let (lo, hi) = wider.unwrap_or(p.range);
                            wider = Some((lo.min(p.range.0), hi.max(p.range.1)));
                        }
                    }
                }
            }
        }
        if let Some(placed) = self.residency.get_mut(&key) {
            if let Some((lo, hi)) = wider {
                placed.range = (placed.range.0.min(lo), placed.range.1.max(hi));
            }
            placed.missing |= missing;
        }
    }
}
