//! The finer level a resting view asks for: the finest level of the plotted
//! multiscale variable with no more cells than screen pixels over the
//! visible window; the plot a tile belongs to (`plot_hash`), and dropping
//! what the plot no longer wants.

use std::hash::{Hash, Hasher};

use super::Want;
use super::geometry::{Axis, pixels_over};
use crate::app::OctantApp;
use crate::app::layers::LevelChoice;
use crate::app::pipeline::paint_layer::CanvasView;
use crate::app::state::navigation::MAX_ZOOM;
use crate::data::multiscale::fit_level;
use crate::plots::PlotType;

impl OctantApp {
    /// The finer level the view asks for over the visible window; `None`
    /// when the plotted level already fits the screen.
    pub(super) fn detail_want(&self, view: &CanvasView, context: u64) -> Option<Want> {
        let plotted = self.plotted();
        let var = plotted.variable_info()?;
        let r = var.multiscale.as_ref()?;
        let (Some(x), Some(y), _) = var.resolve_spatial_dim_indices(&plotted.dim_config) else {
            return None;
        };
        let flipped = |d: usize| {
            let name = var.dimension_names.get(d);
            let flips = &self.layers.base.data.flipped_dims;
            name.is_some_and(|n| flips.iter().any(|f| f.eq_ignore_ascii_case(n)))
        };
        let set = &r.set;
        let axis =
            |d: usize| Axis::plotted(set, r.level, d, *plotted.dim_ranges.get(d)?, flipped(d));
        let axes = [axis(x)?, axis(y)?];
        let (u, v) = crate::data::ViewportResampler::compute_visible_data_bounds(
            view.pan,
            view.zoom,
            view.aspect_scale,
        );
        let visible = [axes[0].visible(u), axes[1].visible(v)];
        let [px_w, px_h] = self.layout.canvas_px;
        let px = [pixels_over(px_w, u), pixels_over(px_h, v)];
        let window = level0_window(plotted, set, r.level, [x, y], visible);
        let demand = crate::app::controllers::controller_for(PlotType::Heatmap).resolution_demand(
            var,
            &plotted.dim_config,
            px,
        );
        let level = fit_level(set, &window, &demand);
        (level < r.level).then(|| Want {
            set: set.clone(),
            level,
            context,
            axes,
            visible,
        })
    }

    /// How far the heatmap may zoom in: over a level of a multiscale variable
    /// read by Auto (which detail tiles replace with finer ones), until each
    /// level-0 cell spans `PIXELS_PER_CELL` screen pixels (the plotted level
    /// has about a pixel per cell when the whole window shows), never less
    /// than [`MAX_ZOOM`].
    pub(crate) fn heatmap_max_zoom(&self) -> f32 {
        let plotted = self.plotted();
        let factor = plotted
            .variable_info()
            .filter(|_| plotted.level == LevelChoice::Auto)
            .and_then(|var| {
                let r = var.multiscale.as_ref()?;
                let (Some(x), Some(y), _) = var.resolve_spatial_dim_indices(&plotted.dim_config)
                else {
                    return None;
                };
                let level = r.info()?;
                Some(level.factor.get(x)?.max(*level.factor.get(y)?))
            });
        max_zoom_over(factor.unwrap_or(1.0))
    }

    /// Whether `layer`'s tile fetch was lost (aborted: neither pending nor
    /// cached).
    pub(super) fn fetch_lost(&self, layer: &crate::app::layers::Layer) -> bool {
        layer.data.detail.pending.as_ref().is_some_and(|p| {
            !self.block_prefetcher.is_pending(&p.key) && !self.block_cache.contains(&p.key)
        })
    }

    /// The plot a tile belongs to, when the base layer can have one: a
    /// heatmap at rest reading a level of a multiscale variable chosen by
    /// Auto, on a regular grid, without composite.
    pub(super) fn detail_context(&self, plot_type: PlotType) -> Option<u64> {
        let base = &self.layers.base;
        let plotted = base.selection();
        // Global (0) and regional (1) regular grids draw cells evenly over
        // the quad, as tiles do; coordinate lookups (2) and HEALPix don't.
        let regular = matches!(base.data.matrix.as_ref()?.grid.render_coord_mode(), 0 | 1);
        let readable = plot_type == PlotType::Heatmap
            && regular
            && !self.playback.is_playing
            && !base.composite.enabled
            && plotted.level == LevelChoice::Auto
            && plotted.variable_info()?.multiscale.is_some();
        let step = self.playback.current_timestep;
        readable.then(|| plot_hash(plotted, step, &base.data.flipped_dims))
    }

    /// Drops what no longer belongs to the plot (`context`, `None` when the
    /// base layer can have no tile) and a pending fetch that was lost (an
    /// abort); whether one was lost.
    pub(super) fn drop_stale_detail(&mut self, context: Option<u64>) -> bool {
        let lost = self.fetch_lost(&self.layers.base);
        let state = &mut self.layers.base.data.detail;
        let Some(context) = context else {
            state.clear();
            return lost;
        };
        let asked = state
            .pending
            .as_ref()
            .or(state.refused.as_ref().map(|r| &r.0));
        if lost || asked.is_some_and(|p| p.tile.context != context) {
            state.drop_request();
        }
        if state.tile.as_ref().is_some_and(|t| t.context != context) {
            state.tile = None;
        }
        lost
    }
}

/// Screen pixels a level-0 cell spans at the deepest zoom.
const PIXELS_PER_CELL: f64 = 4.0;

/// How far the heatmap may zoom in over a level `factor` times coarser than
/// level 0 (f32 keeps the pan within a pixel up to about 10^4).
pub(super) fn max_zoom_over(factor: f64) -> f32 {
    ((factor * PIXELS_PER_CELL) as f32).clamp(MAX_ZOOM, 1e4)
}

/// The level-0 window the view shows: the `visible` positions along the x
/// and y dimensions `xy`, the plotted window (read at `level`) along the
/// others.
fn level0_window(
    plotted: &crate::app::layers::VariableSelection,
    set: &crate::data::multiscale::Multiscale,
    level: usize,
    xy: [usize; 2],
    visible: [(f64, f64); 2],
) -> Vec<std::ops::Range<u64>> {
    (0..set.dims.len())
        .map(|d| match xy.iter().position(|&s| s == d) {
            Some(i) => {
                let (lo, hi) = visible[i];
                lo.floor().max(0.0) as u64..hi.ceil().max(lo.floor() + 1.0) as u64
            }
            None => {
                let (s, e) = plotted.dim_ranges.get(d).copied().unwrap_or((0, 0));
                set.to_base(level, d, s as u64..e as u64 + 1)
                    .unwrap_or(0..1)
            }
        })
        .collect()
}

/// What a plotted selection shows at `step` with `flips`: a tile is kept
/// while it stays the same.
pub(super) fn plot_hash(
    sel: &crate::app::layers::VariableSelection,
    step: usize,
    flips: &[String],
) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    sel.metadata_generation.hash(&mut h);
    sel.variable_idx.hash(&mut h);
    sel.dim_ranges.hash(&mut h);
    sel.dim_indices.hash(&mut h);
    step.hash(&mut h);
    flips.hash(&mut h);
    h.finish()
}
