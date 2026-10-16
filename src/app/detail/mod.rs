//! Detail tiles: a heatmap plotted from a coarse level of a multiscale
//! variable (Auto) shows the visible part at the finest level that fits the
//! screen once zooming or panning stops, drawn over the coarse plot (which
//! stays as the placeholder until the tile arrives, and around it).
//!
//! - `geometry.rs`: the plotted and visible windows on level 0's grid, the
//!   finer level's cells covering them, and where those sit on the quad.
//! - `want.rs`: the finer level the resting view asks for; stale tiles.
//! - `tile.rs`: requesting a tile (cache first), slicing an arrived block
//!   into it and uploading it to its layer's detail renderer, painted right
//!   after that layer's heatmap (base, base tile, overlay, overlay tile).
//! - `follow.rs`: overlays' tiles, which follow the base layer's.
//!
//! A tile is asked for only once the view rests (`view_interacting` off and
//! the view unchanged for `SETTLE` seconds, so the frames between mouse
//! wheel ticks ask nothing) and playback is stopped, on regular grids
//! without composites; it is kept while it still covers the view at the
//! level the view wants, and dropped when the plot (selection, step)
//! changes. The hover reads it where it is drawn (`ui/hover/detail.rs`).
//!
//! Each drawn overlay whose variable is multiscale reads, as soon as the
//! base layer asks for a tile, the level of its own stack on the base
//! tile's grid (`multiscale::level_matching`) over the same cells, so both
//! tiles line up cell for cell at the same place on the quad; an overlay
//! whose stack has no such level gets none. The hover's overlay rows read
//! an overlay's tile under the pointer (`ui/hover/overlays.rs`).

#[cfg(test)]
mod fetch_tests;
mod follow;
#[cfg(test)]
mod follow_tests;
mod geometry;
#[cfg(test)]
mod geometry_tests;
#[cfg(test)]
mod tests;
mod tile;
mod want;

use crate::app::OctantApp;
use crate::app::pipeline::paint_layer::CanvasView;
use crate::data::BlockCacheKey;
use crate::data::multiscale::Multiscale;
use crate::plots::PlotType;
use geometry::{Axis, covers};

/// Share of the visible span a tile reads beyond each side, so small pans
/// keep it.
const MARGIN: f64 = 0.25;

/// Seconds a view stays unchanged before it counts as resting: longer than
/// the frames between mouse wheel ticks.
const SETTLE: f64 = 0.15;

/// The base layer's detail tile, the one being fetched, and the view the
/// last decision was made for.
#[derive(Default)]
pub struct DetailState {
    pub tile: Option<DetailTile>,
    pub pending: Option<PendingDetail>,
    /// A tile the prefetcher refused while every worker was busy, asked
    /// again once one frees up (`retry_refused_details`).
    refused: Option<TilePlan>,
    /// The view last decided for.
    view: Option<ViewKey>,
    /// The latest view and the egui time it first showed.
    seen: Option<(ViewKey, f64)>,
    /// An overlay's: the base tile and overlay selection last followed
    /// (`follow.rs`).
    follows: Option<u64>,
}

impl DetailState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Drops the tile being fetched or waiting for a worker.
    fn drop_request(&mut self) {
        self.pending = None;
        self.refused = None;
    }
}

/// What a tile shows and where.
#[derive(Debug, Clone)]
pub struct DetailTile {
    pub level: usize,
    /// The level's variable (index into the dataset's variables).
    pub var_idx: usize,
    /// Its x and y dimensions, and the level cells where the tile starts
    /// along them.
    pub dims: [usize; 2],
    pub origin: [u64; 2],
    /// The level cells where it ends (exclusive) along them.
    pub end: [u64; 2],
    /// The tile's values, kept for the hover once it is shown.
    pub matrix: Option<std::sync::Arc<crate::data::MatrixData>>,
    /// The plot it belongs to (`OctantApp::detail_context`).
    pub context: u64,
    /// Level-0 positions it covers along x and y.
    pub cover: [(f64, f64); 2],
    /// Its `tile_bounds` on the plotted quad: `[u0, v0, u1, v1]`.
    pub uv: [f32; 4],
}

/// A tile being fetched, with what slicing its block needs.
pub struct PendingDetail {
    /// The layer the tile is for.
    pub layer: crate::app::layers::LayerId,
    pub key: BlockCacheKey,
    pub tile: DetailTile,
    /// The level's dimension names, roles, indices and ranges.
    pub dims: Vec<String>,
    pub config: Vec<crate::app::DimConfig>,
    pub indices: Vec<usize>,
    pub ranges: Vec<(usize, usize)>,
    pub anim_dim: Option<usize>,
    pub step: usize,
}

/// A tile's fetch: what slicing its block needs, its request and store.
type TilePlan = (
    PendingDetail,
    crate::data::SliceRequest,
    crate::data::StoreHandle,
);

/// The view a decision was made for.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ViewKey {
    pan: [f32; 2],
    zoom: f32,
    aspect: [f32; 2],
    px: [u32; 2],
    context: u64,
}

/// What the view asks for: a finer level over the visible window.
struct Want {
    set: std::sync::Arc<Multiscale>,
    level: usize,
    context: u64,
    axes: [Axis; 2],
    visible: [(f64, f64); 2],
}

impl OctantApp {
    /// Keeps the base layer's detail tile in line with the view: drops it
    /// when the plot changed, and once the view has rested `SETTLE` seconds
    /// (egui time `now`) asks for the visible window at a finer level when
    /// the plotted one is coarser than the screen. Returns how long to wait
    /// before the view counts as resting, for a repaint then. Overlays'
    /// tiles follow the base layer's.
    pub(crate) fn refresh_detail(
        &mut self,
        plot_type: PlotType,
        view: &CanvasView,
        now: f64,
    ) -> Option<f64> {
        let wait = self.refresh_base_detail(plot_type, view, now);
        self.follow_detail_overlays();
        wait
    }

    /// The base layer's part of [`Self::refresh_detail`].
    fn refresh_base_detail(
        &mut self,
        plot_type: PlotType,
        view: &CanvasView,
        now: f64,
    ) -> Option<f64> {
        let context = self.detail_context(plot_type);
        let pending_lost = self.drop_stale_detail(context);
        let context = context?;
        if self.nav.view_interacting {
            return None;
        }
        let key = ViewKey {
            pan: view.pan,
            zoom: view.zoom,
            aspect: view.aspect_scale,
            px: self.layout.canvas_px,
            context,
        };
        let state = &mut self.layers.base.data.detail;
        match state.seen {
            Some((seen, since)) if seen == key && now - since >= SETTLE => {}
            Some((seen, since)) if seen == key => return Some(SETTLE - (now - since)),
            _ => {
                // Wheel ticks arrive frames apart: wait until the view stays.
                state.seen = Some((key, now));
                return Some(SETTLE);
            }
        }
        if state.view == Some(key) && !pending_lost {
            return None;
        }
        state.view = Some(key);
        self.decide_detail(view, context);
        None
    }

    /// Asks for the tile the resting `view` wants, unless the tile shown or
    /// pending already covers it at that level; drops both when the plotted
    /// level fits the view.
    fn decide_detail(&mut self, view: &CanvasView, context: u64) {
        let Some(want) = self.detail_want(view, context) else {
            let state = &mut self.layers.base.data.detail;
            state.tile = None;
            state.drop_request();
            return;
        };
        let state = &self.layers.base.data.detail;
        let held = |t: &DetailTile| {
            t.level == want.level
                && covers(t.cover[0], want.visible[0])
                && covers(t.cover[1], want.visible[1])
        };
        if state.tile.as_ref().is_some_and(held)
            || state.pending.as_ref().is_some_and(|p| held(&p.tile))
        {
            return;
        }
        self.request_detail(&want);
    }
}
