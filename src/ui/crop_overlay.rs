use crate::export::{AspectPreset, RoiCropBox};

/// Actions dispatched from the interactive Crop Toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CropOverlayAction {
    Save,
    Reset,
    Done,
}

/// Renders the interactive Region of Interest (ROI) crop box and guiding lines on top of the canvas.
pub fn show_crop_overlay(
    ui: &mut egui::Ui,
    canvas_rect: egui::Rect,
    crop_box: &mut RoiCropBox,
    is_open: &mut bool,
) -> Option<CropOverlayAction> {
    if !*is_open || canvas_rect.width() <= 10.0 || canvas_rect.height() <= 10.0 {
        return None;
    }

    crop_box.clamp_bounds();
    let dark_mode = ui.visuals().dark_mode;
    let mut action = None;

    // Theme-adaptive colors
    let mask_color = if dark_mode {
        egui::Color32::from_black_alpha(150)
    } else {
        egui::Color32::from_black_alpha(80)
    };

    let accent_color = if dark_mode {
        egui::Color32::from_rgb(0, 190, 255)
    } else {
        egui::Color32::from_rgb(0, 125, 220)
    };

    let grid_stroke = egui::Stroke::new(
        1.0,
        if dark_mode {
            egui::Color32::from_white_alpha(60)
        } else {
            egui::Color32::from_black_alpha(50)
        },
    );

    let border_stroke = egui::Stroke::new(1.5, accent_color);

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
    painter.rect_filled(
        egui::Rect::from_min_max(
            canvas_rect.min,
            egui::pos2(canvas_rect.right(), box_rect.top()),
        ),
        0.0,
        mask_color,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(canvas_rect.left(), box_rect.bottom()),
            canvas_rect.max,
        ),
        0.0,
        mask_color,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(canvas_rect.left(), box_rect.top()),
            egui::pos2(box_rect.left(), box_rect.bottom()),
        ),
        0.0,
        mask_color,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(box_rect.right(), box_rect.top()),
            egui::pos2(canvas_rect.right(), box_rect.bottom()),
        ),
        0.0,
        mask_color,
    );

    // 2. Rule-of-Thirds Grid Lines
    let w_third = box_rect.width() / 3.0;
    let h_third = box_rect.height() / 3.0;

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
    painter.rect_stroke(box_rect, 0.0, border_stroke, egui::StrokeKind::Outside);

    // 4. Interactive Drag & Resize Handling
    let response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());
    let hit_radius = 18.0;

    if let Some(mouse_pos) = response.hover_pos() {
        let handle_rect = |p: egui::Pos2| {
            egui::Rect::from_center_size(p, egui::vec2(hit_radius * 2.0, hit_radius * 2.0))
        };

        let nw = handle_rect(box_rect.left_top());
        let ne = handle_rect(box_rect.right_top());
        let sw = handle_rect(box_rect.left_bottom());
        let se = handle_rect(box_rect.right_bottom());
        let top_mid = handle_rect(egui::pos2(box_rect.center().x, box_rect.top()));
        let bot_mid = handle_rect(egui::pos2(box_rect.center().x, box_rect.bottom()));
        let left_mid = handle_rect(egui::pos2(box_rect.left(), box_rect.center().y));
        let right_mid = handle_rect(egui::pos2(box_rect.right(), box_rect.center().y));

        if nw.contains(mouse_pos) || se.contains(mouse_pos) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeNorthWest);
        } else if ne.contains(mouse_pos) || sw.contains(mouse_pos) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeNorthEast);
        } else if top_mid.contains(mouse_pos) || bot_mid.contains(mouse_pos) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeVertical);
        } else if left_mid.contains(mouse_pos) || right_mid.contains(mouse_pos) {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
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

    // 5. Draw Modern Pro-Grade Handles (L-shaped Corner Brackets & Elongated Thin Side Blocks)
    let corner_arm_len = 16.0;
    let corner_stroke = egui::Stroke::new(3.0, accent_color);

    // 4 Corner L-Brackets
    for &(corner, dx, dy) in &[
        (box_rect.left_top(), 1.0, 1.0),
        (box_rect.right_top(), -1.0, 1.0),
        (box_rect.left_bottom(), 1.0, -1.0),
        (box_rect.right_bottom(), -1.0, -1.0),
    ] {
        painter.line_segment(
            [corner, egui::pos2(corner.x + dx * corner_arm_len, corner.y)],
            corner_stroke,
        );
        painter.line_segment(
            [corner, egui::pos2(corner.x, corner.y + dy * corner_arm_len)],
            corner_stroke,
        );
    }

    // 4 Elongated Thin Side Edge Pills
    let edge_pill_stroke = egui::Stroke::new(
        1.0,
        if dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        },
    );

    let edge_pills = [
        (
            egui::pos2(box_rect.center().x, box_rect.top()),
            egui::vec2(28.0, 4.0),
        ),
        (
            egui::pos2(box_rect.center().x, box_rect.bottom()),
            egui::vec2(28.0, 4.0),
        ),
        (
            egui::pos2(box_rect.left(), box_rect.center().y),
            egui::vec2(4.0, 28.0),
        ),
        (
            egui::pos2(box_rect.right(), box_rect.center().y),
            egui::vec2(4.0, 28.0),
        ),
    ];

    for (pos, size) in edge_pills {
        painter.rect(
            egui::Rect::from_center_size(pos, size),
            2.0,
            accent_color,
            edge_pill_stroke,
            egui::StrokeKind::Outside,
        );
    }

    // 6. Floating Theme-Aware Control Toolbar on Top of the Crop Box
    let toolbar_pos = egui::pos2(
        box_rect.left().max(canvas_rect.left() + 8.0),
        (box_rect.top() - 38.0).max(canvas_rect.top() + 8.0),
    );

    egui::Area::new(egui::Id::new("octant_crop_toolbar"))
        .fixed_pos(toolbar_pos)
        .order(egui::Order::Foreground)
        .show(ui.ctx(), |ui| {
            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(8, 5))
                .corner_radius(6.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("✂️ ROI:").small().strong());

                        let w_px = (box_rect.width()).round() as u32;
                        let h_px = (box_rect.height()).round() as u32;
                        ui.label(
                            egui::RichText::new(format!("{}×{} px", w_px, h_px))
                                .small()
                                .color(accent_color),
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
                            .button(egui::RichText::new("💾 Save").strong())
                            .on_hover_text("Save cropped ROI figure (Cmd+S)")
                            .clicked()
                        {
                            action = Some(CropOverlayAction::Save);
                        }

                        if ui
                            .button("⟲ Reset")
                            .on_hover_text("Fit crop box to full canvas")
                            .clicked()
                        {
                            *crop_box = RoiCropBox::default();
                            action = Some(CropOverlayAction::Reset);
                        }

                        if ui
                            .button("✓ Done")
                            .on_hover_text("Close crop overlay")
                            .clicked()
                        {
                            *is_open = false;
                            action = Some(CropOverlayAction::Done);
                        }
                    });
                });
        });

    action
}
