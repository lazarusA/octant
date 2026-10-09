//! One layer's colorbar panel: editable title, the bar, clip triangles and
//! range inputs, placed in its slot of the stack.

use super::bars::{self, BarColors, BarStyle};
use super::handles::{draw_clip_triangles, draw_end_range_inputs};
use super::ticks::format_scientific_tick;
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::plots::PlotType;
use egui::{Pos2, Rect, Vec2};

/// Height a panel takes in the stack, its gap to the next included.
const PANEL_STEP: f32 = 84.0;
/// Height reserved for one panel above the bottom bar.
const PANEL_H: f32 = 88.0;

/// Where the stack sits: centered above the bottom bar, growing upward.
#[derive(Clone, Copy)]
pub struct Placement {
    width: f32,
    bottom_min: Pos2,
}

impl Placement {
    pub fn of(app: &OctantApp, ctx: &egui::Context) -> Self {
        let screen = ctx.input(|i| i.viewport_rect());
        let width = 490.0_f32.min((screen.width() - 32.0).max(180.0));
        let bottom_h = app.bottom_bar_height();
        let bottom_bar_top = if bottom_h > 0.0 {
            screen.max.y - bottom_h
        } else {
            screen.max.y - 4.0
        };
        let bottom_min = Pos2::new(
            screen.center().x - width / 2.0,
            bottom_bar_top - PANEL_H - 8.0,
        );
        Self { width, bottom_min }
    }

    /// The top-left corner and width of the panel in stack slot `slot` (0 at
    /// the bottom).
    pub fn slot(self, slot: usize) -> (Pos2, f32) {
        let y = self.bottom_min.y - slot as f32 * PANEL_STEP;
        (Pos2::new(self.bottom_min.x, y), self.width)
    }
}

/// Draws layer `id`'s colorbar panel at `(min, width)`.
pub fn show(app: &mut OctantApp, ctx: &egui::Context, id: LayerId, (min, width): (Pos2, f32)) {
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
        .fixed_pos(min)
        .show(ctx, |ui| {
            egui::Frame::window(ui.style())
                .fill(fill)
                .stroke(stroke)
                .shadow(shadow)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_width(width - 24.0);
                    ui.vertical_centered(|ui| {
                        title_row(app, ui, id);
                        ui.add_space(3.0);
                        bar_row(app, ui, id, width);
                    });
                });
        });
}

/// The editable title: the custom label, or the variable name and units.
fn title_row(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    let Some(layer) = app.layers.get(id) else {
        return;
    };
    let default_label = layer.default_colorbar_label();
    let mut label = layer.colorbar_label();
    ui.horizontal(|ui| {
        let avail = ui.available_width();
        let text_w = (avail - 40.0).clamp(60.0, 320.0);
        ui.add_space(((avail - text_w) / 2.0).max(0.0));
        let edit = egui::TextEdit::singleline(&mut label)
            .hint_text(&default_label)
            .font(egui::TextStyle::Body)
            .horizontal_align(egui::Align::Center)
            .desired_width(text_w)
            .frame(egui::Frame::NONE);
        let response = ui.add(edit).on_hover_text("Colorbar title. Click to edit.");
        if response.changed() {
            let custom = (!label.trim().is_empty() && label != default_label).then_some(label);
            app.layers.get_or_base_mut(id).color.custom_label = custom;
        }
    });
}

/// The bar with its ticks, clip triangles, range inputs and value tooltip.
fn bar_row(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId, width: f32) {
    let bar_w = (width - 90.0).max(80.0);
    let (widget_rect, response) =
        ui.allocate_exact_size(Vec2::new(bar_w, 38.0), egui::Sense::hover());
    let bar_rect = Rect::from_min_size(widget_rect.min, Vec2::new(bar_w, 13.0));
    let is_3d = matches!(
        app.effective_canvas_plot_type(),
        PlotType::Volume | PlotType::PointCloud
    );
    let Some(layer) = app.layers.get(id) else {
        return;
    };
    let visuals = ui.visuals();
    let colors = BarColors {
        border: visuals.widgets.noninteractive.fg_stroke.color,
        strong_text: visuals.strong_text_color(),
        text: visuals.text_color(),
    };
    let bar = BarStyle {
        color: &layer.color,
        colormap: app.layer_colormap(layer),
    };
    if !is_3d && layer.color.categorical {
        let unique = layer
            .data
            .matrix
            .as_ref()
            .and_then(|m| m.detect_unique_values());
        bars::draw_categorical(ui, bar_rect, &bar, unique, colors);
    } else {
        bars::draw_continuous(ui, bar_rect, &bar, colors);
    }
    let style = &layer.color;
    let hover = response.hover_pos().map(|pos| {
        let t = ((pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
        crate::utils::colormap::unscale_norm_to_value(
            t,
            style.range_min,
            style.range_max,
            style.scale_type,
            style.scale_param,
        )
    });
    let color = &mut app.layers.get_or_base_mut(id).color;
    draw_clip_triangles(ui, bar_rect, color, id);
    draw_end_range_inputs(ui, bar_rect, color);
    if let Some(value) = hover {
        response.on_hover_text(format!("Val: {}", format_scientific_tick(value)));
    }
}
