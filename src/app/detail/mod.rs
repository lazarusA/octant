//! Detail tiles: a heatmap plotted from a coarse level of a multiscale
//! variable (Auto) shows the visible part at the finest level that fits the
//! screen once zooming or panning stops, drawn over the coarse plot (which
//! stays as the placeholder until the tile arrives, and around it).
//!
//! - `geometry.rs`: the plotted and visible windows on level 0's grid, the
//!   finer level's cells covering them, and where those sit on the quad.
//! - `want.rs`: the finer level the resting view asks for.
//! - `tile.rs`: slicing an arrived block into the tile and uploading it to
//!   the base layer's detail renderer, painted after its heatmap.
//!
//! A tile is asked for only while the view rests (`view_interacting` off)
//! and playback is stopped, on regular grids without composites; it is
//! kept while it still covers the view at the level the view wants, and
//! dropped when the plot (selection, step) changes.

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
use crate::data::multiscale::Multiscale;
use crate::data::{BlockCacheKey, BlockRequest};
use crate::plots::PlotType;
pub(crate) use geometry::Axis;
use geometry::covers;

/// Share of the visible span a tile reads beyond each side, so small pans
/// keep it.
const MARGIN: f64 = 0.25;

/// The base layer's detail tile, the one being fetched, and the view the
/// last decision was made for.
#[derive(Default)]
pub struct DetailState {
    pub tile: Option<DetailTile>,
    pub pending: Option<PendingDetail>,
    view: Option<ViewKey>,
}

impl DetailState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// What a tile shows and where.
#[derive(Debug, Clone, PartialEq)]
pub struct DetailTile {
    pub level: usize,
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
    /// when the plot changed, and once the view rests asks for the visible
    /// window at a finer level when the plotted one is coarser than the screen.
    pub(crate) fn refresh_detail(&mut self, plot_type: PlotType, view: &CanvasView) {
        let context = self.detail_context(plot_type);
        let pending_lost = self
            .layers
            .base
            .data
            .detail
            .pending
            .as_ref()
            .is_some_and(|p| {
                !self.block_prefetcher.is_pending(&p.key) && !self.block_cache.contains(&p.key)
            });
        let state = &mut self.layers.base.data.detail;
        let Some(context) = context else {
            state.clear();
            return;
        };
        if pending_lost
            || state
                .pending
                .as_ref()
                .is_some_and(|p| p.tile.context != context)
        {
            state.pending = None;
        }
        if state.tile.as_ref().is_some_and(|t| t.context != context) {
            state.tile = None;
        }
        if self.nav.view_interacting {
            return;
        }
        let key = ViewKey {
            pan: view.pan,
            zoom: view.zoom,
            aspect: view.aspect_scale,
            px: self.layout.canvas_px,
            context,
        };
        if state.view == Some(key) && !pending_lost {
            return;
        }
        state.view = Some(key);
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

    /// Asks for `want`'s tile: shown at once from the block cache, else
    /// fetched and shown when it arrives (`accept_detail_block`).
    fn request_detail(&mut self, want: &Want) {
        let Some((pending, slice, handle)) = self.plan_detail(want) else {
            return;
        };
        if let Some(block) = self.block_cache.get(&pending.key) {
            self.show_detail(pending, &block);
            return;
        }
        self.layers.base.data.detail.pending = Some(pending);
        self.block_prefetcher
            .request(BlockRequest::new(handle, slice), &self.block_cache);
    }

    /// Takes a fetch result when it is the pending tile's (shown and cached,
    /// or dropped with a log when the fetch failed); whether it was.
    pub(crate) fn take_detail_result(
        &mut self,
        key: &BlockCacheKey,
        result: &Result<crate::data::OctantBlock, String>,
    ) -> bool {
        let state = &mut self.layers.base.data.detail;
        if state.pending.as_ref().is_none_or(|p| &p.key != key) {
            return false;
        }
        let Some(pending) = state.pending.take() else {
            return false;
        };
        match result {
            Ok(block) => {
                self.block_cache.put(key.clone(), block.clone());
                self.show_detail(pending, block);
            }
            Err(e) => log::warn!("Detail tile of '{}' failed: {e}", key.variable_name),
        }
        true
    }
}
