//! Canvas visual feedback overlays: capture flash, crop frame, and drag-and-drop indicator.

use crate::app::OctantApp;

/// Draws transient visual overlays on top of the active canvas.
pub fn draw_canvas_overlays(app: &mut OctantApp, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
    draw_flash_overlay(app, ui, canvas_rect);
    draw_crop_overlay(app, ui, canvas_rect);
    draw_drag_hover_cue(app, ui, canvas_rect);
}

fn draw_flash_overlay(app: &mut OctantApp, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
    let Some(flash_start) = app.export_flash_timer else {
        return;
    };

    let elapsed = flash_start.elapsed().as_secs_f32();
    let duration = 0.32;
    if elapsed < duration {
        let progress = (elapsed / duration).clamp(0.0, 1.0);
        let alpha = ((1.0 - progress) * 120.0) as u8;
        let flash_rect = if app.show_crop_overlay {
            egui::Rect::from_min_max(
                egui::pos2(
                    canvas_rect.left() + app.roi_crop_box.u_min * canvas_rect.width(),
                    canvas_rect.top() + app.roi_crop_box.v_min * canvas_rect.height(),
                ),
                egui::pos2(
                    canvas_rect.left() + app.roi_crop_box.u_max * canvas_rect.width(),
                    canvas_rect.top() + app.roi_crop_box.v_max * canvas_rect.height(),
                ),
            )
        } else {
            canvas_rect
        };

        ui.painter()
            .rect_filled(flash_rect, 0.0, egui::Color32::from_white_alpha(alpha));
        ui.painter().rect_stroke(
            flash_rect,
            0.0,
            egui::Stroke::new(
                2.0,
                egui::Color32::from_rgba_unmultiplied(
                    100,
                    220,
                    255,
                    ((alpha as f32) * 1.5).min(255.0) as u8,
                ),
            ),
            egui::StrokeKind::Inside,
        );
        ui.ctx().request_repaint();
    } else {
        app.export_flash_timer = None;
    }
}

fn draw_crop_overlay(app: &mut OctantApp, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
    if app.show_crop_overlay
        && app.pending_export.is_none()
        && let Some(crate::ui::crop_overlay::CropOverlayAction::Save) =
            crate::ui::crop_overlay::show_crop_overlay(
                ui,
                canvas_rect,
                &mut app.roi_crop_box,
                &mut app.show_crop_overlay,
            )
    {
        app.quick_save_canvas();
    }
}

fn draw_drag_hover_cue(app: &OctantApp, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
    let ctx = ui.ctx();
    let is_drag_hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
    if !is_drag_hovering || app.pending_export.is_some() {
        return;
    }

    let is_dark = ui.visuals().dark_mode;
    let stroke_color = if is_dark {
        egui::Color32::from_rgb(0, 190, 255)
    } else {
        egui::Color32::from_rgb(0, 125, 220)
    };
    let fill_color = if is_dark {
        egui::Color32::from_rgba_unmultiplied(0, 190, 255, 24)
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 125, 220, 18)
    };

    let overlay_rect = canvas_rect.shrink(12.0);
    ui.painter().rect(
        overlay_rect,
        10.0,
        fill_color,
        egui::Stroke::new(2.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    let badge_pos = overlay_rect.center();
    ui.painter().text(
        badge_pos,
        egui::Align2::CENTER_CENTER,
        "Drop dataset to visualize (.nc, .h5, .zarr, .icechunk, .tif)",
        egui::FontId::proportional(15.0),
        stroke_color,
    );
    ctx.request_repaint();
}
