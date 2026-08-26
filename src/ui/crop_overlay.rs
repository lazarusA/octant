use crate::export::{AspectPreset, RoiCropBox};

/// Renders the interactive Region of Interest (ROI) crop box and guiding lines on top of the canvas.
pub fn show_crop_overlay(
    ui: &mut egui::Ui,
    canvas_rect: egui::Rect,
    crop_box: &mut RoiCropBox,
    is_open: &mut bool,
) {
    if !*is_open || canvas_rect.width() <= 10.0 || canvas_rect.height() <= 10.0 {
        return;
    }

    crop_box.clamp_bounds();

    // Map normalized [0..1] coordinates to screen pixels
    let rect_min = egui::pos2(
        canvas_rect.left() + crop_box.u_min * canvas_rect.width(),
        canvas_rect.top() + crop_box.v_min * canvas_rect.height(),
    );
    let rect_max = egui::pos2(
        canvas_rect.left() + crop_box.u_max * canvas_rect.width(),
        canvas_rect.top() + crop_box.v_max * canvas_rect.height(),
    );

    let box_rect = egui::Rect::from_min_max(rect_min, rect_max);
    let painter = ui.painter().with_clip_rect(canvas_rect);

    // 1. Dimmed outer mask (4 surrounding rectangles)
    let mask_color = egui::Color32::from_black_alpha(140);
    // Top
    painter.rect_filled(
        egui::Rect::from_min_max(
            canvas_rect.min,
            egui::pos2(canvas_rect.right(), box_rect.top()),
        ),
        0.0,
        mask_color,
    );
    // Bottom
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(canvas_rect.left(), box_rect.bottom()),
            canvas_rect.max,
        ),
        0.0,
        mask_color,
    );
    // Left
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(canvas_rect.left(), box_rect.top()),
            egui::pos2(box_rect.left(), box_rect.bottom()),
        ),
        0.0,
        mask_color,
    );
    // Right
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(box_rect.right(), box_rect.top()),
            egui::pos2(canvas_rect.right(), box_rect.bottom()),
        ),
        0.0,
        mask_color,
    );

    // 2. Rule-of-Thirds Grid Lines
    let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(70));
    let w_third = box_rect.width() / 3.0;
    let h_third = box_rect.height() / 3.0;

    // Vertical third lines
    painter.line_segment(
        [
            egui::pos2(box_rect.left() + w_third, box_rect.top()),
            egui::pos2(box_rect.left() + w_third, box_rect.bottom()),
        ],
        grid_stroke,
    );
    painter.line_segment(
        [
            egui::pos2(box_rect.left() + 2.0 * w_third, box_rect.top()),
            egui::pos2(box_rect.left() + 2.0 * w_third, box_rect.bottom()),
        ],
        grid_stroke,
    );

    // Horizontal third lines
    painter.line_segment(
        [
            egui::pos2(box_rect.left(), box_rect.top() + h_third),
            egui::pos2(box_rect.right(), box_rect.top() + h_third),
        ],
        grid_stroke,
    );
    painter.line_segment(
        [
            egui::pos2(box_rect.left(), box_rect.top() + 2.0 * h_third),
            egui::pos2(box_rect.right(), box_rect.top() + 2.0 * h_third),
        ],
        grid_stroke,
    );

    // 3. Boundary Border
    let border_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 180, 255));
    painter.rect_stroke(box_rect, 0.0, border_stroke, egui::StrokeKind::Outside);

    // 4. Interactive Drag & Resize Handling
    let handle_size = 12.0;
    let response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

    if let Some(mouse_pos) = response.hover_pos() {
        let handle_rect = |p: egui::Pos2| {
            egui::Rect::from_center_size(p, egui::vec2(handle_size * 2.0, handle_size * 2.0))
        };

        let nw = handle_rect(box_rect.left_top());
        let ne = handle_rect(box_rect.right_top());
        let sw = handle_rect(box_rect.left_bottom());
        let se = handle_rect(box_rect.right_bottom());

        if nw.contains(mouse_pos) || se.contains(mouse_pos) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeNorthWest);
        } else if ne.contains(mouse_pos) || sw.contains(mouse_pos) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeNorthEast);
        } else if box_rect.contains(mouse_pos) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
        }
    }

    if response.dragged() {
        let delta = response.drag_delta();
        let norm_dx = delta.x / canvas_rect.width();
        let norm_dy = delta.y / canvas_rect.height();

        if let Some(start_pos) = response.interact_pointer_pos() {
            let p = start_pos - delta;
            let grab_handle_margin = 24.0;

            let on_left = (p.x - box_rect.left()).abs() <= grab_handle_margin;
            let on_right = (p.x - box_rect.right()).abs() <= grab_handle_margin;
            let on_top = (p.y - box_rect.top()).abs() <= grab_handle_margin;
            let on_bottom = (p.y - box_rect.bottom()).abs() <= grab_handle_margin;

            if on_left || on_right || on_top || on_bottom {
                if on_left {
                    crop_box.u_min += norm_dx;
                }
                if on_right {
                    crop_box.u_max += norm_dx;
                }
                if on_top {
                    crop_box.v_min += norm_dy;
                }
                if on_bottom {
                    crop_box.v_max += norm_dy;
                }

                // Apply aspect ratio constraints if enabled
                if let Some(target_ratio) = crop_box.aspect.ratio() {
                    let cur_w = (crop_box.u_max - crop_box.u_min) * canvas_rect.width();
                    let target_h = (cur_w / target_ratio) / canvas_rect.height();
                    let center_v = (crop_box.v_min + crop_box.v_max) * 0.5;
                    crop_box.v_min = (center_v - target_h * 0.5).max(0.0);
                    crop_box.v_max = (center_v + target_h * 0.5).min(1.0);
                }
            } else if box_rect.contains(p) {
                // Pan/move entire crop box
                let w = crop_box.u_max - crop_box.u_min;
                let h = crop_box.v_max - crop_box.v_min;

                crop_box.u_min = (crop_box.u_min + norm_dx).clamp(0.0, 1.0 - w);
                crop_box.u_max = crop_box.u_min + w;
                crop_box.v_min = (crop_box.v_min + norm_dy).clamp(0.0, 1.0 - h);
                crop_box.v_max = crop_box.v_min + h;
            }
        }
    }

    // 5. Draw 8 Corner/Edge Handle markers
    let handle_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
    let handle_fill = egui::Color32::from_rgb(0, 180, 255);

    let handles = [
        box_rect.left_top(),
        egui::pos2(box_rect.center().x, box_rect.top()),
        box_rect.right_top(),
        egui::pos2(box_rect.right(), box_rect.center().y),
        box_rect.right_bottom(),
        egui::pos2(box_rect.center().x, box_rect.bottom()),
        box_rect.left_bottom(),
        egui::pos2(box_rect.left(), box_rect.center().y),
    ];

    for &h_pos in &handles {
        let r = egui::Rect::from_center_size(h_pos, egui::vec2(handle_size, handle_size));
        painter.rect(
            r,
            2.0,
            handle_fill,
            handle_stroke,
            egui::StrokeKind::Outside,
        );
    }

    // 6. Floating Control Toolbar on Top of the Crop Box
    let toolbar_pos = egui::pos2(
        box_rect.left().max(canvas_rect.left() + 8.0),
        (box_rect.top() - 36.0).max(canvas_rect.top() + 8.0),
    );

    egui::Area::new(egui::Id::new("octant_crop_toolbar"))
        .fixed_pos(toolbar_pos)
        .order(egui::Order::Foreground)
        .show(ui.ctx(), |ui| {
            egui::Frame::window(ui.style())
                .fill(egui::Color32::from_black_alpha(220))
                .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("✂️ Crop Area:").small().strong());

                        let w_px = (box_rect.width()).round() as u32;
                        let h_px = (box_rect.height()).round() as u32;
                        ui.label(
                            egui::RichText::new(format!("{} × {} px", w_px, h_px))
                                .small()
                                .color(egui::Color32::from_rgb(0, 200, 255)),
                        );

                        ui.separator();

                        // Aspect ratio presets
                        for preset in AspectPreset::ALL {
                            let is_active = crop_box.aspect == preset;
                            if ui.selectable_label(is_active, preset.label()).clicked() {
                                crop_box.aspect = preset;
                                if let Some(target_ratio) = preset.ratio() {
                                    let cur_w =
                                        (crop_box.u_max - crop_box.u_min) * canvas_rect.width();
                                    let target_h = (cur_w / target_ratio) / canvas_rect.height();
                                    let center_v = (crop_box.v_min + crop_box.v_max) * 0.5;
                                    crop_box.v_min = (center_v - target_h * 0.5).max(0.0);
                                    crop_box.v_max = (center_v + target_h * 0.5).min(1.0);
                                }
                            }
                        }

                        ui.separator();

                        if ui
                            .button("⟲ Reset")
                            .on_hover_text("Fit to Full Canvas")
                            .clicked()
                        {
                            *crop_box = RoiCropBox::default();
                        }

                        if ui
                            .button("✓ Done")
                            .on_hover_text("Close Guiding Lines Overlay")
                            .clicked()
                        {
                            *is_open = false;
                        }
                    });
                });
        });
}
