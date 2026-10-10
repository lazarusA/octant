//! The bar of a line plot colored by series (All Lines Series): the colormap
//! over the lines, one swatch per line when categorical, labeled with the
//! coordinate of the dimension the lines run across (`series_field`, as the
//! hover card shows it).

use std::sync::Arc;

use egui::{Mesh, Rangef, Shape, Stroke};

use super::axis::BarAxis;
use super::bars::{self, BarColors};
use crate::app::{OctantApp, series_line_at, series_t};
use crate::ui::hover::entries_1d::{series_dim, series_field};
use crate::ui::temp_cache::{cached, hash_key};
use crate::utils::colormap::{orient, registry};

/// Lines labeled at most, spread evenly from the first to the last.
const MAX_LABELS: usize = 5;
/// Swatches drawn at most: past this many lines a swatch is under a pixel,
/// and a gradient looks the same.
const MAX_SWATCHES: usize = 256;
/// Narrowest swatch that gets dividers.
const MIN_DIVIDED: f32 = 6.0;
/// Least room between two labels along the bar.
const LABEL_SPACING: f32 = 6.0;

/// What the base layer's series bar shows this frame.
pub(super) struct Series {
    /// Lines in the plot.
    n: usize,
    categorical: bool,
    reversed: bool,
    /// Identifies what the coordinate labels read (`labels_key`).
    labels_key: u64,
}

impl Series {
    pub(super) fn of(app: &OctantApp) -> Self {
        let style = &app.layers.base.color;
        let n = app.line_layout().1.line_count;
        Self {
            n,
            categorical: style.categorical,
            reversed: style.reversed,
            labels_key: labels_key(app, n),
        }
    }
}

/// A labeled line: its place on the bar and its coordinate.
#[derive(Clone)]
struct SeriesTick {
    t: f32,
    label: Arc<str>,
}

/// Draws `series` on `axis` in colormap row `colormap`.
pub(super) fn draw(
    app: &OctantApp,
    ui: &egui::Ui,
    axis: BarAxis,
    series: &Series,
    colormap: u32,
    colors: BarColors,
) {
    let n = series.n;
    let color_at = |t: f32| registry::sample(colormap, orient(t, series.reversed));
    let swatches = series.categorical && n <= MAX_SWATCHES;
    let mut mesh = Mesh::default();
    if swatches {
        for i in 0..n {
            let color = color_at(series_t(i, n, true));
            let quad = axis.quad(i as f32 / n as f32, (i + 1) as f32 / n as f32);
            bars::push_quad(&mut mesh, quad, [color, color]);
        }
    } else {
        bars::push_gradient(&mut mesh, axis, registry::is_stepped(colormap), color_at);
    }
    let painter = ui.painter();
    painter.add(Shape::mesh(mesh));
    let along = axis.rect.width().max(axis.rect.height());
    if swatches && along / n.max(1) as f32 >= MIN_DIVIDED {
        let divider = Stroke::new(1.0, egui::Color32::from_black_alpha(120));
        for i in 1..n {
            painter.line_segment(axis.across(i as f32 / n as f32), divider);
        }
    }
    let border = Stroke::new(1.0, colors.border);
    painter.rect_stroke(axis.rect, 0.0, border, egui::StrokeKind::Middle);
    let ticks = ticks(app, ui.ctx(), series);
    for tick in ticks.iter() {
        bars::major_tick(ui, axis, tick.t, None, colors.strong_text);
    }
    draw_labels(ui, axis, &ticks, colors.strong_text);
}

/// Draws the labels of `ticks` that fit: the first and last first, then the
/// others in order, each only when it keeps clear of those already drawn.
fn draw_labels(ui: &egui::Ui, axis: BarAxis, ticks: &[SeriesTick], color: egui::Color32) {
    let last = ticks.len().saturating_sub(1);
    let order = [0, last].into_iter().chain(1..last).take(ticks.len());
    let mut taken = [Rangef::NOTHING; MAX_LABELS];
    let mut kept = 0;
    for tick in order.filter_map(|i| ticks.get(i)) {
        let text = tick.label.to_string();
        let galley = ui.painter().layout_no_wrap(text, bars::label_font(), color);
        let (pos, align) = axis.label(tick.t, bars::LABEL_GAP);
        let rect = align.anchor_size(pos, galley.size());
        let span = if axis.rect.width() >= axis.rect.height() {
            rect.x_range()
        } else {
            rect.y_range()
        };
        let apart =
            |t: &Rangef| span.max + LABEL_SPACING <= t.min || t.max + LABEL_SPACING <= span.min;
        if kept < MAX_LABELS && taken[..kept].iter().all(apart) {
            taken[kept] = span;
            kept += 1;
            ui.painter().galley(rect.min, galley, color);
        }
    }
}

/// Shows the coordinate of the line drawn at bar position `t` as tooltip,
/// formatted only when the hovered line or the labels' inputs change.
pub(super) fn show_hover_text(app: &OctantApp, response: egui::Response, series: &Series, t: f32) {
    let line = series_line_at(t, series.n, series.categorical);
    let id = egui::Id::new("colorbar_series_hover");
    let text: Arc<str> = cached(&response.ctx, id, (series.labels_key, line), || {
        let plotted = app.plotted();
        let meta = plotted.metadata.as_ref();
        let (field, _) = series_field(app, meta, plotted.variable_info(), line, series.n);
        format!("{}: {}", field.label, field.value).into()
    });
    response.on_hover_text(&*text);
}

/// Identifies what the coordinate labels of `n` series lines read: the
/// dataset and its coordinates, and where the series dimension's window sits
/// (its roles, range and flip).
fn labels_key(app: &OctantApp, n: usize) -> u64 {
    let plotted = app.plotted();
    hash_key((
        (plotted.metadata_generation, app.coordinates_revision),
        (plotted.store_target.as_str(), plotted.variable_idx),
        series_dim(app, plotted.variable_info()),
        n,
    ))
}

/// The labeled lines, formatted only when the series or its coordinates
/// change (kept in egui temp memory).
fn ticks(app: &OctantApp, ctx: &egui::Context, series: &Series) -> Arc<[SeriesTick]> {
    let (n, categorical) = (series.n, series.categorical);
    let id = egui::Id::new("colorbar_series_ticks");
    cached(ctx, id, (series.labels_key, categorical), || {
        let plotted = app.plotted();
        let meta = plotted.metadata.as_ref();
        labeled_lines(n)
            .map(|line| {
                let (field, _) = series_field(app, meta, plotted.variable_info(), line, n);
                SeriesTick {
                    t: series_t(line, n, categorical),
                    label: field.value.into(),
                }
            })
            .collect()
    })
}

/// Up to `MAX_LABELS` lines of `n`, evenly spread from the first to the last.
fn labeled_lines(n: usize) -> impl Iterator<Item = usize> {
    let last = n.saturating_sub(1);
    let steps = (MAX_LABELS - 1).min(last);
    (0..=steps)
        .filter(move |_| n > 0)
        .map(move |k| (k * last + steps / 2) / steps.max(1))
}

#[cfg(test)]
mod tests {
    use super::labeled_lines;

    #[test]
    fn labels_spread_from_first_to_last_line() {
        assert_eq!(labeled_lines(0).collect::<Vec<_>>(), Vec::<usize>::new());
        assert_eq!(labeled_lines(1).collect::<Vec<_>>(), [0]);
        assert_eq!(labeled_lines(3).collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(labeled_lines(9).collect::<Vec<_>>(), [0, 2, 4, 6, 8]);
        assert_eq!(labeled_lines(100).collect::<Vec<_>>(), [0, 25, 50, 74, 99]);
    }
}
