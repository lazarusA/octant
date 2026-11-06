//! The heatmap's visible window by coordinate values (share links' `view`):
//! each end read as `SetRange` reads it, on level 0's grid, then placed on
//! the plotted quad (`detail/geometry.rs` `Axis`, flips included) as the
//! window `render_canvas` shows once the plot draws. A coarse level shows
//! first; the detail tile then reads the finest level the view needs.

use super::{Done, Failure, Outcome};
use crate::app::OctantApp;
use crate::app::state::navigation::PendingView;
use crate::plots::PlotType;

impl OctantApp {
    pub(super) fn set_view(&mut self, x: (&str, &str), y: (&str, &str)) -> Outcome {
        if !self.view_settable() {
            log::warn!("A view is set only on an evenly drawn heatmap of the staged variable");
            return Err(Failure::Refused);
        }
        let axes = self.plotted_axes().ok_or(Failure::NoDataset)?;
        let mut window = [(0.0, 1.0); 2];
        for (k, (axis, (from, to))) in axes.iter().zip([x, y]).enumerate() {
            self.await_coordinates(axis.dim)?;
            let a = self.index_for(axis.dim, from, false)?;
            let b = self.index_for(axis.dim, to, true)?;
            let (t0, t1) = axis.quad_fraction((a.min(b) as f64, a.max(b) as f64 + 1.0));
            window[k] = (f64::from(t0), f64::from(t1));
        }
        let plotted = self.plotted();
        let plot = (plotted.metadata_generation, plotted.variable_idx);
        let window = (window[0], window[1]);
        self.nav.pending_view = Some(PendingView { window, plot });
        Ok(Done::ViewSet)
    }

    fn view_settable(&self) -> bool {
        self.heatmap_evenly_drawn() && !self.is_exploring_unplotted_variable()
    }

    /// Whether the canvas shows a heatmap on an evenly drawn grid, the only
    /// ones whose quad is linear in the index (as detail tiles).
    pub(crate) fn heatmap_evenly_drawn(&self) -> bool {
        let even = self.layers.base.data.matrix.as_ref();
        self.effective_canvas_plot_type() == PlotType::Heatmap
            && even.is_some_and(|m| matches!(m.grid.render_coord_mode(), 0 | 1))
    }
}
