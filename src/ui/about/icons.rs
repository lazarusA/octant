//! Native procedural vector icons gallery tab for the About Octant dialog.

use super::types::ICON_CATEGORIES;
use crate::ui::icons::{Icon, UiIconExt};

pub fn show_icons_tab(ui: &mut egui::Ui) {
    let search_id = egui::Id::new(("about_icons", "search_query"));
    let mut search_query: String = ui.ctx().data(|d| d.get_temp(search_id)).unwrap_or_default();

    let cat_id = egui::Id::new(("about_icons", "category_filter"));
    let mut selected_cat: usize = ui.ctx().data(|d| d.get_temp(cat_id)).unwrap_or(0);

    let copied_id = egui::Id::new(("about_icons", "copied_feedback"));
    let copied_feedback: Option<(String, web_time::Instant)> =
        ui.ctx().data(|d| d.get_temp(copied_id));

    ui.horizontal(|ui| {
        ui.icon(Icon::Search, 13.0);
        let has_text = !search_query.is_empty();
        let edit_resp = ui.add(
            egui::TextEdit::singleline(&mut search_query)
                .hint_text("Search icons...")
                .desired_width(180.0),
        );
        if edit_resp.changed() {
            ui.ctx()
                .data_mut(|d| d.insert_temp(search_id, search_query.clone()));
        }
        if has_text && ui.icon_button(Icon::Cross, "").clicked() {
            search_query.clear();
            ui.ctx()
                .data_mut(|d| d.insert_temp(search_id, String::new()));
        }

        ui.separator();

        egui::ComboBox::from_id_salt("about_icon_category_combo")
            .selected_text(ICON_CATEGORIES[selected_cat])
            .show_ui(ui, |ui| {
                for (idx, &cat_name) in ICON_CATEGORIES.iter().enumerate() {
                    if ui
                        .selectable_value(&mut selected_cat, idx, cat_name)
                        .clicked()
                    {
                        ui.ctx().data_mut(|d| d.insert_temp(cat_id, selected_cat));
                    }
                }
            });

        if let Some((ref name, timestamp)) = copied_feedback
            && timestamp.elapsed().as_secs_f32() < 2.0
        {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Copied Icon::{name}"))
                            .small()
                            .color(ui.visuals().selection.bg_fill),
                    );
                    ui.icon_colored(Icon::Check, 12.0, ui.visuals().selection.bg_fill);
                });
            });
        }
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    let query_trimmed = search_query.trim();
    let current_cat_filter = ICON_CATEGORIES[selected_cat];

    let icon_matches = |icon: &Icon| -> bool {
        let matches_cat = current_cat_filter == "All" || icon.category() == current_cat_filter;
        let matches_search = query_trimmed.is_empty()
            || contains_ignore_ascii_case(icon.name(), query_trimmed)
            || contains_ignore_ascii_case(icon.category(), query_trimmed);
        matches_cat && matches_search
    };

    let total_matches = Icon::ALL.iter().filter(|i| icon_matches(i)).count();

    if total_matches == 0 {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.icon_colored(Icon::Info, 18.0, ui.visuals().weak_text_color());
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("No matching vector icons found.")
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(30.0);
        });
    } else {
        egui::ScrollArea::vertical()
            .max_height(380.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let active_categories: &[&str] = if current_cat_filter == "All" {
                    &ICON_CATEGORIES[1..]
                } else {
                    &ICON_CATEGORIES[selected_cat..=selected_cat]
                };

                for &cat in active_categories {
                    let cat_matches_count = Icon::ALL
                        .iter()
                        .filter(|icon| icon.category() == cat && icon_matches(icon))
                        .count();

                    if cat_matches_count == 0 {
                        continue;
                    }

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("{cat} ({cat_matches_count})"))
                            .strong()
                            .small(),
                    );
                    ui.add_space(2.0);

                    egui::Frame::default()
                        .fill(ui.visuals().extreme_bg_color)
                        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for icon in Icon::ALL {
                                    if icon.category() != cat || !icon_matches(icon) {
                                        continue;
                                    }

                                    let icon_name = icon.name();
                                    let is_dark = ui.visuals().dark_mode;

                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(66.0, 52.0),
                                        egui::Sense::click(),
                                    );

                                    if ui.is_rect_visible(rect) {
                                        let bg_fill = if resp.hovered() {
                                            ui.visuals().widgets.hovered.bg_fill
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        };

                                        let stroke = if resp.hovered() {
                                            ui.visuals().widgets.hovered.bg_stroke
                                        } else {
                                            egui::Stroke::NONE
                                        };

                                        ui.painter().rect(
                                            rect,
                                            4.0,
                                            bg_fill,
                                            stroke,
                                            egui::StrokeKind::Inside,
                                        );

                                        let icon_size = 20.0;
                                        let icon_rect = egui::Rect::from_center_size(
                                            egui::pos2(rect.center().x, rect.top() + 16.0),
                                            egui::vec2(icon_size, icon_size),
                                        );

                                        let icon_color = if resp.hovered() {
                                            ui.visuals().widgets.hovered.text_color()
                                        } else {
                                            ui.visuals().text_color()
                                        };

                                        icon.paint(ui.painter(), icon_rect, icon_color, is_dark);

                                        let text_rect = egui::Rect::from_min_max(
                                            egui::pos2(rect.left() + 2.0, rect.top() + 28.0),
                                            egui::pos2(rect.right() - 2.0, rect.bottom() - 2.0),
                                        );

                                        ui.painter().text(
                                            text_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            icon_name,
                                            egui::FontId::proportional(10.0),
                                            icon_color,
                                        );
                                    }

                                    if resp.clicked() {
                                        let code_snippet = format!("Icon::{icon_name}");
                                        ui.ctx().copy_text(code_snippet);
                                        ui.ctx().data_mut(|d| {
                                            d.insert_temp(
                                                copied_id,
                                                Some((
                                                    icon_name.to_string(),
                                                    web_time::Instant::now(),
                                                )),
                                            );
                                        });
                                    }

                                    resp.on_hover_ui(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("Icon::{icon_name}"))
                                                .strong(),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "Category: {}",
                                                icon.category()
                                            ))
                                            .small()
                                            .weak(),
                                        );
                                        ui.small("Click to copy enum identifier");
                                    });

                                    ui.add_space(2.0);
                                }
                            });
                        });
                }
            });
    }
}

#[inline]
fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))
}
