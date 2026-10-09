//! The bar of a line plot colored by series (All Lines Series): the colormap
//! over the lines, one swatch per line when categorical, labeled with the
//! coordinate of the dimension the lines run across (`series_field`, as the
//! hover card shows it).

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use egui::{Mesh, Shape, Stroke};

use super::axis::BarAxis;
use super::bars::{self, BarColors};
use crate::app::{OctantApp, series_line_at, series_t};
use crate::ui::hover::entries_1d::series_field;
use crate::utils::colormap::{orient, registry};

/// Lines labeled at most, spread evenly from the first to the last.
const MAX_LABELS: usize = 5;
/// Swatches drawn at most: past this many lines a swatch is under a pixel,
/// and a gradient looks the same.
const MAX_SWATCHES: usize = 256;
/// Narrowest swatch that gets dividers.
const MIN_DIVIDED: f32 = 6.0;
/// Segments of the gradient drawn for continuous or very long series.
const SEGMENTS: usize = 128;

/// A labeled line: its place on the bar and its coordinate.
#[derive(Clone)]
struct SeriesTick {
    t: f32,
    label: Arc<str>,
}

/// Draws the base layer's series bar on `axis` in colormap row `colormap`.
pub(super) fn draw(
    app: &OctantApp,
    ui: &egui::Ui,
    axis: BarAxis,
    colormap: u32,
    colors: BarColors,
) {
    let style = &app.layers.base.color;
    let (n, categorical) = (app.line_series_count(), style.categorical);
    let color_at = |t: f32| registry::sample(colormap, orient(t, style.reversed));
    let swatches = categorical && n <= MAX_SWATCHES;
    let mut mesh = Mesh::default();
    if swatches {
        for i in 0..n {
            let color = color_at(series_t(i, n, true));
            let quad = axis.quad(i as f32 / n as f32, (i + 1) as f32 / n as f32);
            bars::push_quad(&mut mesh, quad, [color, color]);
        }
    } else {
        for i in 0..SEGMENTS {
            let (t0, t1) = (i as f32 / SEGMENTS as f32, (i + 1) as f32 / SEGMENTS as f32);
            bars::push_quad(&mut mesh, axis.quad(t0, t1), [color_at(t0), color_at(t1)]);
        }
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
    for tick in ticks(app, ui.ctx(), n, categorical).iter() {
        bars::major_tick(ui, axis, tick.t, Some(&tick.label), colors.strong_text);
    }
}

/// The hover text at bar position `t`: the coordinate of the line drawn there.
pub(super) fn hover_text(app: &OctantApp, t: f32) -> String {
    let n = app.line_series_count();
    let line = series_line_at(t, n, app.layers.base.color.categorical);
    let plotted = app.plotted();
    let (field, _) = series_field(
        app,
        plotted.metadata.as_ref(),
        plotted.variable_info(),
        line,
        n,
    );
    format!("{}: {}", field.label, field.value)
}

/// The labeled lines, formatted only when the series or its coordinates
/// change (kept in egui temp memory).
fn ticks(app: &OctantApp, ctx: &egui::Context, n: usize, categorical: bool) -> Arc<[SeriesTick]> {
    let plotted = app.plotted();
    let base = &app.layers.base;
    let mut hasher = DefaultHasher::new();
    (
        (plotted.metadata_generation, app.coordinates_revision),
        (plotted.store_target.as_str(), plotted.variable_idx),
        &plotted.dim_ranges,
        base.load.slice_request.as_ref().map(|r| &r.selections),
        &base.data.flipped_dims,
        (app.line_profile_dim_idx, n, categorical),
    )
        .hash(&mut hasher);
    let key = hasher.finish();
    let id = egui::Id::new("colorbar_series_ticks");
    if let Some((cached, ticks)) = ctx.data(|d| d.get_temp::<(u64, Arc<[SeriesTick]>)>(id))
        && cached == key
    {
        return ticks;
    }
    let ticks = labeled_lines(n)
        .map(|line| {
            let (field, _) = series_field(
                app,
                plotted.metadata.as_ref(),
                plotted.variable_info(),
                line,
                n,
            );
            let t = series_t(line, n, categorical);
            SeriesTick {
                t,
                label: field.value.into(),
            }
        })
        .collect::<Arc<[_]>>();
    ctx.data_mut(|d| d.insert_temp(id, (key, ticks.clone())));
    ticks
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
