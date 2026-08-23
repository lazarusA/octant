use std::time::{Duration, Instant};

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

    // Handle dropped files across the entire window
    let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
    if let Some(file) = dropped_files.first() {
        let path = file.path();
        let source_path = path.to_string_lossy().trim().to_string();

        if !source_path.is_empty() {
            app.hero_state.input = source_path.clone();
            app.submit_or_activate_source(&source_path, None);
        }
    }

    // Main centered composition with procedural cube, title, and intake
    let available_h = ui.available_height();

    ui.vertical_centered(|ui| {
        ui.add_space((available_h * 0.14).max(16.0));

        // Centered 3D Octant procedural widget (interactive click to hop)
        let octant_resp = draw_octant_widget(ui, 140.0, filled, extra_rot, extra_scale);
        if octant_resp.on_hover_text("Click to hop octant").clicked() {
            app.hero_state.start_hop(Duration::from_millis(350));
        }

        ui.add_space(22.0);
        header_title(ui);

        ui.add_space(28.0);
        intake_row(ui, app);

        ui.add_space(14.0);
        sample_pills_row(ui, app);

        if app.is_loading || app.hero_state.loading {
            ui.add_space(20.0);
            let label = if !app.hero_state.source_label.is_empty() {
                format!("● loading — {}", app.hero_state.source_label)
            } else {
                "● loading...".to_string()
            };
            ui.label(
                egui::RichText::new(label)
                    .monospace()
                    .size(12.0)
                    .color(ui.visuals().weak_text_color()),
            );
        } else if app.hero_state.loaded && !app.hero_state.source_label.is_empty() {
            ui.add_space(20.0);
            ui.label(
                egui::RichText::new(format!("● loaded — {}", app.hero_state.source_label))
                    .monospace()
                    .size(12.0)
                    .color(ui.visuals().text_color()),
            );
        }
    });
}

fn header_title(ui: &mut egui::Ui) {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        "Bring data into ",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(14.0),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        },
    );
    job.append(
        "Octant",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(14.0),
            color: ui.visuals().strong_text_color(),
            ..Default::default()
        },
    );
    job.append(
        ". Start exploring.",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(14.0),
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
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(14, 6))
        .show(ui, |ui| {
            ui.set_width(420.0);
            ui.horizontal(|ui| {
                let has_input = !app.hero_state.input.trim().is_empty();
                let right_reserve = if has_input { 58.0 } else { 36.0 };

                let edit = egui::TextEdit::singleline(&mut app.hero_state.input)
                    .hint_text("https://… (.zarr / .icechunk), or local path…")
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
                        let stroke = egui::Stroke::new(1.2, color);
                        let c = clear_rect.center();
                        let r = 3.5_f32;
                        ui.painter().line_segment(
                            [egui::pos2(c.x - r, c.y - r), egui::pos2(c.x + r, c.y + r)],
                            stroke,
                        );
                        ui.painter().line_segment(
                            [egui::pos2(c.x - r, c.y + r), egui::pos2(c.x + r, c.y - r)],
                            stroke,
                        );
                    }

                    if clear_resp.on_hover_text("Clear input").clicked() {
                        app.hero_state.input.clear();
                    }
                }

                // Procedural download / load icon button
                let btn_size = egui::vec2(28.0, 24.0);
                let (btn_rect, btn_response) =
                    ui.allocate_exact_size(btn_size, egui::Sense::click());

                if ui.is_rect_visible(btn_rect) {
                    let btn_visuals = ui.style().interact(&btn_response);
                    ui.painter().rect(
                        btn_rect,
                        6.0,
                        btn_visuals.bg_fill,
                        btn_visuals.bg_stroke,
                        egui::StrokeKind::Inside,
                    );

                    let c = btn_rect.center();
                    let stroke = egui::Stroke::new(1.3, btn_visuals.fg_stroke.color);

                    // Downward arrow stem
                    ui.painter().line_segment(
                        [egui::pos2(c.x, c.y - 4.5), egui::pos2(c.x, c.y + 1.5)],
                        stroke,
                    );
                    // Arrowhead wings
                    ui.painter().line_segment(
                        [egui::pos2(c.x - 3.2, c.y - 1.2), egui::pos2(c.x, c.y + 2.0)],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [egui::pos2(c.x + 3.2, c.y - 1.2), egui::pos2(c.x, c.y + 2.0)],
                        stroke,
                    );
                    // Load / tray bracket bottom
                    ui.painter().line_segment(
                        [
                            egui::pos2(c.x - 4.8, c.y + 3.5),
                            egui::pos2(c.x - 4.8, c.y + 5.2),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(c.x - 4.8, c.y + 5.2),
                            egui::pos2(c.x + 4.8, c.y + 5.2),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(c.x + 4.8, c.y + 5.2),
                            egui::pos2(c.x + 4.8, c.y + 3.5),
                        ],
                        stroke,
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

fn sample_pills_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    let is_dark = ui.visuals().dark_mode;

    let samples: [(&str, &str, egui::Color32, egui::Color32); 2] = [
        (
            "🔥 SeasFire",
            "https://s3.bgc-jena.mpg.de:9000/misc/seasfire_rechunked.zarr",
            egui::Color32::from_rgb(255, 140, 70), // amber
            egui::Color32::from_rgba_unmultiplied(255, 140, 70, 24),
        ),
        (
            "🎲 Procedural 4D",
            "procedural://volume4d",
            if is_dark {
                egui::Color32::from_rgb(222, 228, 238)
            } else {
                egui::Color32::from_rgb(70, 76, 88)
            }, // white-ish / neutral
            if is_dark {
                egui::Color32::from_rgba_unmultiplied(222, 228, 238, 22)
            } else {
                egui::Color32::from_rgba_unmultiplied(70, 76, 88, 16)
            },
        ),
    ];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        let total_w: f32 = samples
            .iter()
            .map(|(label, _, _, _)| {
                let galley = ui.painter().layout_no_wrap(
                    label.to_string(),
                    egui::FontId::monospace(11.5),
                    egui::Color32::WHITE,
                );
                galley.size().x + 20.0
            })
            .sum::<f32>()
            + ((samples.len() - 1) as f32 * 8.0);

        let pad = ((ui.available_width() - total_w) * 0.5).max(0.0);
        if pad > 0.0 {
            ui.add_space(pad);
        }

        for (label, uri, text_color, bg_color) in samples {
            let resp = render_pill_button(ui, label, text_color, bg_color);
            if resp.clicked() {
                app.hero_state.input = uri.to_string();
                app.submit_or_activate_source(uri, None);
            }
        }
    });
}

fn render_pill_button(
    ui: &mut egui::Ui,
    label: &str,
    text_color: egui::Color32,
    bg_color: egui::Color32,
) -> egui::Response {
    let font_id = egui::FontId::monospace(11.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font_id, text_color);
    let padding = egui::vec2(16.0, 7.0);
    let desired_size = galley.size() + padding;

    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();
        let current_bg = if is_hovered {
            bg_color.gamma_multiply(1.8)
        } else {
            bg_color
        };
        let border_stroke = if is_hovered {
            egui::Stroke::new(1.0, text_color.gamma_multiply(0.85))
        } else {
            egui::Stroke::new(0.8, text_color.gamma_multiply(0.35))
        };

        ui.painter().rect(
            rect,
            12.0,
            current_bg,
            border_stroke,
            egui::StrokeKind::Inside,
        );

        let text_pos = rect.center() - galley.size() * 0.5;
        ui.painter().galley(text_pos, galley, text_color);
    }

    response.on_hover_text(format!("Load sample: {}", label))
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
