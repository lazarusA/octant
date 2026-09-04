use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct DropZoneWarningState {
    pub triggered_at: Instant,
    pub message: String,
}

impl Default for DropZoneWarningState {
    fn default() -> Self {
        Self {
            triggered_at: Instant::now(),
            message: String::new(),
        }
    }
}

/// Triggers an interactive warning on all active drop zones and requests a repaint.
pub fn trigger_drop_zone_warning(ctx: &egui::Context, message: impl Into<String>) {
    let id = egui::Id::new("drop_zone_warning_state");
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            DropZoneWarningState {
                triggered_at: Instant::now(),
                message: message.into(),
            },
        );
    });
    ctx.request_repaint();
}

/// Renders a modern, theme-adaptive drag-and-drop drop zone widget.
///
/// Returns `Some(PathBuf)` if a supported file or directory was dropped during this frame.
pub fn show_drop_zone(
    ui: &mut egui::Ui,
    desired_width: Option<f32>,
    height: f32,
) -> Option<PathBuf> {
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

    let hovered_files = ui.ctx().input(|i| i.raw.hovered_files.clone());
    let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());

    let is_drag_hovering = !hovered_files.is_empty();
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
            draw_dashed_border(ui.painter(), rect, 8.0, stroke_color.gamma_multiply(0.4));
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

            // Exclamation mark stem
            ui.painter().line_segment(
                [
                    egui::pos2(center.x, icon_y - 2.5),
                    egui::pos2(center.x, icon_y + 1.5),
                ],
                icon_stroke,
            );
            // Exclamation mark dot
            ui.painter()
                .circle_filled(egui::pos2(center.x, icon_y + 3.8), 1.0, stroke_color);
        } else {
            // Custom drawn drop arrow + container tray icon
            // Arrow shaft
            ui.painter().line_segment(
                [
                    egui::pos2(center.x, icon_y - 6.0),
                    egui::pos2(center.x, icon_y + 2.0),
                ],
                icon_stroke,
            );
            // Arrow head
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
            // Bottom tray
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

    // Process dropped files
    if let Some(first) = dropped_files.first() {
        let path = first.path();
        let path_str = path.to_string_lossy().trim().to_string();
        if !path_str.is_empty() {
            match crate::utils::infer_store_kind_from_target(&path_str) {
                Ok(_) => {
                    ui.ctx()
                        .data_mut(|d| d.remove_temp::<DropZoneWarningState>(warning_id));
                    return Some(path.to_path_buf());
                }
                Err(err) => {
                    trigger_drop_zone_warning(ui.ctx(), format!("{err}: {path_str}"));
                    return None;
                }
            }
        }
    }

    None
}

fn draw_dashed_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    _corner_radius: f32,
    color: egui::Color32,
) {
    let dash_len = 5.0;
    let gap_len = 4.0;
    let stroke = egui::Stroke::new(1.0, color);

    // Top edge
    let mut x = rect.left() + 8.0;
    while x + dash_len <= rect.right() - 8.0 {
        painter.line_segment(
            [
                egui::pos2(x, rect.top()),
                egui::pos2(x + dash_len, rect.top()),
            ],
            stroke,
        );
        x += dash_len + gap_len;
    }

    // Bottom edge
    let mut x = rect.left() + 8.0;
    while x + dash_len <= rect.right() - 8.0 {
        painter.line_segment(
            [
                egui::pos2(x, rect.bottom()),
                egui::pos2(x + dash_len, rect.bottom()),
            ],
            stroke,
        );
        x += dash_len + gap_len;
    }

    // Left edge
    let mut y = rect.top() + 8.0;
    while y + dash_len <= rect.bottom() - 8.0 {
        painter.line_segment(
            [
                egui::pos2(rect.left(), y),
                egui::pos2(rect.left(), y + dash_len),
            ],
            stroke,
        );
        y += dash_len + gap_len;
    }

    // Right edge
    let mut y = rect.top() + 8.0;
    while y + dash_len <= rect.bottom() - 8.0 {
        painter.line_segment(
            [
                egui::pos2(rect.right(), y),
                egui::pos2(rect.right(), y + dash_len),
            ],
            stroke,
        );
        y += dash_len + gap_len;
    }
}
