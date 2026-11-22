//! Bricked volume LOD (`.agents/skills/octant-rendering-wgpu/volume_lod.md`):
//! a volume plotted from a level of a multiscale variable (Auto) is drawn
//! from bricks of several levels, finer where the view needs them, read as
//! the view rests and kept within the atlas's slots.
//!
//! - `select.rs`: the bricks a view wants.
//! - `residency.rs`: atlas slots and the indirection table's mirror.
//! - `setup.rs`: when a volume is bricked, and its GPU renderer.
//! - `occlusion.rs`: which table cells a resting DVR view sees.
//! - `apron.rs`: bricks' aprons filled from resident neighbours.
//! - `fetch.rs`: brick reads through the prefetcher (the status bar's
//!   spinner shows them) and their upload.
//! - `draw.rs`: which renderer the volume draws with, and the hover's level
//!   and value under the pointer.
//!
//! Off unless Settings' "Detail by view" (`VolumePlotConfig::lod`) is on:
//! then nothing is read and the volume draws as before. Any mode,
//! composites included, not while playing or animating along x, y or z;
//! the plotted volume draws until every root brick has arrived, and where
//! bricks cannot draw the plot (`setup.rs` `Off`, with a notice).

mod apron;
#[cfg(test)]
mod apron_tests;
mod draw;
mod encode;
mod fetch;
mod occlusion;
#[cfg(test)]
mod occlusion_tests;
mod place;
mod placement;
mod ranges;
#[cfg(test)]
mod ranges_tests;
mod reset;
mod residency;
#[cfg(test)]
mod residency_tests;
mod select;
#[cfg(test)]
mod select_tests;
mod setup;
mod want;

use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Range;
use std::sync::Arc;

use crate::app::OctantApp;
use crate::app::detail::{LevelRead, SETTLE};
use crate::app::pipeline::paint_layer::CanvasView;
use crate::data::multiscale::Multiscale;
use crate::data::multiscale::bricks::{BrickKey, Bricks};
use crate::data::{BlockCacheKey, OctantBlock};
use crate::plots::PlotType;
use crate::plots::camera::OrbitCamera;
use residency::Residency;
use select::Placement;

/// The base layer's bricks, the plot they belong to and the resting view.
#[derive(Default)]
pub struct VolumeLodState {
    pub(crate) context: Option<u64>,
    pub(crate) lod: Option<Lod>,
    /// Counts selections: what the latest one wants is never evicted.
    generation: u64,
    view: Option<ViewKey>,
    seen: Option<(ViewKey, f64)>,
    /// The table cells the resting view's rays reached (`feedback.rs`).
    feedback: Option<crate::data::multiscale::bricks::CellBits>,
    /// A composite's new settings and since when (`recolor_volume_lod`).
    colors: Option<(u64, f64)>,
    /// The composite settings the last build used (`composite::colors_of`),
    /// whether or not it built bricks.
    colored: Option<u64>,
    /// The variable and reason last told the plot stays dense (once).
    warned: Option<(String, setup::Off)>,
}

/// One plot's bricks.
pub(crate) struct Lod {
    set: Arc<Multiscale>,
    /// The variable's x, y and z dimensions.
    dims: [usize; 3],
    /// Atlas slots along x, y, z.
    slots: [u32; 3],
    /// The plotted level, whose bricks are the roots (pinned).
    root_level: usize,
    roots: Vec<BrickKey>,
    place: Placement,
    /// Storage runs opposite to the drawn order along x, y, z.
    flip: [bool; 3],
    /// Table cells inside the plotted window.
    window: [Range<u32>; 3],
    /// Atlas values are `(value - offset) / scale`.
    offset: f32,
    scale: f32,
    residency: Residency<Placed>,
    pending: HashMap<BlockCacheKey, (BrickKey, LevelRead)>,
    /// Wanted bricks to read, in order.
    queue: VecDeque<BrickKey>,
    /// Read bricks waiting to be placed (`fetch::place_arrivals`).
    arrived: VecDeque<(BrickKey, LevelRead, OctantBlock)>,
    /// The egui time the last batch was placed.
    placed_at: f64,
    /// What the latest selection wants (kept in the atlas over the rest).
    wanted: HashSet<BrickKey>,
    /// The finest level wanted and resident: the ray's step at rest.
    finest: usize,
    /// A failed read was reported (once per plot).
    failure_reported: bool,
    /// Bricks whose block did not hold them: never asked again.
    unreadable: HashSet<BrickKey>,
    /// How a composite's bricks are colored (`composite.rs`).
    composite: Option<composite::CompositeLod>,
    /// What the atlas holds (`setup.rs` `atlas_encoding`).
    texel: crate::plots::volume::bricks::AtlasTexel,
}

/// A resident brick: the block it was read as (the hover reads values from
/// the block cache while it is there) and the range of its valid values.
pub(crate) struct Placed {
    block: BlockCacheKey,
    range: (f32, f32),
    /// Whether any of its voxels is missing.
    missing: bool,
    /// Its sub-blocks' ranges (`subblocks.rs`).
    subs: subblocks::SubRanges,
}

impl Lod {
    fn bricks(&self) -> Bricks<'_> {
        Bricks {
            set: &self.set,
            dims: self.dims,
        }
    }

    /// Whether every root brick is in the atlas.
    pub(crate) fn ready(&self) -> bool {
        self.roots.iter().all(|k| self.residency.holds(k))
    }

    /// Recomputes the finest level wanted and resident.
    fn refresh_finest(&mut self) {
        let resident = self.wanted.iter().filter(|k| self.residency.holds(k));
        self.finest = resident
            .map(|k| usize::from(k.level))
            .min()
            .unwrap_or(self.root_level)
            .min(self.root_level);
    }
}

/// The view a selection was made for.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ViewKey {
    camera: OrbitCamera,
    aspect: [f32; 3],
    px: [u32; 2],
    quality: f32,
    /// The values drawn (`Lod::draws_nothing`).
    shown: Shown,
    /// How DVR hides what lies behind (`occlusion.rs`), `None` in modes
    /// that read every voxel.
    occluding: Option<occlusion::Opacity>,
}

/// The values a volume draws: the color range, or everything when clip
/// colors are on or the mode colors values outside it; whether missing
/// values draw (a NaN color), and the plotted volume's minimum (`floor`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Shown {
    lo: f32,
    hi: f32,
    nan: bool,
    floor: f32,
}

impl OctantApp {
    /// Keeps the base layer's bricks in line with the plot and the view:
    /// builds them for a new plot (its roots asked for at once), and once
    /// the view has rested `SETTLE` seconds (egui time `now`) selects the
    /// bricks it wants; reads queued bricks while the view is still.
    /// Returns how long to wait before looking again, for a repaint then.
    pub(crate) fn refresh_volume_lod(
        &mut self,
        plot_type: PlotType,
        view: &CanvasView,
        now: f64,
    ) -> Option<f64> {
        let context = self.volume_lod_context(plot_type);
        if context != self.layers.base.data.volume_lod.context {
            // A new step reuses the atlas when it fits: no new one per step.
            let renderer = self.layers.base.renderers.volume_bricked.take();
            self.reset_volume_lod(context, renderer);
        }
        let recolor = self.recolor_volume_lod(now);
        // No bricks (the last build stayed dense): a pending recolor still
        // asks to be looked at again.
        if self.layers.base.data.volume_lod.lod.is_none() {
            return recolor;
        }
        self.place_arrivals(now);
        let lod = self.layers.base.data.volume_lod.lod.as_ref()?;
        let placing = (!lod.arrived.is_empty()).then_some(fetch::PLACE_EVERY);
        if self.nav.view_interacting {
            return placing.or(recolor);
        }
        let reading = self.take_ray_feedback(view);
        let wait = self.settle_volume_lod(view, now);
        self.requeue_lost_bricks();
        self.request_queued_bricks();
        let lod = self.layers.base.data.volume_lod.lod.as_ref()?;
        let waiting = !lod.queue.is_empty() && lod.pending.is_empty();
        wait.or(placing)
            .or(reading)
            .or(recolor)
            .or(waiting.then_some(SETTLE))
    }
}
