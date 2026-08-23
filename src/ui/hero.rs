use std::time::{Duration, Instant};

use crate::app::OctantApp;

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

#[inline]
pub fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[inline]
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Fast non-cryptographic PRNG (xorshift64) to generate idle wander pauses
/// without pulling external dependencies.
fn pseudo_random_f32(seed: &mut u64) -> f32 {
    let mut x = *seed;
    if x == 0 {
        x = 0x853c49e6748fea9b;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *seed = x;
    ((x & 0x00ff_ffff) as f32) / (0x0100_0000 as f32)
}

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
        let r = pseudo_random_f32(&mut self.rng_seed);
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
            app.hero_state.begin_submit(&source_path);
            app.store_target_input = source_path;
            app.inspect_active_store();
        }
    }

    // Main centered composition with procedural cube, title, and intake
    let available_h = ui.available_height();

    ui.vertical_centered(|ui| {
        ui.add_space((available_h * 0.14).max(16.0));

        // Centered 3D Octant procedural widget
        draw_octant_widget(ui, 140.0, filled, extra_rot, extra_scale);

        ui.add_space(22.0);
        header_title(ui);

        ui.add_space(28.0);
        intake_row(ui, app);

        ui.add_space(14.0);
        ui.label(
            egui::RichText::new("Drop a file anywhere")
                .monospace()
                .size(11.5)
                .color(ui.visuals().weak_text_color()),
        );

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
        "N-DIMENSIONAL DATA EXPLORER",
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
                let edit = egui::TextEdit::singleline(&mut app.hero_state.input)
                    .hint_text("URL, path, file, or dataset…")
                    .font(egui::TextStyle::Monospace)
                    .frame(egui::Frame::NONE)
                    .desired_width(ui.available_width() - 36.0);
                let response = ui.add(edit);

                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                // Procedural downward chevron button (clean on all platforms without glyph missing issues)
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

                    let center = btn_rect.center();
                    let stroke = btn_visuals.fg_stroke;
                    let w = 4.0;
                    let h = 2.5;
                    ui.painter().line_segment(
                        [
                            egui::pos2(center.x - w, center.y - h),
                            egui::pos2(center.x, center.y + h),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(center.x, center.y + h),
                            egui::pos2(center.x + w, center.y - h),
                        ],
                        stroke,
                    );
                }

                let go_clicked = btn_response
                    .on_hover_text("Inspect Store & Open Variables")
                    .clicked();

                if enter_pressed || go_clicked {
                    let input_target = if !app.hero_state.input.trim().is_empty() {
                        app.hero_state.input.trim().to_string()
                    } else {
                        app.store_target_input.clone()
                    };

                    app.hero_state.begin_submit(&input_target);
                    app.store_target_input = input_target;
                    app.inspect_active_store();
                }
            });
        });
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
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
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

        let fill_stroke = egui::Stroke::new(1.0, base);

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
