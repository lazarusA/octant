//! One layer's colorbar panel: editable title, the bar (a click opens the
//! colormap picker), clip triangles, range inputs and its controls, placed
//! by its `ColorbarPlacement`, and its Color window.

use super::axis::BarAxis;
use super::bars::{self, BarColors, BarStyle};
use super::handles::{VERTICAL_END_ROOM, draw_clip_triangles, draw_end_range_inputs};
use super::ticks::{ScientificTick, TICK_BUF};
use super::{color_menu, controls, layout, series};
use crate::app::OctantApp;
use crate::app::layers::{BarOrientation, LayerId};
use crate::i18n::{ColormapKey as K, tr};
use crate::ui::layer_label::LabelEditor;
use crate::utils::colormap::unscale_norm_to_value;
use egui::{Pos2, Rect, Vec2};

/// Thickness of the bar across it.
const BAR_THICKNESS: f32 = 13.0;
/// Height of a horizontal bar's row: the bar, its tick labels and inputs.
const HORIZONTAL_ROW_H: f32 = 38.0;
/// Room left of a vertical bar's center, so its range inputs fit.
const VERTICAL_BAR_X: f32 = 30.0;
/// Vertical margin inside a panel's frame.
const MARGIN_Y: i8 = 8;

/// Draws layer `id`'s colorbar panel on `canvas`.
pub fn show(app: &mut OctantApp, ctx: &egui::Context, id: LayerId, canvas: Rect) {
    let Some(placement) = app.layers.get(id).map(|l| l.colorbar) else {
        return;
    };
    let (point, pivot) = layout::position(&placement, canvas);
    let orientation = placement.orientation;
    let width = layout::panel_width(orientation, canvas);
    let bar_len = layout::bar_length(orientation, width, canvas);

    let frame = panel_frame(ctx, app.layout.colorbar_transparency);
    let shown = egui::Area::new(super::area_id(id))
        .order(egui::Order::Middle)
        .pivot(pivot)
        .fixed_pos(point)
        .constrain_to(canvas)
        .show(ctx, |ui| {
            let shown = frame.show(ui, |ui| {
                ui.set_width(width - 2.0 * f32::from(layout::MARGIN_X));
                contents(app, ui, id, orientation, bar_len);
            });
            controls::show(app, ui, id, shown.response.rect, (pivot, canvas));
        });
    color_menu::show(app, ctx, id, (shown.response.rect, orientation), canvas);
}

/// The window frame of a colorbar panel, its fill, outline and shadow
/// faded by the colorbar `transparency`.
fn panel_frame(ctx: &egui::Context, transparency: f32) -> egui::Frame {
    let style = ctx.style_of(ctx.theme());
    let visuals = &style.visuals;
    let alpha = (1.0 - transparency).clamp(0.0, 1.0);
    let stroke_color = visuals.window_stroke.color.linear_multiply(alpha);
    let mut shadow = visuals.window_shadow;
    shadow.color = shadow.color.linear_multiply(alpha);
    egui::Frame::window(&style)
        .fill(visuals.window_fill.linear_multiply(alpha))
        .stroke(egui::Stroke::new(visuals.window_stroke.width, stroke_color))
        .shadow(shadow)
        .inner_margin(egui::Margin::symmetric(layout::MARGIN_X, MARGIN_Y))
}

/// The title over a horizontal bar `bar_len` long, or the controls' room,
/// the title and a vertical bar.
fn contents(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    orientation: BarOrientation,
    bar_len: f32,
) {
    match orientation {
        BarOrientation::Horizontal => ui.vertical_centered(|ui| {
            // The controls sit beside the title, never over the bar.
            let below_controls = controls::REACH - f32::from(MARGIN_Y);
            title_row(app, ui, id, controls::SIDE_ROOM, below_controls);
            ui.add_space(3.0);
            horizontal_bar(app, ui, id, bar_len);
        }),
        BarOrientation::Vertical => ui.vertical(|ui| {
            // Room for the controls above the title.
            ui.add_space(controls::CONTROL);
            title_row(app, ui, id, 0.0, 0.0);
            vertical_bar(app, ui, id, bar_len);
        }),
    };
}

/// The editable title: the custom label, or the variable name and units;
/// centered, at most 320 wide, `side` clear on either side, at least
/// `min_height` tall.
fn title_row(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId, side: f32, min_height: f32) {
    let Some(layer) = app.layers.get(id) else {
        return;
    };
    let mut edited = None;
    ui.horizontal(|ui| {
        ui.set_min_height(min_height);
        let avail = ui.available_width();
        let width = (avail - 2.0 * side).clamp(avail.min(60.0), 320.0);
        ui.add_space(((avail - width) / 2.0).max(0.0));
        let editor = LabelEditor {
            id: egui::Id::unique(super::salt("colorbar_title", id)),
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
/// value tooltip of `response` (the bar's row); the bar is outlined while
/// hovered and a click on it opens the layer's colormap picker.
fn bar_row(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    axis: BarAxis,
    response: egui::Response,
) {
    let bar = ui
        .interact(axis.rect, super::bar_id(id), egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    // Before the bar paints, so a colormap hovered in the picker previews
    // on it this frame.
    crate::ui::colormap::show_picker_popup(app, &bar, id);
    let on_bar = bar.hovered();
    let outlined = on_bar || (bar.has_focus() && crate::ui::keyboard::focus::visible(ui.ctx()));
    let hovered = if on_bar { bar } else { response };
    if id == LayerId::BASE && app.line_series_colored() {
        series_bar_row(app, ui, axis, hovered);
        axis.outline(ui, outlined);
        return;
    }
    let hover = paint_bar(app, ui, id, axis, hovered.hover_pos());
    axis.outline(ui, outlined);
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let color = &mut layer.color;
    draw_clip_triangles(ui, axis, color, id);
    draw_end_range_inputs(ui, axis, color);
    if let Some(value) = hover {
        value_tooltip(hovered, value, on_bar);
    }
}

/// The hovered `value` as `response`'s tooltip, with the click hint while
/// over the bar (`on_bar`).
fn value_tooltip(response: egui::Response, value: f32, on_bar: bool) {
    let mut buf = [0u8; TICK_BUF + 48];
    let text = crate::i18n::fill(&mut buf, tr(K::HoveredValue), &[&ScientificTick(value)]);
    response.on_hover_ui(|ui| {
        ui.label(text);
        if on_bar {
            ui.label(
                egui::RichText::new(tr(K::ChangeColormapHover))
                    .small()
                    .weak(),
            );
        }
    });
}

/// The series bar of a line plot colored by series: no range inputs or clip
/// triangles (the colors follow the lines, not the values), and the hovered
/// line's coordinate as tooltip.
fn series_bar_row(app: &OctantApp, ui: &egui::Ui, axis: BarAxis, response: egui::Response) {
    let colors = BarColors::from_visuals(ui.visuals());
    let series = series::Series::of(app);
    let colormap = app.layer_colormap(&app.layers.base);
    series::draw(app, ui, axis, &series, colormap, colors);
    if let Some(pos) = response.hover_pos() {
        series::show_hover_text(app, response, &series, axis.t_at(pos));
    }
}

/// Paints layer `id`'s bar on `axis`: categorical swatches when the layer is
/// set to categorical and its plot draws categories, else a gradient. Returns the value under `hover`.
fn paint_bar(
    app: &OctantApp,
    ui: &egui::Ui,
    id: LayerId,
    axis: BarAxis,
    hover: Option<Pos2>,
) -> Option<f32> {
    let layer = app.layers.get(id)?;
    let colors = BarColors::from_visuals(ui.visuals());
    let style = &layer.color;
    let bar = BarStyle {
        color: style,
        colormap: app.layer_colormap(layer),
    };
    if style.categorical && app.effective_canvas_plot_type().draws_categories() {
        let unique = layer.data.matrix.as_ref().and_then(|m| m.unique_values());
        bars::draw_categorical(ui, axis, &bar, unique, colors);
    } else {
        bars::draw_continuous(ui, axis, &bar, colors);
    }
    let t = axis.t_at(hover?);
    let (min, max) = (style.range_min, style.range_max);
    let value = unscale_norm_to_value(t, min, max, style.scale_type, style.scale_param);
    Some(value)
}
