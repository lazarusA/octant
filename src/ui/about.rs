use crate::{
    app::OctantApp,
    ui::icons::{Icon, UiIconExt},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AboutTab {
    Overview,
    Icons,
}

pub fn show_about_window(app: &mut OctantApp, ctx: &egui::Context) {
    if app.show_icon_gallery_window {
        app.show_about_window = true;
    }

    if !app.show_about_window {
        return;
    }

    let storage_id = egui::Id::new(("about_window", "active_tab"));
    let mut active_tab: AboutTab = ctx
        .data(|d| d.get_temp(storage_id))
        .unwrap_or(AboutTab::Overview);

    if app.show_icon_gallery_window {
        active_tab = AboutTab::Icons;
        app.show_icon_gallery_window = false;
    }

    let mut open = app.show_about_window;

    let screen_size = ctx.viewport_rect().size();
    let max_screen_h = (screen_size.y * 0.85).max(320.0);

    let (default_size, min_size, max_size) = match active_tab {
        AboutTab::Overview => (
            egui::vec2(450.0, (screen_size.y * 0.65).clamp(360.0, 520.0)),
            egui::vec2(360.0, 260.0),
            egui::vec2(720.0, max_screen_h),
        ),
        AboutTab::Icons => (
            egui::vec2(560.0, (screen_size.y * 0.75).clamp(420.0, 600.0)),
            egui::vec2(420.0, 280.0),
            egui::vec2(840.0, max_screen_h),
        ),
    };

    let title = match active_tab {
        AboutTab::Overview => "About Octant",
        AboutTab::Icons => "Native Vector Icons",
    };

    let response = egui::Window::new(title)
        .open(&mut open)
        .default_size(default_size)
        .min_size(min_size)
        .max_size(max_size)
        .resizable(true)
        .collapsible(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::Frame::default()
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    // Header navigation bar with Tab Switcher
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                active_tab == AboutTab::Overview,
                                egui::RichText::new("Overview").strong(),
                            )
                            .clicked()
                        {
                            active_tab = AboutTab::Overview;
                        }

                        let icons_tab_text = format!("Vector Icons ({})", Icon::ALL.len());
                        if ui
                            .selectable_label(
                                active_tab == AboutTab::Icons,
                                egui::RichText::new(icons_tab_text).strong(),
                            )
                            .clicked()
                        {
                            active_tab = AboutTab::Icons;
                        }
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(6.0);

                    match active_tab {
                        AboutTab::Overview => {
                            show_overview_tab(app, ui, &mut active_tab);
                        }
                        AboutTab::Icons => {
                            show_icons_tab(ui);
                        }
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Footer
                    ui.horizontal(|ui| {
                        ui.small("Licensed under MIT or Apache-2.0");
                    });
                });
        });

    ctx.data_mut(|d| d.insert_temp(storage_id, active_tab));

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }

    if let Some(r) = response
        && let Some(pos) = ctx.input(|i| i.pointer.interact_pos())
        && ctx.input(|i| i.pointer.primary_pressed())
        && !r.response.rect.contains(pos)
    {
        open = false;
    }

    app.show_about_window = open;
}

pub fn show_icon_gallery_window(app: &mut OctantApp, ctx: &egui::Context) {
    if app.show_icon_gallery_window {
        app.show_about_window = true;
        show_about_window(app, ctx);
    }
}

fn show_overview_tab(_app: &mut OctantApp, ui: &mut egui::Ui, active_tab: &mut AboutTab) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.add_space(2.0);

            // Centered Header with Animated Octant Cube Widget & Title
            ui.vertical_centered(|ui| {
                crate::ui::hero::draw_octant_widget(ui, 42.0, [-1.0, -1.0, -1.0], 0.0, 1.0);
                ui.add_space(4.0);
                ui.heading(
                    egui::RichText::new(format!("Octant v{}", env!("CARGO_PKG_VERSION"))).strong(),
                );
                ui.label(
                    egui::RichText::new("N-Dimensional Data Explorer")
                        .small()
                        .italics()
                        .color(ui.visuals().weak_text_color()),
                );
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Core Platform Description
            use egui::special_emojis::{OS_APPLE, OS_LINUX, OS_WINDOWS};
            ui.label(format!(
                "Octant is an interactive viewer for N-dimensional datasets with native support for local and cloud object storage, Zarr (v2/v3), and Icechunk. Built in Rust with GPU-accelerated rendering via WGPU. Octant runs natively on {}{}{}.",
                OS_APPLE, OS_LINUX, OS_WINDOWS
            ));

            ui.add_space(10.0);

            // Key Specs
            egui::Frame::default()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.horizontal(|ui| {
                        ui.icon(Icon::Bolt, 13.0);
                        ui.label(egui::RichText::new("Hyperslab Slicing").strong());
                    });
                    ui.label("   Async LRU chunk cache & multi-resolution pyramids.");
                    ui.horizontal(|ui| {
                        ui.icon(Icon::Icechunk, 13.0);
                        ui.label(egui::RichText::new("Zarr & Icechunk Native").strong());
                    });
                    ui.label("   Local, S3, GCS, Azure, and HTTP streaming backends.");
                    ui.horizontal(|ui| {
                        ui.icon(Icon::Colormap, 13.0);
                        ui.label(
                            egui::RichText::new("Hardware-Accelerated WGPU Shaders").strong(),
                        );
                    });
                    ui.label(
                        "   2D Flatmaps, 3D Spheres, Elevation Surfaces, Volumes & 1D Profiles.",
                    );
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Links & Resources
            ui.heading("Links & Resources");
            ui.add_space(4.0);

            use egui::special_emojis::GITHUB;
            ui.horizontal_wrapped(|ui| {
                ui.hyperlink_to(
                    format!("{GITHUB} github.com/lazarusA/octant"),
                    "https://github.com/lazarusA/octant",
                );
            });
            ui.horizontal_wrapped(|ui| {
                ui.icon(Icon::Catalog, 12.0);
                ui.hyperlink_to("octant documentation", "https://docs.rs/octant");
            });
            ui.horizontal_wrapped(|ui| {
                ui.icon(Icon::Globe, 12.0);
                ui.hyperlink_to("@lazarusA", "https://github.com/lazarusA");
            });

            ui.add_space(8.0);
            if ui
                .icon_button(
                    Icon::Colormap,
                    format!("Browse Native Vector Icons ({}) →", Icon::ALL.len()),
                )
                .on_hover_text("Explore procedural vector icons in popover")
                .clicked()
            {
                *active_tab = AboutTab::Icons;
            }
        });
}

fn show_icons_tab(ui: &mut egui::Ui) {
    let search_id = egui::Id::new(("about_icons", "search_query"));
    let mut search_query: String = ui.ctx().data(|d| d.get_temp(search_id)).unwrap_or_default();

    let cat_id = egui::Id::new(("about_icons", "category_filter"));
    let mut selected_cat: usize = ui.ctx().data(|d| d.get_temp(cat_id)).unwrap_or(0);

    let copied_id = egui::Id::new(("about_icons", "copied_feedback"));
    let copied_feedback: Option<(String, std::time::Instant)> =
        ui.ctx().data(|d| d.get_temp(copied_id));

    // Search and Category Filters
    let categories = [
        "All",
        "Navigation & Menus",
        "Playback & Timeline",
        "Plot Types & Colormaps",
        "Data Store & Files",
        "Tools & Status Badges",
    ];

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
        if has_text && ui.small_button("✕").clicked() {
            search_query.clear();
            ui.ctx()
                .data_mut(|d| d.insert_temp(search_id, String::new()));
        }

        ui.separator();

        egui::ComboBox::from_id_salt("about_icon_category_combo")
            .selected_text(categories[selected_cat])
            .show_ui(ui, |ui| {
                for (idx, &cat_name) in categories.iter().enumerate() {
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
                ui.label(
                    egui::RichText::new(format!("✓ Copied Icon::{name}"))
                        .small()
                        .color(ui.visuals().selection.bg_fill),
                );
            });
        }
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    let query_lower = search_query.trim().to_lowercase();
    let current_cat_filter = categories[selected_cat];

    let filtered_icons: Vec<Icon> = Icon::ALL
        .iter()
        .copied()
        .filter(|icon| {
            let matches_cat = current_cat_filter == "All" || icon.category() == current_cat_filter;
            let matches_search = query_lower.is_empty()
                || icon.name().to_lowercase().contains(&query_lower)
                || icon.category().to_lowercase().contains(&query_lower);
            matches_cat && matches_search
        })
        .collect();

    if filtered_icons.is_empty() {
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
                // Group by category if "All" is selected, otherwise render directly
                let active_categories: Vec<&'static str> = if current_cat_filter == "All" {
                    categories[1..].to_vec()
                } else {
                    vec![current_cat_filter]
                };

                for cat in active_categories {
                    let cat_icons: Vec<Icon> = filtered_icons
                        .iter()
                        .copied()
                        .filter(|icon| icon.category() == cat)
                        .collect();

                    if cat_icons.is_empty() {
                        continue;
                    }

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("{} ({})", cat, cat_icons.len()))
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
                                for icon in cat_icons {
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
                                                    std::time::Instant::now(),
                                                )),
                                            );
                                        });
                                    }

                                    let tooltip = format!(
                                        "Icon::{icon_name}\nCategory: {}\n(Click to copy code)",
                                        icon.category()
                                    );
                                    resp.on_hover_text(tooltip);

                                    ui.add_space(2.0);
                                }
                            });
                        });
                }
            });
    }
}
