//! How a line plot is colored: one custom color (no colorbar), by value, or
//! by series (All Lines Series), where line `i` of `n` takes the colormap at
//! `series_t(i, n, categorical)`, matching `line.wgsl` `series_t`.

use egui::Color32;

use super::app_state::OctantApp;
use crate::plots::PlotType;
use crate::utils::colormap::{orient, registry};

impl OctantApp {
    /// Whether the canvas line plot draws every line in the custom color.
    pub fn line_custom_colored(&self) -> bool {
        self.effective_canvas_plot_type() == PlotType::Line && self.line_use_custom_color
    }

    /// Whether the canvas line plot colors each line by its place in the
    /// series (All Lines Series without a custom color).
    pub fn line_series_colored(&self) -> bool {
        self.effective_canvas_plot_type() == PlotType::Line
            && self.line_plot_all_series
            && !self.line_use_custom_color
    }

    /// Lines in `get_line_profile_payload`'s series, without building it.
    pub fn line_series_count(&self) -> usize {
        if !self.line_plot_all_series {
            return 1;
        }
        if self.line_profile_dim_idx == 2
            && let Some(vdata) = &self.layers.base.data.volume
            && vdata.depth > 1
        {
            vdata.width * vdata.height
        } else if let Some(matrix) = &self.layers.base.data.matrix {
            match self.line_profile_dim_idx {
                0 => matrix.height,
                _ => matrix.width,
            }
        } else {
            0
        }
    }

    /// The color of series line `line`, as the line shader draws it.
    pub fn line_series_color(&self, line: usize) -> Color32 {
        let base = &self.layers.base;
        let t = series_t(line, self.line_series_count(), base.color.categorical);
        registry::sample(self.layer_colormap(base), orient(t, base.color.reversed))
    }
}

/// Colormap position of line `i` of `n`: bin centers when `categorical` (one
/// color per line), else spread so the first and last lines take the ends.
pub fn series_t(i: usize, n: usize, categorical: bool) -> f32 {
    if categorical {
        (i as f32 + 0.5) / n.max(1) as f32
    } else {
        i as f32 / n.saturating_sub(1).max(1) as f32
    }
}

/// The line of `n` drawn at colormap position `t` (the inverse of `series_t`).
pub fn series_line_at(t: f32, n: usize, categorical: bool) -> usize {
    let last = n.saturating_sub(1);
    let t = t.clamp(0.0, 1.0);
    let line = if categorical {
        (t * n as f32).floor()
    } else {
        (t * last as f32).round()
    };
    (line as usize).min(last)
}

#[cfg(test)]
mod tests {
    use super::{series_line_at, series_t};

    #[test]
    fn continuous_series_span_the_colormap() {
        assert_eq!(series_t(0, 5, false), 0.0);
        assert_eq!(series_t(4, 5, false), 1.0);
        assert_eq!(series_t(0, 1, false), 0.0);
    }

    #[test]
    fn categorical_series_take_bin_centers() {
        assert_eq!(series_t(0, 4, true), 0.125);
        assert_eq!(series_t(3, 4, true), 0.875);
    }

    #[test]
    fn line_at_inverts_series_t() {
        for categorical in [false, true] {
            for i in 0..7 {
                let t = series_t(i, 7, categorical);
                assert_eq!(series_line_at(t, 7, categorical), i, "{categorical}");
            }
        }
        assert_eq!(series_line_at(1.0, 4, true), 3);
        assert_eq!(series_line_at(0.5, 0, false), 0);
    }
}
