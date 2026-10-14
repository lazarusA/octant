//! A scale bar on 2D plots of OME-Zarr images: a round physical length
//! along x, in the canvas's bottom-left corner. It is part of the figure, so
//! exports keep it.

#[cfg(test)]
mod tests;

use crate::app::OctantApp;
use crate::data::ome::axes::meters_per_unit;
use crate::plots::PlotType;
use crate::utils::stack_str;

/// Units the bar's length is written in, smallest first: `(meters, symbol)`.
const LENGTH_UNITS: [(f64, &str); 6] = [
    (1e-12, "pm"),
    (1e-9, "nm"),
    (1e-6, "µm"),
    (1e-3, "mm"),
    (1.0, "m"),
    (1e3, "km"),
];
/// Bar length aimed at, as a share of the canvas width, and its bounds in points.
const TARGET_SHARE: f32 = 0.18;
const TARGET_MIN: f32 = 60.0;
const TARGET_MAX: f32 = 160.0;
const MARGIN: f32 = 16.0;
/// Extra lift above the bottom margin, clearing the plot's bottom edge.
const BOTTOM_LIFT: f32 = 12.0;
const BAR_HEIGHT: f32 = 4.0;
const PAD: f32 = 6.0;

/// Meters per data pixel along x of the plotted OME-Zarr level, when its x
/// axis has a length unit.
pub(crate) fn meters_per_pixel(app: &OctantApp) -> Option<f64> {
    let var = app.plotted_variable_info()?;
    let x = app.get_spatial_dim_index(0);
    let unit = var.ome.as_ref()?.image.axes.get(x)?.unit.as_deref()?;
    let meters = meters_per_unit(unit)? * var.ome_scale(x)?;
    (meters.is_finite() && meters > 0.0).then_some(meters)
}

/// The longest round length (1, 2 or 5 times a power of ten) up to
/// `max_meters`, as `(meters, value, unit symbol)` in the largest unit that
/// keeps the value at least 1.
pub(crate) fn nice_length(max_meters: f64) -> Option<(f64, f64, &'static str)> {
    if !max_meters.is_finite() || max_meters <= 0.0 {
        return None;
    }
    let decade = 10f64.powf(max_meters.log10().floor());
    let mantissa = [5.0, 2.0, 1.0]
        .into_iter()
        .find(|m| m * decade <= max_meters * (1.0 + 1e-9))
        .unwrap_or(1.0);
    let meters = mantissa * decade;
    let (unit, symbol) = LENGTH_UNITS
        .iter()
        .rev()
        .find(|(u, _)| meters / u >= 1.0 - 1e-9)
        .copied()
        .unwrap_or(LENGTH_UNITS[0]);
    let value = (meters / unit * 1e6).round() / 1e6;
    Some((meters, value, symbol))
}

/// Draws the scale bar over a 2D heatmap whose x axis has a physical length.
pub fn draw_scale_bar(
    app: &OctantApp,
    ui: &egui::Ui,
    canvas_rect: egui::Rect,
    plot_rect: egui::Rect,
    plot_type: PlotType,
) {
    if !app.layout.show_scale_bar || plot_type != PlotType::Heatmap {
        return;
    }
    let Some(meters_per_px) = meters_per_pixel(app) else {
        return;
    };
    // No bar until data arrives: the placeholder size would mislead.
    let Some((width_px, _)) = app.layers.base.data.shown_dimensions_2d() else {
        return;
    };
    if width_px == 0 || plot_rect.width() <= 0.0 {
        return;
    }
    let meters_per_point = meters_per_px * width_px as f64 / plot_rect.width() as f64;
    let target = (canvas_rect.width() * TARGET_SHARE).clamp(TARGET_MIN, TARGET_MAX);
    let Some((meters, value, symbol)) = nice_length(target as f64 * meters_per_point) else {
        return;
    };
    let bar_w = (meters / meters_per_point) as f32;
    let area = plot_rect.intersect(canvas_rect);
    if bar_w < 1.0 || area.width() < bar_w + 2.0 * MARGIN + 2.0 * PAD {
        return;
    }
    let mut buf = [0u8; 32];
    let label = stack_str(
        &mut buf,
        format_args!(
            "{} {symbol}",
            crate::i18n::number::Num(format_args!("{value}"))
        ),
    );
    paint_bar(ui, canvas_rect, area, bar_w, label);
}

/// Paints a bar `bar_w` points long, labeled `label`, framed in the
/// bottom-left corner of `area`.
fn paint_bar(ui: &egui::Ui, canvas_rect: egui::Rect, area: egui::Rect, bar_w: f32, label: &str) {
    let visuals = ui.visuals();
    let text_color = visuals.text_color();
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(11.0),
        text_color,
    );
    let size = egui::vec2(
        bar_w.max(galley.size().x) + 2.0 * PAD,
        galley.size().y + BAR_HEIGHT + 3.0 * PAD,
    );
    let min = egui::pos2(
        area.left() + MARGIN,
        area.bottom() - MARGIN - BOTTOM_LIFT - size.y,
    );
    let frame = egui::Rect::from_min_size(min, size);
    let painter = ui.painter().with_clip_rect(canvas_rect);
    painter.rect_filled(frame, 4.0, visuals.extreme_bg_color.gamma_multiply(0.75));
    painter.galley(egui::pos2(min.x + PAD, min.y + PAD), galley, text_color);
    let bar = egui::Rect::from_min_size(
        egui::pos2(min.x + PAD, frame.bottom() - PAD - BAR_HEIGHT),
        egui::vec2(bar_w, BAR_HEIGHT),
    );
    painter.rect_filled(bar, 1.0, text_color);
}
