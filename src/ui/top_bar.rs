use super::{cache, colormap, plot_type, status, store};
use crate::app::OctantApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TopBarItem {
    Brand,
    Store,
    Variables,
    Dimensions,
    PlotType,
    Colormap,
    Settings,
    Cache,
    Theme,
    Status,
    OverflowBtn,
}

impl TopBarItem {
    fn default_width(self) -> f32 {
        match self {
            TopBarItem::Brand => 115.0,
            TopBarItem::Store => 80.0,
            TopBarItem::Variables => 105.0,
            TopBarItem::Dimensions => 120.0,
            TopBarItem::PlotType => 180.0,
            TopBarItem::Colormap => 105.0,
            TopBarItem::Settings => 95.0,
            TopBarItem::Cache => 85.0,
            TopBarItem::Theme => 75.0,
            TopBarItem::Status => 230.0,
            TopBarItem::OverflowBtn => 36.0,
        }
    }
}

pub fn show_top_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::Panel::top("octant_top_bar")
        .exact_size(34.0)
        .show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                let storage_id = egui::Id::new(("top_bar", "measured_widths"));
                let mut widths: std::collections::HashMap<TopBarItem, f32> = ui
                    .ctx()
                    .data(|d| d.get_temp(storage_id))
                    .unwrap_or_default();

                let get_w =
                    |item: TopBarItem, map: &std::collections::HashMap<TopBarItem, f32>| -> f32 {
                        map.get(&item)
                            .copied()
                            .unwrap_or_else(|| item.default_width())
                    };

                let left_items = [
                    TopBarItem::Store,
                    TopBarItem::Variables,
                    TopBarItem::Dimensions,
                    TopBarItem::PlotType,
                    TopBarItem::Colormap,
                    TopBarItem::Settings,
                ];

                let total_width = ui.available_width();
                let spacing = ui.spacing().item_spacing.x;

                let show_right_status = app.block_prefetcher.pending_count() > 0;
                let mut show_right_cache = true;
                let mut show_right_theme = true;

                let mut right_needed = 0.0;
                if show_right_status {
                    right_needed += get_w(TopBarItem::Status, &widths) + spacing + 8.0;
                }
                right_needed += get_w(TopBarItem::Cache, &widths) + spacing + 8.0;
                right_needed += get_w(TopBarItem::Theme, &widths);

                let brand_w = get_w(TopBarItem::Brand, &widths);
                let overflow_btn_w = get_w(TopBarItem::OverflowBtn, &widths);

                // If space is extremely constrained, overflow right items as well
                if total_width < brand_w + right_needed + overflow_btn_w + spacing * 2.0 {
                    show_right_cache = false;
                    right_needed -= get_w(TopBarItem::Cache, &widths) + spacing + 8.0;
                }
                if total_width < brand_w + right_needed + overflow_btn_w + spacing * 2.0 {
                    show_right_theme = false;
                    right_needed -= get_w(TopBarItem::Theme, &widths);
                }

                let available_for_left =
                    (total_width - brand_w - right_needed - spacing * 2.0).max(0.0);

                let total_left_needed: f32 =
                    left_items.iter().map(|&it| get_w(it, &widths)).sum::<f32>()
                        + (left_items.len().saturating_sub(1) as f32) * spacing;

                let (num_visible_left, show_overflow) = if total_left_needed <= available_for_left
                    && show_right_cache
                    && show_right_theme
                {
                    (left_items.len(), false)
                } else {
                    let mut acc = overflow_btn_w + spacing;
                    let mut count = 0;
                    for &item in &left_items {
                        let needed = get_w(item, &widths) + spacing;
                        if acc + needed <= available_for_left {
                            acc += needed;
                            count += 1;
                        } else {
                            break;
                        }
                    }
                    (count, true)
                };

                // 1. Octant Brand Header (Icon toggles Hero, Label triggers About window)
                let brand_resp = ui.scope(|ui| {
                    let icon_resp = octant_icon(ui, 24.0)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Toggle Hero / Landing View");
                    if icon_resp.clicked() {
                        app.show_hero = !app.show_hero;
                    }

                    let label_resp = ui
                        .add(
                            egui::Label::new(egui::RichText::new("Octant").strong().heading())
                                .sense(egui::Sense::click()),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("About Octant");

                    if label_resp.clicked() {
                        app.show_about_window = !app.show_about_window;
                    }
                    ui.separator();
                });
                widths.insert(TopBarItem::Brand, brand_resp.response.rect.width());

                // 2. Visible Left Items
                for &item in &left_items[..num_visible_left] {
                    let resp = ui.scope(|ui| {
                        render_item(item, false, app, ui);
                    });
                    widths.insert(item, resp.response.rect.width());
                }

                // 3. Overflow Menu Button "..."
                if show_overflow {
                    let overflow_resp = ui.scope(|ui| {
                        ui.menu_button(egui::RichText::new("…").strong(), |ui| {
                            ui.set_min_width(180.0);
                            ui.label(egui::RichText::new("More Options").small().weak());
                            ui.separator();

                            for &item in &left_items[num_visible_left..] {
                                render_item(item, true, app, ui);
                            }

                            if !show_right_cache || !show_right_theme {
                                ui.separator();
                                if !show_right_cache {
                                    render_item(TopBarItem::Cache, true, app, ui);
                                }
                                if !show_right_theme {
                                    render_item(TopBarItem::Theme, true, app, ui);
                                }
                            }
                        })
                        .response
                        .on_hover_text("More options");
                    });
                    widths.insert(TopBarItem::OverflowBtn, overflow_resp.response.rect.width());
                }

                // 4. Right-aligned items
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if show_right_theme {
                        let resp = ui.scope(|ui| {
                            render_item(TopBarItem::Theme, false, app, ui);
                        });
                        widths.insert(TopBarItem::Theme, resp.response.rect.width());
                    }

                    if show_right_cache {
                        ui.separator();
                        let resp = ui.scope(|ui| {
                            render_item(TopBarItem::Cache, false, app, ui);
                        });
                        widths.insert(TopBarItem::Cache, resp.response.rect.width());
                    }

                    if show_right_status {
                        ui.separator();
                        let resp = ui.scope(|ui| {
                            status::show_status_bar(app, ui);
                        });
                        widths.insert(TopBarItem::Status, resp.response.rect.width());
                    }
                });

                ui.ctx().data_mut(|d| d.insert_temp(storage_id, widths));
            });
        });
}

fn render_item(item: TopBarItem, in_menu: bool, app: &mut OctantApp, ui: &mut egui::Ui) {
    match item {
        TopBarItem::Brand | TopBarItem::OverflowBtn | TopBarItem::Status => {}
        TopBarItem::Store => {
            if in_menu {
                if ui
                    .button(egui::RichText::new("🌐 Store").strong())
                    .clicked()
                {
                    app.show_left_panel = !app.show_left_panel;
                    ui.close();
                }
            } else {
                store::show_store_menu(app, ui);
            }
        }
        TopBarItem::Variables => {
            let resp = ui.button(egui::RichText::new("📊 Variables").strong());
            if resp.clicked() {
                app.show_variables_overlay = !app.show_variables_overlay;
                if in_menu {
                    ui.close();
                }
            }
        }
        TopBarItem::Dimensions => {
            let resp = ui
                .button(egui::RichText::new("🎛️ Dimensions").strong())
                .on_hover_text("Toggle Variable Controls Panel");
            if resp.clicked() {
                app.show_variable_controls = !app.show_variable_controls;
                if in_menu {
                    ui.close();
                }
            }
        }
        TopBarItem::PlotType => {
            plot_type::show_plot_type_menu(app, ui);
        }
        TopBarItem::Colormap => {
            colormap::show_colormap_menu(app, ui);
        }
        TopBarItem::Settings => {
            let resp = ui.button(egui::RichText::new("⚙️ Settings").strong());
            if resp.clicked() {
                app.show_settings_panel = !app.show_settings_panel;
                if in_menu {
                    ui.close();
                }
            }
        }
        TopBarItem::Cache => {
            cache::show_cache_menu(app, ui);
        }
        TopBarItem::Theme => {
            let is_dark = app.theme_preference == egui::ThemePreference::Dark;
            let theme_label = if is_dark { "☀ Light" } else { "🌙 Dark" };
            let theme_hover = if is_dark {
                "Switch to Light mode"
            } else {
                "Switch to Dark mode"
            };
            if ui
                .button(egui::RichText::new(theme_label))
                .on_hover_text(theme_hover)
                .clicked()
            {
                app.theme_preference = if is_dark {
                    egui::ThemePreference::Light
                } else {
                    egui::ThemePreference::Dark
                };
                ui.ctx().set_theme(app.theme_preference);
                if in_menu {
                    ui.close();
                }
            }
        }
    }
}

fn octant_icon(ui: &mut egui::Ui, size: f32) -> egui::Response {
    crate::ui::hero::draw_octant_widget(ui, size, [-1.0, -1.0, -1.0], 0.0, 1.0)
}
