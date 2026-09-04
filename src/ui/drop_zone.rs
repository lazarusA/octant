use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct DropZoneWarningState {
    pub triggered_at: Instant,
}

impl Default for DropZoneWarningState {
    fn default() -> Self {
        Self {
            triggered_at: Instant::now(),
        }
    }
}

/// Triggers an interactive warning on all active drop zones and requests a repaint.
pub fn trigger_drop_zone_warning(ctx: &egui::Context) {
    let id = egui::Id::new("drop_zone_warning_state");
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            DropZoneWarningState {
                triggered_at: Instant::now(),
            },
        );
    });
    ctx.request_repaint();
}

/// Clears any active warning state on drop zones.
pub fn clear_drop_zone_warning(ctx: &egui::Context) {
    let id = egui::Id::new("drop_zone_warning_state");
    ctx.data_mut(|d| {
        d.remove_temp::<DropZoneWarningState>(id);
    });
}

/// Renders a modern, theme-adaptive drag-and-drop drop zone widget.
pub fn show_drop_zone(
    ui: &mut egui::Ui,
    desired_width: Option<f32>,
    height: f32,
) -> egui::Response {
    let width = desired_width.unwrap_or_else(|| ui.available_width().max(120.0));
    let size = egui::vec2(width, height);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    let warning_id = egui::Id::new("drop_zone_warning_state");
    let active_warning: Option<DropZoneWarningState> = ui.ctx().data(|d| d.get_temp(warning_id));
    let is_warning_active = active_warning
        .as_ref()
        .is_some_and(|w| w.triggered_at.elapsed() < Duration::from_millis(3500));

    if is_warning_active {
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }

    // Zero heap-allocation drag-hover detection
    let is_drag_hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    let is_pointer_hovering = response.hovered();

    if is_drag_hovering {
        ui.ctx().request_repaint();
    }

    if ui.is_rect_visible(rect) {
        let is_dark = ui.visuals().dark_mode;
        let normal_accent = if is_dark {
            egui::Color32::from_rgb(0, 190, 255)
        } else {
            egui::Color32::from_rgb(0, 125, 220)
        };
        let warning_accent = egui::Color32::from_rgb(255, 130, 60);

        let accent_color = if is_warning_active {
            warning_accent
        } else {
            normal_accent
        };

        let bg_fill = if is_warning_active {
            if is_dark {
                egui::Color32::from_rgba_unmultiplied(255, 110, 50, 36)
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 130, 60, 26)
            }
        } else if is_drag_hovering {
            if is_dark {
                egui::Color32::from_rgba_unmultiplied(0, 190, 255, 30)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 125, 220, 24)
            }
        } else if is_pointer_hovering {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            ui.visuals().extreme_bg_color
        };

        let stroke_color = if is_warning_active {
            warning_accent
        } else if is_drag_hovering {
            accent_color
        } else if is_pointer_hovering {
            ui.visuals().widgets.hovered.bg_stroke.color
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke.color
        };

        let stroke_width = if is_warning_active || is_drag_hovering {
            1.8
        } else {
            1.0
        };
        let stroke = egui::Stroke::new(stroke_width, stroke_color);

        // Draw background container
        ui.painter()
            .rect(rect, 8.0, bg_fill, stroke, egui::StrokeKind::Inside);

        // Draw dashed perimeter cue when idle to indicate drop capability
        if !is_drag_hovering && !is_warning_active && stroke_width <= 1.2 {
            draw_dashed_border(ui.painter(), rect, stroke_color.gamma_multiply(0.4));
        }

        let center = rect.center();
        let icon_y = center.y - 12.0;
        let icon_stroke = egui::Stroke::new(1.4, stroke_color);

        if is_warning_active {
            // Draw warning triangle icon
            let p_top = egui::pos2(center.x, icon_y - 6.5);
            let p_left = egui::pos2(center.x - 7.5, icon_y + 5.5);
            let p_right = egui::pos2(center.x + 7.5, icon_y + 5.5);

            ui.painter().line_segment([p_top, p_left], icon_stroke);
            ui.painter().line_segment([p_left, p_right], icon_stroke);
            ui.painter().line_segment([p_right, p_top], icon_stroke);

            // Exclamation mark stem & dot
            ui.painter().line_segment(
                [
                    egui::pos2(center.x, icon_y - 2.5),
                    egui::pos2(center.x, icon_y + 1.5),
                ],
                icon_stroke,
            );
            ui.painter()
                .circle_filled(egui::pos2(center.x, icon_y + 3.8), 1.0, stroke_color);
        } else {
            // Drop arrow (shaft + head)
            ui.painter().line_segment(
                [
                    egui::pos2(center.x, icon_y - 6.0),
                    egui::pos2(center.x, icon_y + 2.0),
                ],
                icon_stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(center.x - 3.5, icon_y - 1.5),
                    egui::pos2(center.x, icon_y + 2.0),
                ],
                icon_stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(center.x + 3.5, icon_y - 1.5),
                    egui::pos2(center.x, icon_y + 2.0),
                ],
                icon_stroke,
            );

            // Container tray
            ui.painter().line_segment(
                [
                    egui::pos2(center.x - 7.0, icon_y + 1.0),
                    egui::pos2(center.x - 7.0, icon_y + 5.0),
                ],
                icon_stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(center.x - 7.0, icon_y + 5.0),
                    egui::pos2(center.x + 7.0, icon_y + 5.0),
                ],
                icon_stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(center.x + 7.0, icon_y + 5.0),
                    egui::pos2(center.x + 7.0, icon_y + 1.0),
                ],
                icon_stroke,
            );
        }

        // Labels
        let title_text = if is_warning_active {
            "⚠️ Type not supported"
        } else if is_drag_hovering {
            "Release to load dataset"
        } else {
            "Drag & Drop File or Directory"
        };

        let title_color = if is_warning_active {
            warning_accent
        } else if is_drag_hovering {
            accent_color
        } else {
            ui.visuals().strong_text_color()
        };

        let title_pos = egui::pos2(center.x, center.y + 4.0);
        ui.painter().text(
            title_pos,
            egui::Align2::CENTER_CENTER,
            title_text,
            egui::FontId::proportional(12.0),
            title_color,
        );

        let sub_text = if is_warning_active {
            "Supported: .nc, .h5, .zarr, .icechunk"
        } else {
            ".nc, .h5, .zarr, .icechunk"
        };
        let sub_color = if is_warning_active {
            warning_accent.gamma_multiply(0.85)
        } else {
            ui.visuals().weak_text_color()
        };
        let sub_pos = egui::pos2(center.x, center.y + 18.0);
        ui.painter().text(
            sub_pos,
            egui::Align2::CENTER_CENTER,
            sub_text,
            egui::FontId::monospace(10.0),
            sub_color,
        );
    }

    response
}

/// Draws a unified, dashed border around `rect` without code duplication.
fn draw_dashed_border(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let dash_len = 5.0;
    let gap_len = 4.0;
    let step = dash_len + gap_len;
    let stroke = egui::Stroke::new(1.0, color);
    let inset = 8.0;

    // Horizontal edges (Top & Bottom)
    let mut x = rect.left() + inset;
    let max_x = rect.right() - inset;
    while x + dash_len <= max_x {
        painter.line_segment(
            [
                egui::pos2(x, rect.top()),
                egui::pos2(x + dash_len, rect.top()),
            ],
            stroke,
        );
        painter.line_segment(
            [
                egui::pos2(x, rect.bottom()),
                egui::pos2(x + dash_len, rect.bottom()),
            ],
            stroke,
        );
        x += step;
    }

    // Vertical edges (Left & Right)
    let mut y = rect.top() + inset;
    let max_y = rect.bottom() - inset;
    while y + dash_len <= max_y {
        painter.line_segment(
            [
                egui::pos2(rect.left(), y),
                egui::pos2(rect.left(), y + dash_len),
            ],
            stroke,
        );
        painter.line_segment(
            [
                egui::pos2(rect.right(), y),
                egui::pos2(rect.right(), y + dash_len),
            ],
            stroke,
        );
        y += step;
    }
}
