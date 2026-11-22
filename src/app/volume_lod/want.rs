//! Which bricks a resting view wants: the selection over the view, minus bricks that draw nothing or that rays
//! never reach.

use super::{Lod, Shown, ViewKey, occlusion, select};
use crate::app::OctantApp;
use crate::app::detail::SETTLE;
use crate::app::pipeline::paint_layer::CanvasView;
use crate::data::multiscale::bricks::{BrickKey, Bricks, CellBits, cell_index};
use crate::plots::PlotType;

/// Screen pixels a voxel may span at quality 1 before its brick splits.
const PIXELS_PER_VOXEL: f32 = 1.5;

impl Lod {
    /// Selects the bricks `view` wants, keeps them in their slots and
    /// queues those not yet read.
    /// `seen` marks the table cells rays reach (all when `None`): bricks
    /// over none are hidden, never read nor split.
    pub(super) fn want(&mut self, view: &ViewKey, generation: u64, seen: Option<&CellBits>) {
        self.place.aspect = view.aspect;
        let select_view = select::View {
            camera: &view.camera,
            place: self.place,
            window: self.window.clone(),
            px_height: view.px[1].max(1),
            threshold: PIXELS_PER_VOXEL / view.quality.max(0.1),
        };
        let budget = self.residency.slot_count();
        let bricks = self.bricks();
        let hidden = |k: BrickKey| seen.is_some_and(|s| !any_seen(&bricks, k, s));
        let skip = select::Skip {
            empty: &|k| self.draws_nothing(k, view.shown) || hidden(k),
            hidden: &hidden,
            refined: &|k| bricks.children(k).any(|c| self.residency.holds(&c)),
        };
        let wanted = select::select(&bricks, self.root_level, &select_view, budget, skip);
        for key in &wanted {
            self.residency.touch(key, generation);
        }
        self.wanted = wanted.iter().copied().collect();
        self.refresh_finest();
        // Read already: running, or arrived and waiting to be placed.
        let asked: Vec<BrickKey> = (self.pending.values().map(|(k, _)| *k))
            .chain(self.arrived.iter().map(|(k, _, _)| *k))
            .collect();
        self.queue = wanted
            .into_iter()
            .filter(|k| {
                !self.residency.holds(k) && !asked.contains(k) && !self.unreadable.contains(k)
            })
            .collect();
    }
}

impl Lod {
    /// Whether resident `key` and every finer brick under it draw nothing,
    /// so it is never split: all its voxels missing (no NaN color), or all
    /// at the plotted volume's minimum below the shown range (background,
    /// padding). Only then: a coarse level of a mean pyramid understates
    /// its finer cells' extremes, so a dim brick may hold bright voxels.
    pub(super) fn draws_nothing(&self, key: BrickKey, shown: Shown) -> bool {
        let Some(placed) = self.residency.get(&key) else {
            return false;
        };
        let (lo, hi) = placed.range;
        if lo > hi {
            return !shown.nan;
        }
        hi <= shown.floor && shown.floor < shown.lo && !(shown.nan && placed.missing)
    }
}

/// Whether any table cell under `key` is marked in `seen`.
fn any_seen(bricks: &Bricks, key: BrickKey, seen: &CellBits) -> bool {
    let grid = bricks.grid();
    let cells = bricks.cells(key);
    cells[2].clone().any(|z| {
        cells[1].clone().any(|y| {
            (cells[0].clone()).any(|x| seen.get(cell_index(grid, [x, y, z])).unwrap_or(true))
        })
    })
}

impl OctantApp {
    /// Selects the bricks the view wants once it rested; how long until it
    /// counts as resting.
    pub(super) fn settle_volume_lod(&mut self, view: &CanvasView, now: f64) -> Option<f64> {
        let screen = crate::plots::common::compute_aspect_ratio(&view.plot_rect);
        let (ax, ay, az) = self.get_3d_aspect_ratio();
        let key = ViewKey {
            camera: self.nav.orbit_camera(PlotType::Volume, screen),
            aspect: [ax, ay, az],
            px: self.layout.canvas_px,
            quality: self.plot_configs.volume.quality,
            shown: self.shown_values(),
            occluding: self.occluding(),
        };
        let state = &mut self.layers.base.data.volume_lod;
        match state.seen {
            Some((seen, since)) if seen == key && now - since >= SETTLE => {}
            Some((seen, since)) if seen == key => return Some(SETTLE - (now - since)),
            _ => {
                state.seen = Some((key, now));
                state.feedback = None;
                return Some(SETTLE);
            }
        }
        if state.view == Some(key) {
            return None;
        }
        state.view = Some(key);
        state.generation += 1;
        let generation = state.generation;
        let data = &mut self.layers.base.data;
        let (volume, state) = (data.volume.as_ref(), &mut data.volume_lod);
        if let Some(lod) = &mut state.lod {
            // The rays' own report once read back, else the CPU's guess.
            let guessed = match state.feedback {
                Some(_) => None,
                None => super::feedback::guessed_cells(lod, volume?, &key),
            };
            lod.want(
                &key,
                generation,
                state.feedback.as_ref().or(guessed.as_ref()),
            );
        }
        self.drop_unwanted_reads();
        None
    }

    /// The CPU visibility guess's opacity rule (`occlusion.rs`), `None` in
    /// the modes rays read through.
    pub(super) fn occluding(&self) -> Option<occlusion::Opacity> {
        let volume = &self.plot_configs.volume;
        // DVR, and opaque label surfaces (stopping at the first value in
        // range, as opaque DVR does; clip colors only stop rays earlier).
        let opaque_labels = volume.algorithm == 4 && !volume.transparency;
        if !(volume.algorithm == 0 || opaque_labels) || self.layers.base.composite.enabled {
            return None;
        }
        let c = self.get_color_params(&self.layers.base);
        Some(occlusion::Opacity {
            color: c,
            density: volume.opacity,
            opaque: !volume.transparency,
        })
    }

    /// The values the base volume draws.
    /// Everything in MIP and MinIP (a coarse level of a mean pyramid
    /// understates the extremes they draw) and in the classic modes, which
    /// color values outside the range in the colormap's end colors.
    pub(super) fn shown_values(&self) -> Shown {
        let c = self.get_color_params(&self.layers.base);
        let all_drawn = matches!(self.plot_configs.volume.algorithm, 1 | 2 | 5 | 6 | 7);
        let all = all_drawn || c.use_lowclip != 0 || c.use_highclip != 0;
        let (lo, hi) = match all {
            true => (f32::NEG_INFINITY, f32::INFINITY),
            false => (c.cmin.min(c.cmax), c.cmax.max(c.cmin)),
        };
        let floor = self.layers.base.data.volume.as_ref();
        Shown {
            lo,
            hi,
            nan: c.use_nan_color != 0,
            floor: floor.map_or(f32::NEG_INFINITY, |v| v.min_val),
        }
    }
}
