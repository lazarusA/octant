//! The finer level a resting view asks for: the finest level of the plotted
//! multiscale variable with no more cells than screen pixels over the
//! visible window.

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
        let window: Vec<_> = (0..var.shape.len())
            .map(|d| match [x, y].iter().position(|&s| s == d) {
                Some(i) => {
                    let (lo, hi) = visible[i];
                    lo.floor().max(0.0) as u64..hi.ceil().max(lo.floor() + 1.0) as u64
                }
                None => {
                    let (s, e) = plotted.dim_ranges.get(d).copied().unwrap_or((0, 0));
                    set.to_base(r.level, d, s as u64..e as u64 + 1)
                        .unwrap_or(0..1)
                }
            })
            .collect();
        let demand = crate::app::controllers::controller_for(PlotType::Heatmap).resolution_demand(
            var,
            &plotted.dim_config,
            px,
        );
        let level = fit_level(set, &window, &demand);
        (level < r.level).then(|| Want {
            set: set.clone(),
            from: r.level,
            level,
            context,
            axes,
            visible,
        })
    }
}
