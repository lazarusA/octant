//! The finer level a resting view asks for: the finest level of the plotted
//! multiscale variable with no more cells than screen pixels over the
//! visible window; and dropping what the plot no longer wants.

use super::Want;
use super::geometry::{Axis, pixels_over};
use crate::app::OctantApp;
use crate::app::pipeline::paint_layer::CanvasView;
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
