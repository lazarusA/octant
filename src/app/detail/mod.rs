//! Detail tiles: a heatmap plotted from a coarse level of a multiscale
//! variable (Auto) shows the visible part at the finest level that fits the
//! screen once zooming or panning stops, drawn over the coarse plot (which
//! stays as the placeholder until the tile arrives, and around it).
//!
//! - `geometry.rs`: the plotted and visible windows on level 0's grid, the
//!   finer level's cells covering them, and where those sit on the quad.
//! - `want.rs`: the finer level the resting view asks for; stale tiles.
//! - `tile.rs`: requesting a tile (cache first) and slicing an arrived block into the tile and uploading it to
//!   the base layer's detail renderer, painted after its heatmap.
//!
//! A tile is asked for only once the view rests (`view_interacting` off and
//! the view unchanged for `SETTLE` seconds, so the frames between mouse
//! wheel ticks ask nothing) and playback is stopped, on regular grids
//! without composites; it is kept while it still covers the view at the
//! level the view wants, and dropped when the plot (selection, step)
//! changes. The hover reads it where it is drawn (`ui/hover/detail.rs`).
//!
//! Not yet: overlays. They read their stack's level on the base level's
//! grid and get no tile, so they stay coarse over a sharp base. What it needs:
//! 1. Per-layer refresh: `DetailState` already lives on each layer's
//!    `LayerData`; take a `LayerId` in `refresh_detail`, `detail_context`,
//!    `detail_want`, `plan_detail` and `show_detail` (now the base layer's),
//!    and refresh every drawn overlay after the base in `paint_active_plot`.
//! 2. The same grid: an overlay's tile reads the level of its own stack
//!    matching the base tile's level (`multiscale::level_matching` against
//!    the base tile's level selection) over the base tile's cells, only
//!    while the overlay's alignment is `SameGrid`, so both tiles line up
//!    cell for cell.
//! 3. Drawing needs nothing new: `LayerRenderers::detail` is per layer and
//!    `paint_heatmap` draws a layer's tile right after its heatmap, so the
//!    order stays base, base tile, overlay, overlay tile.
//! 4. Hover: `ui/hover/overlays.rs` `hover_rows` reads each overlay's matrix
//!    at the base cell; it should read the overlay's tile under the pointer
//!    as `ui/hover/detail.rs` does for the base.
//! 5. Cost: one more fetch per drawn overlay for every resting view.

mod geometry;
#[cfg(test)]
mod geometry_tests;
#[cfg(test)]
mod tests;
mod tile;
mod want;

use std::hash::{Hash, Hasher};

use crate::app::OctantApp;
use crate::app::layers::LevelChoice;
use crate::app::pipeline::paint_layer::CanvasView;
use crate::data::BlockCacheKey;
use crate::data::multiscale::Multiscale;
use crate::plots::PlotType;
pub(crate) use geometry::Axis;
use geometry::covers;

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
    /// The view last decided for.
    view: Option<ViewKey>,
    /// The latest view and the egui time it first showed.
    seen: Option<(ViewKey, f64)>,
}

impl DetailState {
    pub fn clear(&mut self) {
        *self = Self::default();
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
    from: usize,
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
    /// before the view counts as resting, for a repaint then.
    pub(crate) fn refresh_detail(
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
            state.pending = None;
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

    /// The plot a tile belongs to, when the base layer can have one: a
    /// heatmap at rest reading a level of a multiscale variable chosen by
    /// Auto, on a regular grid, without composite.
    fn detail_context(&self, plot_type: PlotType) -> Option<u64> {
        let base = &self.layers.base;
        let plotted = base.selection();
        let regular = base.data.matrix.as_ref()?.grid.render_coord_mode() == 0;
        let readable = plot_type == PlotType::Heatmap
            && regular
            && !self.playback.is_playing
            && !base.composite.enabled
            && plotted.level == LevelChoice::Auto
            && plotted.variable_info()?.multiscale.is_some();
        if !readable {
            return None;
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        plotted.metadata_generation.hash(&mut h);
        plotted.variable_idx.hash(&mut h);
        plotted.dim_ranges.hash(&mut h);
        plotted.dim_indices.hash(&mut h);
        self.playback.current_timestep.hash(&mut h);
        base.data.flipped_dims.hash(&mut h);
        Some(h.finish())
    }
}
