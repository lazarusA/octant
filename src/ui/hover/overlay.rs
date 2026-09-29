//! Glassmorphic tooltip card and color swatch rendering.

use crate::plots::PlotType;
use crate::ui::hover::callout::draw_leader_callout;
use egui::{Color32, Pos2, Rect, Stroke, StrokeKind};

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_tooltip_card(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    rect: Rect,
    hover_pos: Pos2,
    target_pos: Option<Pos2>,
    plot_type: PlotType,
    var_name: &str,
    raw_val: f32,
    units_str: &str,
    dim_entries: &[String],
    is_rgb: bool,
    pixel_color: Color32,
) {
    let tooltip_w = 210.0;
    let tooltip_est_h = if dim_entries.len() > 2 { 84.0 } else { 68.0 };
    let screen_rect = ctx.input(|i| i.viewport_rect());

    let is_connected_mode = plot_type != PlotType::Line;
    let mut tooltip_pos = if is_connected_mode {
        let offset_x = if hover_pos.x >= rect.center().x {
            36.0
        } else {
            -tooltip_w - 36.0
        };
        let offset_y = if hover_pos.y >= rect.center().y {
            -tooltip_est_h - 18.0
        } else {
            18.0
        };
        Pos2::new(hover_pos.x + offset_x, hover_pos.y + offset_y)
    } else {
        Pos2::new(hover_pos.x + 14.0, hover_pos.y + 14.0)
    };

    tooltip_pos.x = tooltip_pos.x.clamp(
        screen_rect.min.x + 10.0,
        screen_rect.max.x - tooltip_w - 10.0,
    );
    tooltip_pos.y = tooltip_pos.y.clamp(
        screen_rect.min.y + 10.0,
        screen_rect.max.y - tooltip_est_h - 10.0,
    );

    let tooltip_rect = Rect::from_min_size(tooltip_pos, egui::vec2(tooltip_w, tooltip_est_h));

    if let Some(target) = target_pos {
        draw_leader_callout(ui.painter(), ctx, target, tooltip_rect);
    }

    render_tooltip_popup(
        ctx,
        tooltip_pos,
        tooltip_w,
        var_name,
        raw_val,
        units_str,
        dim_entries,
        is_rgb,
        pixel_color,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_tooltip_popup(
    ctx: &egui::Context,
    tooltip_pos: Pos2,
    tooltip_w: f32,
    var_name: &str,
    raw_val: f32,
    units_str: &str,
    dim_entries: &[String],
    is_rgb: bool,
    pixel_color: Color32,
) {
    let (label_prefix, val_formatted) = if raw_val.is_nan() {
        ("Val:", "NaN".to_string())
    } else if is_rgb {
        let packed = raw_val.max(0.0) as u32;
        let r = packed & 0xFF;
        let g = (packed >> 8) & 0xFF;
        let b = (packed >> 16) & 0xFF;
        ("RGB:", format!("({}, {}, {})", r, g, b))
    } else if raw_val.abs() >= 1e4 || (raw_val.abs() <= 1e-3 && raw_val != 0.0) {
        ("Val:", format!("{:.4e}", raw_val))
    } else {
        ("Val:", format!("{:.4}", raw_val))
    };

    let style = ctx.style_of(ctx.theme());
    let strong_text = style.visuals.strong_text_color();
    let text_color = style.visuals.text_color();

    egui::Area::new(egui::Id::new("octant_hover_pixel_tooltip"))
        .order(egui::Order::Tooltip)
        .fixed_pos(tooltip_pos)
        .show(ctx, |ui| {
            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.set_max_width(tooltip_w - 16.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(var_name)
                                .small()
                                .strong()
                                .color(strong_text),
                        );
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            render_color_swatch(ui, pixel_color, raw_val.is_nan());
                            ui.label(egui::RichText::new(label_prefix).small().color(text_color));
                            ui.label(
                                egui::RichText::new(format!("{}{}", val_formatted, units_str))
                                    .size(15.0)
                                    .strong()
                                    .color(strong_text),
                            );
                        });
                        ui.add_space(1.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.spacing_mut().item_spacing.y = 1.0;
                            for (idx, entry) in dim_entries.iter().enumerate() {
                                if idx > 0 {
                                    ui.label(
                                        egui::RichText::new("•")
                                            .size(8.0)
                                            .color(text_color.linear_multiply(0.4)),
                                    );
                                }
                                ui.label(egui::RichText::new(entry).small().color(text_color));
                            }
                        });
                    });
                });
        });
}

fn render_color_swatch(ui: &mut egui::Ui, color: Color32, is_nan: bool) {
    let size = egui::vec2(12.0, 12.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let visuals = ui.visuals();
    let border_stroke = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
    let corner_radius = 2.0;

    if is_nan {
        ui.painter()
            .rect_filled(rect, corner_radius, visuals.faint_bg_color);
        ui.painter()
            .rect_stroke(rect, corner_radius, border_stroke, StrokeKind::Inside);
        let slash_color = visuals.text_color().linear_multiply(0.5);
        ui.painter().line_segment(
            [rect.left_top(), rect.right_bottom()],
            Stroke::new(1.0, slash_color),
        );
    } else {
        ui.painter().rect_filled(rect, corner_radius, color);
        ui.painter()
            .rect_stroke(rect, corner_radius, border_stroke, StrokeKind::Inside);
    }
}
