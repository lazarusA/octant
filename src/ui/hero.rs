use std::time::Duration;
use web_time::Instant;

use crate::app::OctantApp;
use crate::utils::{ease_in_out_cubic, lerp3, xorshift64_f32};

// ---------------------------------------------------------------------
// The 8 octants, visited in Gray-code order so every hop moves to a
// face-adjacent neighbor (exactly one axis flips per step).
// ---------------------------------------------------------------------

pub const SEQUENCE: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [0.0, -1.0, -1.0],
    [0.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [-1.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
];

// ---------------------------------------------------------------------
// Hero State
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HeroState {
    pub input: String,
    pub current: usize,
    pub anim_from: [f32; 3],
    pub anim_to: [f32; 3],
    pub anim_start: Option<Instant>,
    pub anim_duration: Duration,
    pub next_hop_at: Instant,
    pub loading: bool,
    pub hops_left: u32,
    pub loaded: bool,
    pub source_label: String,
    pub rng_seed: u64,
}

impl Default for HeroState {
    fn default() -> Self {
        Self {
            input: String::new(),
            current: 0,
            anim_from: SEQUENCE[0],
            anim_to: SEQUENCE[0],
            anim_start: None,
            anim_duration: Duration::from_millis(1700),
            next_hop_at: Instant::now() + Duration::from_secs(9),
            loading: false,
            hops_left: 0,
            loaded: false,
            source_label: String::new(),
            rng_seed: 0x123456789abcdef0,
        }
    }
}

impl HeroState {
    pub fn begin_submit(&mut self, source_name: &str) {
        if self.loading {
            return;
        }

        self.source_label = source_name.trim().to_owned();
        self.loading = true;
        self.loaded = false;
        self.hops_left = 5;
        self.start_hop(Duration::from_millis(240));
    }

    pub fn start_hop(&mut self, duration: Duration) {
        let next = (self.current + 1) % SEQUENCE.len();
        self.anim_from = SEQUENCE[self.current];
        self.anim_to = SEQUENCE[next];
        self.anim_duration = duration;
        self.anim_start = Some(Instant::now());
    }

    pub fn schedule_next_wander(&mut self) {
        let r = xorshift64_f32(&mut self.rng_seed);
        let delay = 8.0 + r * 7.0; // 8.0 to 15.0 seconds
        self.next_hop_at = Instant::now() + Duration::from_secs_f32(delay);
    }

    /// Advance animation physics and return (filled_corner, extra_rot, extra_scale).
    pub fn update_animation(&mut self, now: Instant) -> ([f32; 3], f32, f32) {
        if let Some(start) = self.anim_start {
            let dur_secs = self.anim_duration.as_secs_f32().max(0.001);
            let t = now.duration_since(start).as_secs_f32() / dur_secs;

            if t >= 1.0 {
                self.current = (self.current + 1) % SEQUENCE.len();
                self.anim_start = None;

                if self.loading {
                    if self.hops_left > 1 {
                        self.hops_left -= 1;
                        self.start_hop(Duration::from_millis(240));
                    } else {
                        self.loading = false;
                        self.loaded = true;
                        self.schedule_next_wander();
                    }
                } else {
                    self.schedule_next_wander();
                }

                (SEQUENCE[self.current], 0.0, 1.0)
            } else {
                let eased = ease_in_out_cubic(t.clamp(0.0, 1.0));
                let f = lerp3(self.anim_from, self.anim_to, eased);
                // gentle scale-down/up + slight rotation while it's in transit
                let wobble = (std::f32::consts::PI * t).sin();
                (f, wobble * 0.16, 1.0 - wobble * 0.09)
            }
        } else {
            if !self.loading && now >= self.next_hop_at {
                self.start_hop(Duration::from_millis(1700));
            }
            (SEQUENCE[self.current], 0.0, 1.0)
        }
    }
}

// ---------------------------------------------------------------------
// Hero Landing UI Rendering
// ---------------------------------------------------------------------

/// Render the clean, centered Hero Landing page.
pub fn show_hero_landing(app: &mut OctantApp, ui: &mut egui::Ui) {
    let now = Instant::now();
    let (filled, extra_rot, extra_scale) = app.hero_state.update_animation(now);

    // Keep animating smoothly at 60 FPS while wandering or loading.
    ui.ctx().request_repaint_after(Duration::from_millis(16));

    let is_drag_hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    if is_drag_hovering {
        ui.ctx().request_repaint();
    }

    // Check warning state from drop handler
    let warning_id = egui::Id::new("drop_zone_warning_state");
    let active_warning: Option<crate::ui::drop_zone::DropZoneWarningState> =
        ui.ctx().data(|d| d.get_temp(warning_id));
    let is_warning_active = active_warning
        .as_ref()
        .is_some_and(|w| w.triggered_at.elapsed() < Duration::from_millis(3500));

    if is_warning_active {
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }

    // Main centered composition with procedural cube, title, and intake
    let available_h = ui.available_height();

    ui.vertical_centered(|ui| {
        ui.add_space((available_h * 0.16).max(20.0));

        // Centered 3D Octant procedural widget (interactive click to hop)
        let octant_resp = draw_octant_widget(ui, 136.0, filled, extra_rot, extra_scale);
        if octant_resp.on_hover_text("Click to hop octant").clicked() {
            app.hero_state.start_hop(Duration::from_millis(350));
        }

        ui.add_space(20.0);
        header_title(ui);

        ui.add_space(24.0);
        intake_row(ui, app);

        ui.add_space(14.0);
        sample_slash_chips_row(ui, app);

        // Minimalist footer / drag feedback
        if is_warning_active {
            ui.add_space(14.0);
            render_warning_banner(ui);
        } else if is_drag_hovering {
            ui.add_space(14.0);
            render_drag_hover_cue(ui);
        } else {
            ui.add_space(14.0);
            render_idle_hint(ui);
        }

        if app.is_loading || app.hero_state.loading {
            ui.add_space(18.0);
            let label = if !app.hero_state.source_label.is_empty() {
                format!("loading — {}", app.hero_state.source_label)
            } else {
                "loading...".to_string()
            };
            let font_id = egui::FontId::monospace(11.5);
            let galley =
                ui.painter()
                    .layout_no_wrap(label.clone(), font_id, ui.visuals().weak_text_color());
            let icon_size = 12.0;
            let gap = 6.0;
            let total_w = icon_size + gap + galley.size().x;
            let pad = ((ui.available_width() - total_w) * 0.5).max(0.0);

            ui.horizontal(|ui| {
                if pad > 0.0 {
                    ui.add_space(pad);
                }
                crate::ui::icons::UiIconExt::icon_colored(
                    ui,
                    crate::ui::icons::Icon::Hourglass,
                    icon_size,
                    ui.visuals().weak_text_color(),
                );
                ui.add_space(gap);
                ui.label(
                    egui::RichText::new(label)
                        .monospace()
                        .size(11.5)
                        .color(ui.visuals().weak_text_color()),
                );
            });
        } else if app.hero_state.loaded && !app.hero_state.source_label.is_empty() {
            ui.add_space(18.0);
            let label = format!("loaded — {}", app.hero_state.source_label);
            let font_id = egui::FontId::monospace(11.5);
            let galley =
                ui.painter()
                    .layout_no_wrap(label.clone(), font_id, ui.visuals().text_color());
            let icon_size = 12.0;
            let gap = 6.0;
            let total_w = icon_size + gap + galley.size().x;
            let pad = ((ui.available_width() - total_w) * 0.5).max(0.0);

            ui.horizontal(|ui| {
                if pad > 0.0 {
                    ui.add_space(pad);
                }
                crate::ui::icons::UiIconExt::icon_colored(
                    ui,
                    crate::ui::icons::Icon::Check,
                    icon_size,
                    ui.visuals().selection.bg_fill,
                );
                ui.add_space(gap);
                ui.label(
                    egui::RichText::new(label)
                        .monospace()
                        .size(11.5)
                        .color(ui.visuals().text_color()),
                );
            });
        }
    });
}

fn header_title(ui: &mut egui::Ui) {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        "Bring data into ",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(13.5),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        },
    );
    job.append(
        "Octant",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(13.5),
            color: ui.visuals().strong_text_color(),
            ..Default::default()
        },
    );
    job.append(
        ". Start exploring.",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(13.5),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        },
    );
    job.halign = egui::Align::Center;
    ui.label(job);
}

fn intake_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    egui::Frame::default()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.set_width(430.0);
            ui.horizontal(|ui| {
                let has_input = !app.hero_state.input.trim().is_empty();
                let right_reserve = if has_input { 56.0 } else { 32.0 };

                let edit = egui::TextEdit::singleline(&mut app.hero_state.input)
                    .hint_text("https://... (.zarr / .icechunk), or path...")
                    .font(egui::TextStyle::Monospace)
                    .frame(egui::Frame::NONE)
                    .desired_width(ui.available_width() - right_reserve);
                let response = ui.add(edit);

                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if has_input {
                    let clear_size = egui::vec2(18.0, 18.0);
                    let (clear_rect, clear_resp) =
                        ui.allocate_exact_size(clear_size, egui::Sense::click());

                    if ui.is_rect_visible(clear_rect) {
                        let is_hovered = clear_resp.hovered();
                        let color = if is_hovered {
                            ui.visuals().strong_text_color()
                        } else {
                            ui.visuals().weak_text_color().gamma_multiply(0.65)
                        };
                        crate::ui::icons::Icon::Cross.paint(
                            ui.painter(),
                            clear_rect.shrink(2.0),
                            color,
                            ui.visuals().dark_mode,
                        );
                    }

                    if clear_resp.on_hover_text("Clear input").clicked() {
                        app.hero_state.input.clear();
                    }
                }

                // Procedural download / load icon button
                let btn_size = egui::vec2(26.0, 22.0);
                let (btn_rect, btn_response) =
                    ui.allocate_exact_size(btn_size, egui::Sense::click());

                if ui.is_rect_visible(btn_rect) {
                    let btn_visuals = ui.style().interact(&btn_response);
                    ui.painter().rect(
                        btn_rect,
                        4.0,
                        btn_visuals.bg_fill,
                        btn_visuals.bg_stroke,
                        egui::StrokeKind::Inside,
                    );

                    let icon_rect = btn_rect.shrink(4.0);
                    crate::ui::icons::Icon::DropTray.paint(
                        ui.painter(),
                        icon_rect,
                        btn_visuals.fg_stroke.color,
                        ui.visuals().dark_mode,
                    );
                }

                let go_clicked = btn_response
                    .on_hover_text("Load Dataset & Open Variables")
                    .clicked();

                if enter_pressed || go_clicked {
                    let input_target = if !app.hero_state.input.trim().is_empty() {
                        app.hero_state.input.trim().to_string()
                    } else {
                        app.store_target_input.clone()
                    };

                    app.submit_or_activate_source(&input_target, None);
                }
            });
        });
}

fn sample_slash_chips_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    let samples: [(&str, &str, &str); 2] = [
        (
            "/seasfire",
            "https://s3.bgc-jena.mpg.de:9000/misc/seasfire_rechunked.zarr",
            "Global wildfire & climate rechunked dataset (Zarr)",
        ),
        (
            "/procedural-4d",
            "procedural://volume4d",
            "Synthetic 4D spatiotemporal volume",
        ),
    ];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        let prefix = "try:";
        let prefix_font = egui::FontId::monospace(11.0);
        let prefix_galley = ui.painter().layout_no_wrap(
            prefix.to_string(),
            prefix_font,
            ui.visuals().weak_text_color(),
        );

        let chip_font = egui::FontId::monospace(11.0);
        let chip_padding = egui::vec2(12.0, 6.0);
        let chip_widths: f32 = samples
            .iter()
            .map(|(label, _, _)| {
                let g = ui.painter().layout_no_wrap(
                    label.to_string(),
                    chip_font.clone(),
                    ui.visuals().text_color(),
                );
                g.size().x + chip_padding.x
            })
            .sum();

        let total_w =
            prefix_galley.size().x + chip_widths + ((samples.len() - 1) as f32 * 6.0) + 6.0;
        let pad = ((ui.available_width() - total_w) * 0.5).max(0.0);
        if pad > 0.0 {
            ui.add_space(pad);
        }

        ui.label(
            egui::RichText::new(prefix)
                .monospace()
                .size(11.0)
                .color(ui.visuals().weak_text_color().gamma_multiply(0.6)),
        );

        for (label, uri, desc) in samples {
            let resp = render_ghost_slash_chip(ui, label, desc);
            if resp.clicked() {
                app.hero_state.input = uri.to_string();
                app.submit_or_activate_source(uri, None);
            }
        }
    });
}

fn render_ghost_slash_chip(ui: &mut egui::Ui, label: &str, desc: &str) -> egui::Response {
    let font_id = egui::FontId::monospace(11.0);
    let padding = egui::vec2(10.0, 4.0);

    // Layout text with slash dimmer than the rest
    let mut job = egui::text::LayoutJob::default();
    if let Some(rest) = label.strip_prefix('/') {
        job.append(
            "/",
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                color: ui.visuals().weak_text_color().gamma_multiply(0.5),
                ..Default::default()
            },
        );
        job.append(
            rest,
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                color: ui.visuals().text_color(),
                ..Default::default()
            },
        );
    } else {
        job.append(
            label,
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                color: ui.visuals().text_color(),
                ..Default::default()
            },
        );
    }

    let galley = ui.painter().layout_job(job);
    let desired_size = galley.size() + padding;

    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();
        let bg_color = if is_hovered {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };

        let stroke = if is_hovered {
            egui::Stroke::new(
                0.8,
                ui.visuals()
                    .widgets
                    .hovered
                    .bg_stroke
                    .color
                    .gamma_multiply(0.6),
            )
        } else {
            egui::Stroke::NONE
        };

        if is_hovered {
            ui.painter()
                .rect(rect, 4.0, bg_color, stroke, egui::StrokeKind::Inside);
        }

        let text_pos = rect.center() - galley.size() * 0.5;
        let text_color = if is_hovered {
            ui.visuals().strong_text_color()
        } else {
            ui.visuals().text_color()
        };
        ui.painter().galley(text_pos, galley, text_color);
    }

    response.on_hover_text(format!("Load sample: {desc}"))
}

fn render_idle_hint(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("paste URL, local path, or drag & drop files anywhere")
            .monospace()
            .size(10.0)
            .color(ui.visuals().strong_text_color()),
    );
}

fn render_drag_hover_cue(ui: &mut egui::Ui) {
    let width = 430.0;
    let height = 40.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let is_dark = ui.visuals().dark_mode;
        let accent = if is_dark {
            egui::Color32::from_rgb(0, 190, 255)
        } else {
            egui::Color32::from_rgb(0, 125, 220)
        };
        let bg = if is_dark {
            egui::Color32::from_rgba_unmultiplied(0, 190, 255, 22)
        } else {
            egui::Color32::from_rgba_unmultiplied(0, 125, 220, 16)
        };

        ui.painter().rect(
            rect,
            6.0,
            bg,
            egui::Stroke::new(1.2, accent),
            egui::StrokeKind::Inside,
        );

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 24.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        crate::ui::icons::Icon::DropTray.paint(ui.painter(), icon_rect, accent, is_dark);

        ui.painter().text(
            egui::pos2(rect.left() + 40.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Drop dataset to load (.nc, .h5, .zarr, .icechunk)",
            egui::FontId::monospace(11.0),
            accent,
        );
    }
}

fn render_warning_banner(ui: &mut egui::Ui) {
    let width = 430.0;
    let height = 36.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let is_dark = ui.visuals().dark_mode;
        let warning_color = egui::Color32::from_rgb(255, 130, 60);
        let bg = if is_dark {
            egui::Color32::from_rgba_unmultiplied(255, 110, 50, 26)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 130, 60, 18)
        };

        ui.painter().rect(
            rect,
            6.0,
            bg,
            egui::Stroke::new(1.0, warning_color),
            egui::StrokeKind::Inside,
        );

        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        crate::ui::icons::Icon::Warning.paint(ui.painter(), icon_rect, warning_color, is_dark);

        ui.painter().text(
            egui::pos2(rect.left() + 36.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Unsupported type — supported: .nc, .h5, .zarr, .icechunk",
            egui::FontId::monospace(10.5),
            warning_color,
        );
    }
}

// ---------------------------------------------------------------------
// The Octant Widget (Core Procedural Isometric Renderer)
//
// Draws a big wireframe cube made of all 8 unit sub-cubes, with one
// full solid mini-cube ("the octant") touring all 8 positions.
// All wireframe lines remain fully intact during transitions.
// ---------------------------------------------------------------------

pub fn draw_octant_widget(
    ui: &mut egui::Ui,
    size: f32,
    filled: [f32; 3],
    extra_rot: f32,
    extra_scale: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let is_dark = ui.visuals().dark_mode;
        let wire_color = ui.visuals().weak_text_color().gamma_multiply(0.40);
        let wire_stroke = egui::Stroke::new(0.8, wire_color);

        let cos30 = 0.8660254_f32;
        let sin30 = 0.5_f32;

        let iso = |x: f32, y: f32, z: f32| -> egui::Vec2 {
            egui::vec2((x - z) * cos30, (x + z) * sin30 - y)
        };

        let scale = size / 4.2;
        let project =
            |x: f32, y: f32, z: f32| -> egui::Pos2 { rect.center() + iso(x, y, z) * scale };

        // Draw all 8 wireframe cubes without skipping any lines
        let draw_wire_cube = |ix: f32, iy: f32, iz: f32| {
            let corners: [egui::Pos2; 8] = std::array::from_fn(|i| {
                let dx = (i & 1) as f32;
                let dy = ((i >> 1) & 1) as f32;
                let dz = ((i >> 2) & 1) as f32;
                project(ix + dx, iy + dy, iz + dz)
            });
            let edges = [
                (0, 1),
                (0, 2),
                (0, 4),
                (1, 3),
                (1, 5),
                (2, 3),
                (2, 6),
                (3, 7),
                (4, 5),
                (4, 6),
                (5, 7),
                (6, 7),
            ];
            for (a, b) in edges {
                painter.line_segment([corners[a], corners[b]], wire_stroke);
            }
        };

        for ix in [-1.0_f32, 0.0] {
            for iy in [-1.0_f32, 0.0] {
                for iz in [-1.0_f32, 0.0] {
                    draw_wire_cube(ix, iy, iz);
                }
            }
        }

        // Full solid moving mini-cube (the octant)
        let [fx, fy, fz] = filled;
        let anchor = project(fx + 0.5, fy + 0.5, fz + 0.5);
        let (s, c) = extra_rot.sin_cos();
        let wobble = |pt: egui::Pos2| -> egui::Pos2 {
            let d = pt - anchor;
            let d = egui::vec2(d.x * c - d.y * s, d.x * s + d.y * c) * extra_scale;
            anchor + d
        };
        let p = |dx: f32, dy: f32, dz: f32| wobble(project(fx + dx, fy + dy, fz + dz));

        // The 3 visible isometric faces of a unit cube:
        // Top face (Y = 1 plane)
        let face_top = vec![
            p(0.0, 1.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(1.0, 1.0, 1.0),
            p(0.0, 1.0, 1.0),
        ];
        // Right face (X = 1 plane)
        let face_right = vec![
            p(1.0, 0.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(1.0, 1.0, 1.0),
            p(1.0, 0.0, 1.0),
        ];
        // Front-left face (Z = 1 plane)
        let face_left = vec![
            p(0.0, 0.0, 1.0),
            p(1.0, 0.0, 1.0),
            p(1.0, 1.0, 1.0),
            p(0.0, 1.0, 1.0),
        ];

        let base = ui.visuals().strong_text_color();
        let shade = |c: egui::Color32, f: f32| {
            egui::Color32::from_rgba_unmultiplied(
                ((c.r() as f32) * f).round() as u8,
                ((c.g() as f32) * f).round() as u8,
                ((c.b() as f32) * f).round() as u8,
                c.a(),
            )
        };

        // Solid octant is black in light mode, off-white in dark mode.
        // In light mode, crisp panel-fill white seams separate the black facets cleanly.
        let fill_stroke = if is_dark {
            egui::Stroke::new(1.0, base)
        } else {
            egui::Stroke::new(1.3, ui.visuals().panel_fill)
        };

        painter.add(egui::Shape::convex_polygon(
            face_top,
            shade(base, 1.0),
            fill_stroke,
        ));
        painter.add(egui::Shape::convex_polygon(
            face_right,
            shade(base, 0.72),
            fill_stroke,
        ));
        painter.add(egui::Shape::convex_polygon(
            face_left,
            shade(base, 0.52),
            fill_stroke,
        ));
    }

    response
}
