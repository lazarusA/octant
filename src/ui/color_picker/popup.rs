//! Streamlined color picker popup with RGB channel edits, 2D SV area, and Hue/Alpha bars.

use crate::ui::icons::{Icon, UiIconExt};
use egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2, ecolor::Hsva};

/// Color space selection for RGB channel displays (Byte 0-255 vs Float 0-1)
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ColorGammaSpace {
    Byte,  // 0 - 255
    Float, // 0.0 - 1.0
}

/// Renders the streamlined color picker popup.
pub(crate) fn show_clean_color_picker_popup(
    ui: &mut egui::Ui,
    id_salt: egui::Id,
    color_rgba: &mut [f32; 4],
    popup_open_id: egui::Id,
    popup_area_id: egui::Id,
    anchor_pos: Pos2,
) {
    let mut is_open = ui
        .data(|d| d.get_temp::<bool>(popup_open_id))
        .unwrap_or(false);
    if !is_open {
        return;
    }

    let screen_rect = ui
        .input(|i| i.raw.screen_rect)
        .unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0)));
    let popup_w = 230.0;
    let popup_h = 320.0;
    let popup_pos = Pos2::new(
        anchor_pos
            .x
            .clamp(10.0, (screen_rect.max.x - popup_w - 10.0).max(10.0)),
        anchor_pos
            .y
            .clamp(10.0, (screen_rect.max.y - popup_h - 10.0).max(10.0)),
    );
    let hsva_id = id_salt.with("hsva_state");
    let gamma_space_id = id_salt.with("gamma_space_mode");

    // Persistent HSVA state to preserve exact hue when brightness or saturation is zero
    let mut hsva = ui.data(|d| d.get_temp::<Hsva>(hsva_id)).unwrap_or_else(|| {
        Hsva::from_rgba_unmultiplied(color_rgba[0], color_rgba[1], color_rgba[2], color_rgba[3])
    });

    let mut gamma_space = ui
        .data(|d| d.get_temp::<ColorGammaSpace>(gamma_space_id))
        .unwrap_or(ColorGammaSpace::Byte);

    egui::Area::new(popup_area_id)
        .order(egui::Order::Tooltip)
        .fixed_pos(popup_pos)
        .show(ui.ctx(), |ui| {
            let frame_resp = egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(popup_w);
                let mut changed = false;

                // 1. TOP SECTION: RGB gamma space mode (0-255 vs 0-1), Copy button & RGB edits
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.selectable_value(&mut gamma_space, ColorGammaSpace::Byte, "0-255");
                    ui.selectable_value(&mut gamma_space, ColorGammaSpace::Float, "0-1");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .icon_button(Icon::Clipboard, "Copy")
                            .on_hover_text("Copy color values to clipboard")
                            .clicked()
                        {
                            let copy_text = match gamma_space {
                                ColorGammaSpace::Byte => {
                                    let r = (color_rgba[0] * 255.0).round() as u8;
                                    let g = (color_rgba[1] * 255.0).round() as u8;
                                    let b = (color_rgba[2] * 255.0).round() as u8;
                                    let a = (color_rgba[3] * 255.0).round() as u8;
                                    if a == 255 {
                                        format!("rgb({}, {}, {})", r, g, b)
                                    } else {
                                        format!("rgba({}, {}, {}, {})", r, g, b, a)
                                    }
                                }
                                ColorGammaSpace::Float => {
                                    format!(
                                        "[{:.3}, {:.3}, {:.3}, {:.3}]",
                                        color_rgba[0], color_rgba[1], color_rgba[2], color_rgba[3]
                                    )
                                }
                            };
                            ui.ctx().copy_text(copy_text);
                        }
                    });
                });

                ui.add_space(3.0);

                // RGB Numeric Inputs
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    match gamma_space {
                        ColorGammaSpace::Byte => {
                            let mut r = (color_rgba[0] * 255.0).round() as u8;
                            let mut g = (color_rgba[1] * 255.0).round() as u8;
                            let mut b = (color_rgba[2] * 255.0).round() as u8;
                            let mut a = (color_rgba[3] * 255.0).round() as u8;
                            let mut rgb_changed = false;

                            ui.label(
                                egui::RichText::new("R")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(255, 100, 100)),
                            );
                            rgb_changed |= ui
                                .add(egui::DragValue::new(&mut r).range(0..=255).speed(1.0))
                                .changed();
                            ui.label(
                                egui::RichText::new("G")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(100, 255, 100)),
                            );
                            rgb_changed |= ui
                                .add(egui::DragValue::new(&mut g).range(0..=255).speed(1.0))
                                .changed();
                            ui.label(
                                egui::RichText::new("B")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(100, 150, 255)),
                            );
                            rgb_changed |= ui
                                .add(egui::DragValue::new(&mut b).range(0..=255).speed(1.0))
                                .changed();
                            ui.label(
                                egui::RichText::new("A")
                                    .small()
                                    .strong()
                                    .color(Color32::from_gray(180)),
                            );
                            rgb_changed |= ui
                                .add(egui::DragValue::new(&mut a).range(0..=255).speed(1.0))
                                .changed();

                            if rgb_changed {
                                *color_rgba = [
                                    r as f32 / 255.0,
                                    g as f32 / 255.0,
                                    b as f32 / 255.0,
                                    a as f32 / 255.0,
                                ];
                                hsva = Hsva::from_rgba_unmultiplied(
                                    color_rgba[0],
                                    color_rgba[1],
                                    color_rgba[2],
                                    color_rgba[3],
                                );
                                changed = true;
                            }
                        }
                        ColorGammaSpace::Float => {
                            let mut r = color_rgba[0];
                            let mut g = color_rgba[1];
                            let mut b = color_rgba[2];
                            let mut a = color_rgba[3];
                            let mut rgb_changed = false;

                            ui.label(
                                egui::RichText::new("R")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(255, 100, 100)),
                            );
                            rgb_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut r)
                                        .range(0.0..=1.0)
                                        .speed(0.01)
                                        .max_decimals(2),
                                )
                                .changed();
                            ui.label(
                                egui::RichText::new("G")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(100, 255, 100)),
                            );
                            rgb_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut g)
                                        .range(0.0..=1.0)
                                        .speed(0.01)
                                        .max_decimals(2),
                                )
                                .changed();
                            ui.label(
                                egui::RichText::new("B")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(100, 150, 255)),
                            );
                            rgb_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut b)
                                        .range(0.0..=1.0)
                                        .speed(0.01)
                                        .max_decimals(2),
                                )
                                .changed();
                            ui.label(
                                egui::RichText::new("A")
                                    .small()
                                    .strong()
                                    .color(Color32::from_gray(180)),
                            );
                            rgb_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut a)
                                        .range(0.0..=1.0)
                                        .speed(0.01)
                                        .max_decimals(2),
                                )
                                .changed();

                            if rgb_changed {
                                *color_rgba = [r, g, b, a];
                                hsva = Hsva::from_rgba_unmultiplied(
                                    color_rgba[0],
                                    color_rgba[1],
                                    color_rgba[2],
                                    color_rgba[3],
                                );
                                changed = true;
                            }
                        }
                    }
                });

                ui.add_space(5.0);

                // 2. MIDDLE SECTION: 2D Saturation / Value Color Area
                let available_w = ui.available_width();
                let sv_size = Vec2::new(available_w, 165.0);
                let (sv_rect, sv_resp) = ui.allocate_exact_size(sv_size, egui::Sense::drag());

                if (sv_resp.dragged() || sv_resp.clicked())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                {
                    hsva.s = ((pos.x - sv_rect.min.x) / sv_rect.width()).clamp(0.0_f32, 1.0_f32);
                    hsva.v = (1.0_f32 - (pos.y - sv_rect.min.y) / sv_rect.height())
                        .clamp(0.0_f32, 1.0_f32);
                    changed = true;
                }

                let mut sv_mesh = egui::Mesh::default();
                let c_tl = Color32::WHITE;
                let c_tr = Color32::from(Hsva::new(hsva.h, 1.0, 1.0, 1.0));
                let c_bl = Color32::BLACK;
                let c_br = Color32::BLACK;

                sv_mesh.colored_vertex(sv_rect.left_top(), c_tl);
                sv_mesh.colored_vertex(sv_rect.right_top(), c_tr);
                sv_mesh.colored_vertex(sv_rect.right_bottom(), c_br);
                sv_mesh.colored_vertex(sv_rect.left_bottom(), c_bl);
                sv_mesh.add_triangle(0, 1, 2);
                sv_mesh.add_triangle(0, 2, 3);
                ui.painter().add(Shape::mesh(sv_mesh));

                ui.painter().rect_stroke(
                    sv_rect,
                    2.0,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    egui::StrokeKind::Inside,
                );

                let cursor_pos = Pos2::new(
                    sv_rect.min.x + hsva.s * sv_rect.width(),
                    sv_rect.min.y + (1.0 - hsva.v) * sv_rect.height(),
                );
                ui.painter().circle(
                    cursor_pos,
                    5.0,
                    Color32::TRANSPARENT,
                    Stroke::new(2.0, Color32::WHITE),
                );
                ui.painter().circle(
                    cursor_pos,
                    6.0,
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, Color32::BLACK),
                );

                ui.add_space(5.0);

                // 3. BOTTOM SECTION: Hue Spectrum Bar & Alpha Bar
                let hue_size = Vec2::new(available_w, 15.0);
                let (hue_rect, hue_resp) = ui.allocate_exact_size(hue_size, egui::Sense::drag());

                if (hue_resp.dragged() || hue_resp.clicked())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                {
                    hsva.h = ((pos.x - hue_rect.min.x) / hue_rect.width()).clamp(0.0_f32, 1.0_f32);
                    changed = true;
                }

                let num_segments = 6;
                let mut hue_mesh = egui::Mesh::default();
                for i in 0..num_segments {
                    let h0 = i as f32 / num_segments as f32;
                    let h1 = (i + 1) as f32 / num_segments as f32;
                    let x0 = hue_rect.min.x + h0 * hue_rect.width();
                    let x1 = hue_rect.min.x + h1 * hue_rect.width();
                    let r = Rect::from_min_max(
                        Pos2::new(x0, hue_rect.min.y),
                        Pos2::new(x1, hue_rect.max.y),
                    );

                    let c0 = Color32::from(Hsva::new(h0, 1.0, 1.0, 1.0));
                    let c1 = Color32::from(Hsva::new(h1, 1.0, 1.0, 1.0));

                    let idx = hue_mesh.vertices.len() as u32;
                    hue_mesh.colored_vertex(r.left_top(), c0);
                    hue_mesh.colored_vertex(r.right_top(), c1);
                    hue_mesh.colored_vertex(r.right_bottom(), c1);
                    hue_mesh.colored_vertex(r.left_bottom(), c0);
                    hue_mesh.add_triangle(idx, idx + 1, idx + 2);
                    hue_mesh.add_triangle(idx, idx + 2, idx + 3);
                }
                ui.painter().add(Shape::mesh(hue_mesh));

                ui.painter().rect_stroke(
                    hue_rect,
                    2.0,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    egui::StrokeKind::Inside,
                );

                let hue_cursor_x = hue_rect.min.x + hsva.h * hue_rect.width();
                let hue_cursor_rect = Rect::from_center_size(
                    Pos2::new(hue_cursor_x, hue_rect.center().y),
                    Vec2::new(5.0, hue_rect.height() + 4.0),
                );
                ui.painter().rect(
                    hue_cursor_rect,
                    2.0,
                    Color32::WHITE,
                    Stroke::new(1.0, Color32::BLACK),
                    egui::StrokeKind::Middle,
                );

                ui.add_space(4.0);

                let alpha_size = Vec2::new(available_w, 15.0);
                let (alpha_rect, alpha_resp) =
                    ui.allocate_exact_size(alpha_size, egui::Sense::drag());

                if (alpha_resp.dragged() || alpha_resp.clicked())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                {
                    hsva.a =
                        ((pos.x - alpha_rect.min.x) / alpha_rect.width()).clamp(0.0_f32, 1.0_f32);
                    changed = true;
                }

                let grid_size = 5.0;
                let mut check_x = alpha_rect.min.x;
                while check_x < alpha_rect.max.x {
                    let mut check_y = alpha_rect.min.y;
                    let col = ((check_x - alpha_rect.min.x) / grid_size) as usize;
                    while check_y < alpha_rect.max.y {
                        let row = ((check_y - alpha_rect.min.y) / grid_size) as usize;
                        let c = if (row + col).is_multiple_of(2) {
                            Color32::from_gray(180)
                        } else {
                            Color32::from_gray(240)
                        };
                        let r = Rect::from_min_max(
                            Pos2::new(check_x, check_y),
                            Pos2::new(
                                (check_x + grid_size).min(alpha_rect.max.x),
                                (check_y + grid_size).min(alpha_rect.max.y),
                            ),
                        );
                        ui.painter().rect_filled(r, 0.0, c);
                        check_y += grid_size;
                    }
                    check_x += grid_size;
                }

                let mut alpha_mesh = egui::Mesh::default();
                let c_trans = Color32::from(Hsva::new(hsva.h, hsva.s, hsva.v, 0.0));
                let c_opaque = Color32::from(Hsva::new(hsva.h, hsva.s, hsva.v, 1.0));
                alpha_mesh.colored_vertex(alpha_rect.left_top(), c_trans);
                alpha_mesh.colored_vertex(alpha_rect.right_top(), c_opaque);
                alpha_mesh.colored_vertex(alpha_rect.right_bottom(), c_opaque);
                alpha_mesh.colored_vertex(alpha_rect.left_bottom(), c_trans);
                alpha_mesh.add_triangle(0, 1, 2);
                alpha_mesh.add_triangle(0, 2, 3);
                ui.painter().add(Shape::mesh(alpha_mesh));

                ui.painter().rect_stroke(
                    alpha_rect,
                    2.0,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    egui::StrokeKind::Inside,
                );

                let alpha_cursor_x = alpha_rect.min.x + hsva.a * alpha_rect.width();
                let alpha_cursor_rect = Rect::from_center_size(
                    Pos2::new(alpha_cursor_x, alpha_rect.center().y),
                    Vec2::new(5.0, alpha_rect.height() + 4.0),
                );
                ui.painter().rect(
                    alpha_cursor_rect,
                    2.0,
                    Color32::WHITE,
                    Stroke::new(1.0, Color32::BLACK),
                    egui::StrokeKind::Middle,
                );

                if changed {
                    let rgba_unmult = egui::ecolor::Rgba::from(hsva).to_rgba_unmultiplied();
                    *color_rgba = rgba_unmult;
                }
            });

            if ui.input(|i| i.pointer.any_pressed())
                && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                && !frame_resp.response.rect.contains(pos)
            {
                is_open = false;
            }
        });

    ui.data_mut(|d| {
        d.insert_temp(popup_open_id, is_open);
        d.insert_temp(hsva_id, hsva);
        d.insert_temp(gamma_space_id, gamma_space);
    });
}
