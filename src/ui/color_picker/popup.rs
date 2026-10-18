//! Streamlined color picker popup with RGB channel edits, 2D SV area, and Hue/Alpha bars.

use super::planes;
use crate::i18n::{ColormapKey as K, tr};
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::keyboard::windows;
use crate::ui::number_input::LocalizedInput;
use egui::{Color32, Pos2, Rect, Vec2, ecolor::Hsva};

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
    let was_open = ui
        .data(|d| d.get_temp::<bool>(popup_open_id))
        .unwrap_or(false);
    // One popup at a time: another opening (a select) closes the picker.
    let ctx = ui.ctx();
    let other_popup =
        || egui::Popup::is_any_open(ctx) && !egui::Popup::is_id_open(ctx, popup_area_id);
    let mut is_open = was_open && !other_popup();
    if was_open && !is_open {
        ui.data_mut(|d| d.insert_temp(popup_open_id, false));
    }
    // Focus returns to the swatch once the popup closes.
    windows::follow_window(ui.ctx(), popup_area_id, is_open);
    if !is_open {
        return;
    }
    // Known to egui as its open popup, so Escape and the canvas keys leave
    // the panels behind it alone.
    egui::Popup::open_id(ui.ctx(), popup_area_id);

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
                            .icon_button(Icon::Clipboard, tr(K::Copy))
                            .on_hover_text(tr(K::CopyHover))
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
                    if channel_inputs(ui, gamma_space, color_rgba) {
                        let [r, g, b, a] = *color_rgba;
                        hsva = Hsva::from_rgba_unmultiplied(r, g, b, a);
                        changed = true;
                    }
                });

                ui.add_space(5.0);

                // 2. MIDDLE SECTION: 2D Saturation / Value Color Area
                let available_w = ui.available_width();
                changed |= planes::sv_plane(ui, &mut hsva, available_w);
                ui.add_space(5.0);

                // 3. BOTTOM SECTION: Hue Spectrum Bar & Alpha Bar
                changed |= planes::hue_strip(ui, &mut hsva, available_w);
                ui.add_space(4.0);
                changed |= planes::alpha_strip(ui, &mut hsva, available_w);

                if changed {
                    let rgba_unmult = egui::ecolor::Rgba::from(hsva).to_rgba_unmultiplied();
                    *color_rgba = rgba_unmult;
                }
            });

            if windows::popup_dismissed(ui.ctx(), frame_resp.response.rect) {
                is_open = false;
            }
        });

    ui.data_mut(|d| {
        d.insert_temp(popup_open_id, is_open);
        d.insert_temp(hsva_id, hsva);
        d.insert_temp(gamma_space_id, gamma_space);
    });
}

/// The R, G, B and A labels in their colors.
const CHANNELS: [(&str, Color32); 4] = [
    ("R", Color32::from_rgb(255, 100, 100)),
    ("G", Color32::from_rgb(100, 255, 100)),
    ("B", Color32::from_rgb(100, 150, 255)),
    ("A", Color32::from_gray(180)),
];

/// A numeric input per channel of `rgba`, in bytes or fractions by
/// `space`; whether one changed.
fn channel_inputs(ui: &mut egui::Ui, space: ColorGammaSpace, rgba: &mut [f32; 4]) -> bool {
    let mut changed = false;
    for (channel, (name, color)) in rgba.iter_mut().zip(CHANNELS) {
        ui.label(egui::RichText::new(name).small().strong().color(color));
        changed |= match space {
            ColorGammaSpace::Byte => {
                let mut byte = (*channel * 255.0).round() as u8;
                let edited = ui.add(byte_drag(&mut byte)).changed();
                if edited {
                    *channel = f32::from(byte) / 255.0;
                }
                edited
            }
            ColorGammaSpace::Float => ui.add(unit_drag(channel)).changed(),
        };
    }
    changed
}

/// A 0-255 channel input, reading typed numbers in the UI language.
fn byte_drag(value: &mut u8) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .localized()
        .range(0..=255)
        .speed(1.0)
}

/// A 0-1 channel input, reading typed numbers in the UI language.
fn unit_drag(value: &mut f32) -> egui::DragValue<'_> {
    egui::DragValue::new(value)
        .localized()
        .range(0.0..=1.0)
        .speed(0.01)
        .max_decimals(2)
}
