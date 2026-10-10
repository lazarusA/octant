//! How a line plot is colored (`LineColoring`): one custom color (no
//! colorbar), by value, or by series (All Lines Series), where line `i` of
//! `n` takes the colormap at `series_t(i, n, categorical)`, matching
//! `line.wgsl` `series_t`.

use egui::Color32;

use super::app_state::OctantApp;
use crate::plots::PlotType;
use crate::utils::colormap::{orient, registry};

/// What a line plot's colors come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineColoring {
    /// Every line in the custom color.
    Custom,
    /// Each line by its place in the series (All Lines Series).
    Series,
    /// Each point by its value.
    Values,
}

impl LineColoring {
    /// The coloring the Custom Color and All Lines Series options choose.
    pub fn of(custom_color: bool, all_series: bool) -> Self {
        if custom_color {
            Self::Custom
        } else if all_series {
            Self::Series
        } else {
            Self::Values
        }
    }

    /// The coloring of a `plot_type` plot: `None` unless it is a line plot.
    pub fn for_plot(plot_type: PlotType, custom_color: bool, all_series: bool) -> Option<Self> {
        (plot_type == PlotType::Line).then(|| Self::of(custom_color, all_series))
    }
}

impl OctantApp {
    /// The canvas line plot's coloring; `None` when the canvas is no line plot.
    pub fn line_coloring(&self) -> Option<LineColoring> {
        let plot_type = self.effective_canvas_plot_type();
        LineColoring::for_plot(
            plot_type,
            self.plot_configs.line.use_custom_color,
            self.plot_configs.line.all_series,
        )
    }

    /// Whether the canvas line plot draws every line in the custom color.
    pub fn line_custom_colored(&self) -> bool {
        self.line_coloring() == Some(LineColoring::Custom)
    }

    /// Whether the canvas line plot colors each line by its place in the series.
    pub fn line_series_colored(&self) -> bool {
        self.line_coloring() == Some(LineColoring::Series)
    }

    /// The color of series line `line`, as the line shader draws it.
    pub fn line_series_color(&self, line: usize) -> Color32 {
        let base = &self.layers.base;
        let n = self.line_layout().1.line_count;
        let t = series_t(line, n, base.color.categorical);
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
    use super::{LineColoring, series_line_at, series_t};

    #[test]
    fn custom_color_wins_over_all_series() {
        assert_eq!(LineColoring::of(true, true), LineColoring::Custom);
        assert_eq!(LineColoring::of(false, true), LineColoring::Series);
        assert_eq!(LineColoring::of(false, false), LineColoring::Values);
    }

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
