//! Floating theme-aware toolbar for interactive ROI cropping.

use crate::export::{AspectPreset, RoiCropBox};
use crate::i18n::{CommonKey, ExportKey, tr};
use crate::ui::icons::{Icon, IconSize, ToolbarButton, UiIconExt};
use egui::{Color32, Rect};

/// Actions dispatched from the interactive Crop Toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CropOverlayAction {
    Save,
    Reset,
    Done,
}

pub fn render_crop_toolbar(
    ui: &mut egui::Ui,
    canvas_rect: Rect,
    box_rect: Rect,
    crop_box: &mut RoiCropBox,
    is_open: &mut bool,
    accent_color: Color32,
) -> Option<CropOverlayAction> {
    let mut action = None;
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
                        ui.icon(Icon::Scissors, IconSize::Xs);
                        ui.label(
                            egui::RichText::new(tr(ExportKey::RoiLabel))
                                .small()
                                .strong(),
                        );

                        let w_px = (box_rect.width()).round() as u32;
                        let h_px = (box_rect.height()).round() as u32;
                        let mut size = [0; 64];
                        let size =
                            crate::i18n::fill(&mut size, tr(ExportKey::RoiSize), &[&w_px, &h_px]);
                        ui.label(egui::RichText::new(size).small().color(accent_color));

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
                            .add(
                                ToolbarButton::new(Icon::Save, tr(CommonKey::Save))
                                    .hover(tr(ExportKey::CropSaveHover)),
                            )
                            .clicked()
                        {
                            action = Some(CropOverlayAction::Save);
                        }

                        if ui
                            .add(
                                ToolbarButton::new(Icon::Reset, tr(CommonKey::Reset))
                                    .hover(tr(ExportKey::CropResetHover)),
                            )
                            .clicked()
                        {
                            *crop_box = RoiCropBox::default();
                            action = Some(CropOverlayAction::Reset);
                        }

                        if ui
                            .add(
                                ToolbarButton::new(Icon::Check, tr(ExportKey::CropDone))
                                    .hover(tr(ExportKey::CropDoneHover)),
                            )
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
