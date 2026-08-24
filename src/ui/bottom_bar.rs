use crate::app::OctantApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum BottomBarItem {
    PlayPause,
    PrevNext,
    Loop,
    Status,
    DateInfo,
    StartDate,
    StepSize,
    EndDate,
    Fps,
    OverflowBtn,
}

impl BottomBarItem {
    fn default_width(self) -> f32 {
        match self {
            BottomBarItem::PlayPause => 70.0,
            BottomBarItem::PrevNext => 55.0,
            BottomBarItem::Loop => 70.0,
            BottomBarItem::Status => 70.0,
            BottomBarItem::DateInfo => 190.0,
            BottomBarItem::StartDate => 85.0,
            BottomBarItem::StepSize => 75.0,
            BottomBarItem::EndDate => 85.0,
            BottomBarItem::Fps => 75.0,
            BottomBarItem::OverflowBtn => 36.0,
        }
    }
}

pub fn show_bottom_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    // Extract to a local bool to avoid split-borrow: we can't hold &mut app.field
    // AND also borrow all of app inside the closure at the same time.
    let mut expanded = app.show_bottom_bar;

    egui::Panel::show_switched(
        ui,
        &mut expanded,
        egui::Panel::bottom("octant_bottom_bar_collapsed")
            .resizable(true)
            .exact_size(20.0),
        egui::Panel::bottom("octant_bottom_bar_expanded")
            .resizable(true)
            .size_range(38.0..=80.0),
        |ui, is_expanded| {
            if is_expanded {
                show_bottom_bar_content(app, ui);
            } else {
                ui.vertical_centered(|ui| {
                    ui.small("▲ Playback (drag to expand)");
                });
            }
        },
    );

    app.show_bottom_bar = expanded;
}

fn show_bottom_bar_content(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let storage_id = egui::Id::new(("bottom_bar", "measured_widths"));
        let mut widths: std::collections::HashMap<BottomBarItem, f32> = ui
            .ctx()
            .data(|d| d.get_temp(storage_id))
            .unwrap_or_default();

        let get_w =
            |item: BottomBarItem, map: &std::collections::HashMap<BottomBarItem, f32>| -> f32 {
                map.get(&item)
                    .copied()
                    .unwrap_or_else(|| item.default_width())
            };

        let play_pause_w = get_w(BottomBarItem::PlayPause, &widths);
        let prev_next_w = get_w(BottomBarItem::PrevNext, &widths);
        let loop_w = get_w(BottomBarItem::Loop, &widths);
        let status_w = get_w(BottomBarItem::Status, &widths);
        let date_info_w = get_w(BottomBarItem::DateInfo, &widths);
        let start_date_w = get_w(BottomBarItem::StartDate, &widths);
        let step_size_w = get_w(BottomBarItem::StepSize, &widths);
        let end_date_w = get_w(BottomBarItem::EndDate, &widths);
        let fps_w = get_w(BottomBarItem::Fps, &widths);
        let overflow_btn_w = get_w(BottomBarItem::OverflowBtn, &widths);
        let spacing = ui.spacing().item_spacing.x;
        let total_width = ui.available_width();

        let min_slider_w = 80.0;

        // Full width needed for all features without overflow
        let full_needed = play_pause_w
            + prev_next_w
            + loop_w
            + status_w
            + date_info_w
            + start_date_w
            + min_slider_w
            + step_size_w
            + end_date_w
            + fps_w
            + spacing * 14.0;

        let (
            show_date_info,
            show_date_badges,
            show_status,
            show_loop,
            show_fps,
            show_prev_next,
            show_overflow,
        ) = if total_width >= full_needed {
            (true, true, true, true, true, true, false)
        } else {
            let base_fixed = play_pause_w + overflow_btn_w + min_slider_w + spacing * 4.0;
            let mut remaining = (total_width - base_fixed).max(0.0);

            let show_prev_next = if remaining >= prev_next_w + spacing {
                remaining -= prev_next_w + spacing;
                true
            } else {
                false
            };

            let show_fps = if remaining >= fps_w + spacing {
                remaining -= fps_w + spacing;
                true
            } else {
                false
            };

            let show_loop = if remaining >= loop_w + spacing {
                remaining -= loop_w + spacing;
                true
            } else {
                false
            };

            let show_status = if remaining >= status_w + spacing {
                remaining -= status_w + spacing;
                true
            } else {
                false
            };

            let show_date_badges =
                if remaining >= start_date_w + step_size_w + end_date_w + spacing * 3.0 {
                    remaining -= start_date_w + step_size_w + end_date_w + spacing * 3.0;
                    true
                } else {
                    false
                };

            let show_date_info = remaining >= date_info_w + spacing;

            (
                show_date_info,
                show_date_badges,
                show_status,
                show_loop,
                show_fps,
                show_prev_next,
                true,
            )
        };

        // 1. Play / Pause Button for Timestep Animation across all plot types
        let play_resp = ui.scope(|ui| {
            let play_text = if app.is_playing {
                "⏸ Pause"
            } else {
                "▶ Play"
            };
            let play_button = egui::Button::new(egui::RichText::new(play_text).strong());
            if ui.add(play_button).clicked() {
                app.is_playing = !app.is_playing;
                app.last_step_time = std::time::Instant::now();
            }
        });
        widths.insert(BottomBarItem::PlayPause, play_resp.response.rect.width());

        let max_steps = app.animated_dim_extent();
        let slider_max = max_steps.saturating_sub(1);

        // 2. Prev / Next Step Buttons
        if show_prev_next {
            let prev_next_resp = ui.scope(|ui| {
                if ui.button("◀").on_hover_text("Previous Step").clicked() {
                    app.step_prev();
                }
                if ui.button("▶").on_hover_text("Next Step").clicked() {
                    app.step_next();
                }
            });
            widths.insert(
                BottomBarItem::PrevNext,
                prev_next_resp.response.rect.width(),
            );
        }

        // 3. Loop Toggle
        if show_loop {
            let loop_resp = ui.scope(|ui| {
                ui.checkbox(&mut app.loop_playback, "🔄 Loop");
            });
            widths.insert(BottomBarItem::Loop, loop_resp.response.rect.width());
        }

        // 4. Status Indicator
        let status_text = if app.is_playing {
            "▶ Playing"
        } else {
            "⏸ Paused"
        };
        if show_status {
            let status_resp = ui.scope(|ui| {
                ui.separator();
                let status_color = if app.is_playing {
                    egui::Color32::from_rgb(255, 99, 71)
                } else {
                    egui::Color32::LIGHT_GRAY
                };
                ui.label(egui::RichText::new(status_text).small().color(status_color));
                ui.separator();
            });
            widths.insert(BottomBarItem::Status, status_resp.response.rect.width());
        }

        // 5. Dimension-Agnostic Axis Reading
        let anim_dim_idx = app.plotted_animated_dim.unwrap_or(0);
        let (active_anim_dim, active_dim_name, active_units, time_start, temp_res) =
            if let Some(plotted_var_info) = app
                .plotted_dataset_metadata
                .as_ref()
                .and_then(|m| m.variables.get(app.plotted_variable_idx))
            {
                let name = plotted_var_info
                    .dimension_names
                    .get(anim_dim_idx)
                    .cloned()
                    .unwrap_or_else(|| "step".to_string());
                (
                    name.clone(),
                    Some(name),
                    plotted_var_info.units.clone(),
                    plotted_var_info.time_coverage_start.clone(),
                    plotted_var_info.temporal_resolution.clone(),
                )
            } else {
                ("step".to_string(), None, None, None, None)
            };

        let dim_coords = app
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.dimension_coordinates.get(&active_anim_dim.to_lowercase()));

        let direct_coord_label =
            dim_coords.and_then(|coords| coords.get(app.current_timestep).cloned());
        let first_coord = dim_coords.and_then(|coords| coords.first().cloned());
        let last_coord = dim_coords.and_then(|coords| coords.last().cloned());

        let format_dim_step = |step: usize, default_coord: Option<String>| -> String {
            if let Some(ref coord) = default_coord {
                let is_raw_numeric = coord.parse::<f64>().is_ok()
                    && !coord.contains('-')
                    && !coord.contains(':')
                    && !coord.contains('/')
                    && !coord.contains('T');

                if !is_raw_numeric && !coord.trim().is_empty() {
                    return coord.clone();
                }
            }

            crate::utils::units::format_axis_value(
                step,
                max_steps,
                active_dim_name.as_deref(),
                active_units.as_deref(),
                time_start.as_deref(),
                temp_res.as_deref(),
                Some(&app.plotted_store_target_input),
            )
        };

        let formatted_axis = format_dim_step(app.current_timestep, direct_coord_label);
        let start_date_str = format_dim_step(0, first_coord);
        let end_date_str = format_dim_step(max_steps.saturating_sub(1), last_coord);
        let step_size_str = temp_res.unwrap_or_else(|| "Step: 1".to_string());

        if show_date_info {
            let date_resp = ui.scope(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "📅 {} | Current: {}",
                        start_date_str, formatted_axis
                    ))
                    .small()
                    .monospace()
                    .strong(),
                )
                .on_hover_text(format!(
                    "Start: {} | End: {} | {}",
                    start_date_str, end_date_str, step_size_str
                ));
            });
            widths.insert(BottomBarItem::DateInfo, date_resp.response.rect.width());
        }

        if show_date_badges {
            let start_badge_resp = ui.scope(|ui| {
                ui.small(format!("[{}]", start_date_str));
            });
            widths.insert(
                BottomBarItem::StartDate,
                start_badge_resp.response.rect.width(),
            );
        }

        // 6. Step Timeline Slider (Dynamically stretched across available space)
        let right_elements_w = (if show_date_badges {
            step_size_w + end_date_w + spacing * 2.0
        } else {
            0.0
        }) + (if show_fps { fps_w + spacing + 8.0 } else { 0.0 })
            + (if show_overflow {
                overflow_btn_w + spacing
            } else {
                0.0
            });

        let slider_w = (ui.available_width() - right_elements_w - spacing * 2.0).max(min_slider_w);
        ui.spacing_mut().slider_width = slider_w;

        let mut displayed_step = app.current_timestep;
        let slider_res = ui.add(
            egui::Slider::new(&mut displayed_step, 0..=slider_max)
                .show_value(false)
                .trailing_fill(true),
        );

        if slider_res.changed() {
            app.request_step_or_load(displayed_step);
        }

        if show_date_badges {
            let step_size_resp = ui.scope(|ui| {
                ui.small(format!("⏱ {}", step_size_str));
            });
            widths.insert(
                BottomBarItem::StepSize,
                step_size_resp.response.rect.width(),
            );

            let end_badge_resp = ui.scope(|ui| {
                ui.small(format!("[{}]", end_date_str));
            });
            widths.insert(BottomBarItem::EndDate, end_badge_resp.response.rect.width());
        }

        // 7. Playback Speed Button / Menu
        if show_fps {
            let fps_resp = ui.scope(|ui| {
                ui.separator();
                let fps_int = app.playback_fps.round() as u32;
                let fps_label = match fps_int {
                    1 => "1 FPS",
                    2 => "2 FPS",
                    3 => "3 FPS",
                    4 => "4 FPS",
                    5 => "5 FPS",
                    6 => "6 FPS",
                    8 => "8 FPS",
                    10 => "10 FPS",
                    12 => "12 FPS",
                    15 => "15 FPS",
                    20 => "20 FPS",
                    24 => "24 FPS",
                    30 => "30 FPS",
                    60 => "60 FPS",
                    _ => "",
                };

                let menu_body = |ui: &mut egui::Ui| {
                    ui.label(egui::RichText::new("Playback Speed").strong());
                    ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
                };

                if !fps_label.is_empty() {
                    ui.menu_button(fps_label, menu_body);
                } else {
                    ui.menu_button(format!("{} FPS", fps_int), menu_body);
                }
            });
            widths.insert(BottomBarItem::Fps, fps_resp.response.rect.width());
        }

        // 8. Overflow Button "..."
        if show_overflow {
            let overflow_resp = ui.scope(|ui| {
                ui.menu_button(egui::RichText::new("…").strong(), |ui| {
                    ui.set_min_width(220.0);
                    ui.label(
                        egui::RichText::new("Playback Options & Info")
                            .small()
                            .weak(),
                    );
                    ui.separator();

                    if !show_loop {
                        ui.checkbox(&mut app.loop_playback, "🔄 Loop Playback");
                        ui.separator();
                    }

                    if !show_fps {
                        ui.label(egui::RichText::new("Playback Speed").strong());
                        ui.add(egui::Slider::new(&mut app.playback_fps, 1.0..=60.0).suffix(" FPS"));
                        ui.separator();
                    }

                    if !show_prev_next {
                        ui.horizontal(|ui| {
                            if ui.button("◀ Prev Step").clicked() {
                                app.step_prev();
                            }
                            if ui.button("Next Step ▶").clicked() {
                                app.step_next();
                            }
                        });
                        ui.separator();
                    }

                    // Jump to Start / End
                    ui.horizontal(|ui| {
                        if ui.button("⏮ First Step").clicked() {
                            app.request_step_or_load(0);
                        }
                        if ui.button("⏭ Last Step").clicked() {
                            app.request_step_or_load(slider_max);
                        }
                    });

                    ui.separator();

                    // Detailed Timeline Details
                    ui.label(egui::RichText::new("Timeline Details").strong());
                    ui.label(format!("• Dimension: {}", active_anim_dim));
                    ui.label(format!("• Step: {} / {}", app.current_timestep, slider_max));
                    ui.label(format!("• Current: {}", formatted_axis));
                    ui.label(format!("• Range: {} → {}", start_date_str, end_date_str));
                    ui.label(format!("• Step Size: {}", step_size_str));
                    ui.label(format!("• Status: {}", status_text));
                })
                .response
                .on_hover_text("More playback options and timeline details");
            });
            widths.insert(
                BottomBarItem::OverflowBtn,
                overflow_resp.response.rect.width(),
            );
        }

        ui.ctx().data_mut(|d| d.insert_temp(storage_id, widths));
    });
}
