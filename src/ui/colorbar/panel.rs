//! One layer's colorbar panel: editable title, the bar, clip triangles,
//! range inputs and its drag and flip controls, placed by its
//! `ColorbarPlacement`.

use super::axis::BarAxis;
use super::bars::{self, BarColors, BarStyle};
use super::handles::{VERTICAL_END_ROOM, draw_clip_triangles, draw_end_range_inputs};
use super::ticks::format_scientific_tick;
use super::{controls, layout};
use crate::app::OctantApp;
use crate::app::layers::{BarOrientation, LayerId};
use crate::plots::PlotType;
use crate::ui::layer_label::LabelEditor;
use crate::utils::colormap::unscale_norm_to_value;
use egui::{Pos2, Rect, Vec2};

/// Thickness of the bar across it.
const BAR_THICKNESS: f32 = 13.0;
/// Height of a horizontal bar's row: the bar, its tick labels and inputs.
const HORIZONTAL_ROW_H: f32 = 38.0;
/// Room left of a vertical bar's center, so its range inputs fit.
const VERTICAL_BAR_X: f32 = 30.0;
/// Room each side of a horizontal panel's title for the controls.
const TITLE_SIDE: f32 = 44.0;

/// Draws layer `id`'s colorbar panel on `canvas`; `left_inset` keeps the
/// Left slot clear of the panels docked on the canvas's left.
pub fn show(app: &mut OctantApp, ctx: &egui::Context, id: LayerId, canvas: Rect, left_inset: f32) {
    let Some(placement) = app.layers.get(id).map(|l| l.colorbar) else {
        return;
    };
    let (point, pivot) = layout::position(&placement, canvas, left_inset);
    let orientation = placement.orientation;
    let width = layout::panel_width(orientation, canvas);
    let bar_len = layout::bar_length(orientation, width, canvas);

    let style = ctx.style_of(ctx.theme());
    let alpha_mult = (1.0 - app.colorbar_transparency).clamp(0.0, 1.0);
    let fill = style.visuals.window_fill.linear_multiply(alpha_mult);
    let stroke_color = style
        .visuals
        .window_stroke
        .color
        .linear_multiply(alpha_mult);
    let stroke = egui::Stroke::new(style.visuals.window_stroke.width, stroke_color);
    let mut shadow = style.visuals.window_shadow;
    shadow.color = shadow.color.linear_multiply(alpha_mult);

    egui::Area::new(egui::Id::new(super::salt("octant_colorbar_overlay", id)))
        .order(egui::Order::Foreground)
        .pivot(pivot)
        .fixed_pos(point)
        .constrain_to(canvas)
        .show(ctx, |ui| {
            let frame = egui::Frame::window(ui.style())
                .fill(fill)
                .stroke(stroke)
                .shadow(shadow)
                .inner_margin(egui::Margin::symmetric(layout::MARGIN_X as i8, 8))
                .show(ui, |ui| {
                    ui.set_width(width - 2.0 * layout::MARGIN_X);
                    match orientation {
                        BarOrientation::Horizontal => ui.vertical_centered(|ui| {
                            title_row(app, ui, id, TITLE_SIDE);
                            ui.add_space(3.0);
                            horizontal_bar(app, ui, id, bar_len);
                        }),
                        BarOrientation::Vertical => ui.vertical(|ui| {
                            // Room for the controls above the title.
                            ui.add_space(controls::CONTROL);
                            title_row(app, ui, id, 0.0);
                            vertical_bar(app, ui, id, bar_len);
                        }),
                    };
                });
            controls::show(app, ui, id, frame.response.rect, (pivot, canvas));
        });
}

/// The editable title: the custom label, or the variable name and units;
/// centered, at most 320 wide, `side` clear on either side.
fn title_row(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId, side: f32) {
    let Some(layer) = app.layers.get(id) else {
        return;
    };
    let mut edited = None;
    ui.horizontal(|ui| {
        let avail = ui.available_width();
        let width = (avail - 2.0 * side).clamp(avail.min(60.0), 320.0);
        ui.add_space(((avail - width) / 2.0).max(0.0));
        let editor = LabelEditor {
            id: egui::Id::new(super::salt("colorbar_title", id)),
            width,
            framed: false,
        };
        edited = editor.show(ui, layer);
    });
    if let (Some(custom), Some(layer)) = (edited, app.layers.get_mut(id)) {
        layer.color.custom_label = custom;
    }
}

/// A horizontal bar `len` long, with ticks and range inputs below it.
fn horizontal_bar(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId, len: f32) {
    let size = Vec2::new(len, HORIZONTAL_ROW_H);
    let (widget_rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let bar_rect = Rect::from_min_size(widget_rect.min, Vec2::new(len, BAR_THICKNESS));
    bar_row(
        app,
        ui,
        id,
        BarAxis::new(bar_rect, BarOrientation::Horizontal),
        response,
    );
}

/// A vertical bar `len` long, ticks on its right, range inputs past its ends.
fn vertical_bar(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId, len: f32) {
    let size = Vec2::new(ui.available_width(), len + 2.0 * VERTICAL_END_ROOM);
    let (widget_rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let center = Pos2::new(widget_rect.min.x + VERTICAL_BAR_X, widget_rect.center().y);
    let bar_rect = Rect::from_center_size(center, Vec2::new(BAR_THICKNESS, len));
    bar_row(
        app,
        ui,
        id,
        BarAxis::new(bar_rect, BarOrientation::Vertical),
        response,
    );
}

/// The bar on `axis` with its ticks, clip triangles, range inputs and the
/// value tooltip of `response`.
fn bar_row(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    axis: BarAxis,
    response: egui::Response,
) {
    let hover = paint_bar(app, ui, id, axis, response.hover_pos());
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let color = &mut layer.color;
    draw_clip_triangles(ui, axis, color, id);
    draw_end_range_inputs(ui, axis, color);
    if let Some(value) = hover {
        response.on_hover_text(format!("Val: {}", format_scientific_tick(value)));
    }
}

/// Paints layer `id`'s bar on `axis`: categorical swatches on 2D plots set
/// to categorical, else a gradient. Returns the value under `hover`.
fn paint_bar(
    app: &OctantApp,
    ui: &egui::Ui,
    id: LayerId,
    axis: BarAxis,
    hover: Option<Pos2>,
) -> Option<f32> {
    let is_3d = matches!(
        app.effective_canvas_plot_type(),
        PlotType::Volume | PlotType::PointCloud
    );
    let layer = app.layers.get(id)?;
    let visuals = ui.visuals();
    let colors = BarColors {
        border: visuals.widgets.noninteractive.fg_stroke.color,
        strong_text: visuals.strong_text_color(),
        text: visuals.text_color(),
    };
    let style = &layer.color;
    let bar = BarStyle {
        color: style,
        colormap: app.layer_colormap(layer),
    };
    if !is_3d && style.categorical {
        let unique = layer
            .data
            .matrix
            .as_ref()
            .and_then(|m| m.detect_unique_values());
        bars::draw_categorical(ui, axis, &bar, unique, colors);
    } else {
        bars::draw_continuous(ui, axis, &bar, colors);
    }
    let t = axis.t_at(hover?);
    let (min, max) = (style.range_min, style.range_max);
    let value = unscale_norm_to_value(t, min, max, style.scale_type, style.scale_param);
    Some(value)
}
