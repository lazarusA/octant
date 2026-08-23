use std::time::{Duration, Instant};

use crate::app::OctantApp;

// ---------------------------------------------------------------------
// Color Tokens
// ---------------------------------------------------------------------

#[inline]
pub fn accent_color() -> egui::Color32 {
    egui::Color32::from_rgb(255, 139, 71) // warm amber
}

#[inline]
pub fn panel_fill_color() -> egui::Color32 {
    egui::Color32::from_rgb(10, 13, 18)
}

#[inline]
pub fn card_fill_color() -> egui::Color32 {
    egui::Color32::from_rgb(16, 20, 28)
}

#[inline]
pub fn card_hover_color() -> egui::Color32 {
    egui::Color32::from_rgb(20, 25, 34)
}

#[inline]
pub fn text_primary_color() -> egui::Color32 {
    egui::Color32::from_rgb(233, 236, 241)
}

#[inline]
pub fn text_secondary_color() -> egui::Color32 {
    egui::Color32::from_gray(124)
}

#[inline]
pub fn text_dim_color() -> egui::Color32 {
    egui::Color32::from_gray(74)
}

#[inline]
pub fn border_subtle_color() -> egui::Color32 {
    egui::Color32::from_gray(40)
}

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
    pub fn begin_submit(&mut self, source_override: Option<&str>) {
        if self.loading {
            return;
        }

        self.source_label = if let Some(src) = source_override {
            src.to_owned()
        } else if self.input.trim().is_empty() {
            "demo_dataset.nc".to_owned()
        } else {
            self.input.trim().to_owned()
        };

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

    /// Advance animation physics and return (filled_corner, extra_rot, extra_scale, skip_from, skip_to).
    pub fn update_animation(&mut self, now: Instant) -> ([f32; 3], f32, f32, [f32; 3], [f32; 3]) {
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

                (
                    SEQUENCE[self.current],
                    0.0,
                    1.0,
                    self.anim_from,
                    self.anim_to,
                )
            } else {
                let eased = ease_in_out_cubic(t.clamp(0.0, 1.0));
                let f = lerp3(self.anim_from, self.anim_to, eased);
                // gentle scale-down/up + slight rotation while it's in transit
                let wobble = (std::f32::consts::PI * t).sin();
                (
                    f,
                    wobble * 0.16,
                    1.0 - wobble * 0.09,
                    self.anim_from,
                    self.anim_to,
                )
            }
        } else {
            if !self.loading && now >= self.next_hop_at {
                self.start_hop(Duration::from_millis(1700));
            }
            (
                SEQUENCE[self.current],
                0.0,
                1.0,
                self.anim_from,
                self.anim_to,
            )
        }
    }
}

// ---------------------------------------------------------------------
// Hero Landing UI Rendering
// ---------------------------------------------------------------------

/// Render the complete Hero Landing page inside the allocated UI region.
pub fn show_hero_landing(app: &mut OctantApp, ui: &mut egui::Ui) {
    let now = Instant::now();
    let (filled, extra_rot, extra_scale, anim_from, anim_to) = app.hero_state.update_animation(now);

    // Keep animating smoothly at 60 FPS while wandering or loading.
    ui.ctx().request_repaint_after(Duration::from_millis(16));

    // Handle dropped files across the entire window
    let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
    if let Some(file) = dropped_files.first() {
        let path = file.path();
        let source_path = path.to_string_lossy().trim().to_string();

        if !source_path.is_empty() {
            app.hero_state.input = source_path.clone();
            app.hero_state.begin_submit(Some(&source_path));
            app.store_target_input = source_path;
            app.inspect_active_store();
        }
    }

    // Main centered composition (upper-middle positioning)
    let available_h = ui.available_height();
    ui.add_space(available_h * 0.22);

    ui.vertical_centered(|ui| {
        header_title(ui);
        ui.add_space(36.0);
        intake_row(ui, app);
        ui.add_space(18.0);
        ui.label(
            egui::RichText::new("Drop a file anywhere")
                .monospace()
                .size(11.5)
                .color(ui.visuals().weak_text_color()),
        );

        if app.hero_state.loaded {
            ui.add_space(28.0);
            ui.label(
                egui::RichText::new(format!("● loaded — {}", app.hero_state.source_label))
                    .monospace()
                    .size(12.0)
                    .color(accent_color()),
            );
        } else if app.is_loading {
            ui.add_space(28.0);
            ui.label(
                egui::RichText::new(format!(
                    "● inspecting store — {}",
                    app.hero_state.source_label
                ))
                .monospace()
                .size(12.0)
                .color(accent_color()),
            );
        }
    });

    // Anchored animated octant logo in the bottom-right corner
    egui::Area::new(egui::Id::new("octant_hero_logo_widget"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-32.0, -32.0))
        .show(ui.ctx(), |ui| {
            let skip = [anim_from, anim_to];
            draw_octant_widget(
                ui,
                220.0,
                filled,
                &skip,
                accent_color(),
                extra_rot,
                extra_scale,
            );
        });
}

fn header_title(ui: &mut egui::Ui) {
    let is_dark = ui.visuals().dark_mode;
    let mut job = egui::text::LayoutJob::default();
    job.append(
        "OCTANT",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(16.0),
            color: if is_dark {
                egui::Color32::from_rgb(233, 236, 241)
            } else {
                egui::Color32::from_rgb(20, 24, 33)
            },
            ..Default::default()
        },
    );
    job.append(
        ": N-Dimensional Data Explorer",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(16.0),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        },
    );
    job.halign = egui::Align::Center;
    ui.label(job);
}

fn intake_row(ui: &mut egui::Ui, app: &mut OctantApp) {
    let is_dark = ui.visuals().dark_mode;
    let card_bg = if is_dark {
        egui::Color32::from_rgb(16, 20, 28)
    } else {
        egui::Color32::from_rgb(244, 246, 250)
    };
    let border_color = if is_dark {
        egui::Color32::from_gray(40)
    } else {
        egui::Color32::from_gray(210)
    };

    egui::Frame::default()
        .fill(card_bg)
        .stroke(egui::Stroke::new(1.0, border_color))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(14, 6))
        .show(ui, |ui| {
            ui.set_width(420.0);
            ui.horizontal(|ui| {
                let edit = egui::TextEdit::singleline(&mut app.hero_state.input)
                    .hint_text("URL, path, file, or dataset…")
                    .font(egui::TextStyle::Monospace)
                    .frame(egui::Frame::NONE)
                    .desired_width(ui.available_width() - 40.0);
                let response = ui.add(edit);

                let enter_pressed =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                let go_clicked = ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("↓")
                                .strong()
                                .color(egui::Color32::BLACK),
                        )
                        .fill(accent_color()),
                    )
                    .on_hover_text("Inspect Store & Open Variables")
                    .clicked();

                if enter_pressed || go_clicked {
                    let input_target = app.hero_state.input.trim().to_string();
                    app.hero_state.begin_submit(None);
                    if !input_target.is_empty() {
                        app.store_target_input = input_target;
                    }
                    app.inspect_active_store();
                }
            });
        });
}

// ---------------------------------------------------------------------
// The Octant Widget (Core Procedural Isometric Renderer)
//
// Draws a big wireframe cube made of its 8 unit sub-cubes, with one
// sub-cube ("the octant") filled solid. `filled` is its current corner
// — usually one of the 8 grid corners, but can be a fractional point
// in between while it's sliding from one to the next. `skip` hides the
// wireframe outline at the corners currently in transit (its origin
// and destination) so the moving solid piece doesn't overlap a wire
// outline sitting in the same spot.
// ---------------------------------------------------------------------

pub fn draw_octant_widget(
    ui: &mut egui::Ui,
    size: f32,
    filled: [f32; 3],
    skip: &[[f32; 3]],
    accent: egui::Color32,
    extra_rot: f32,
    extra_scale: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let is_dark = ui.visuals().dark_mode;
        let wire_color = if is_dark {
            egui::Color32::from_gray(90).gamma_multiply(0.55)
        } else {
            egui::Color32::from_gray(160).gamma_multiply(0.70)
        };
        let wire_stroke = egui::Stroke::new(0.9, wire_color);

        let cos30 = 0.8660254_f32;
        let sin30 = 0.5_f32;

        let iso = |x: f32, y: f32, z: f32| -> egui::Vec2 {
            egui::vec2((x - z) * cos30, (x + z) * sin30 - y)
        };

        let scale = size / 4.4;
        let project =
            |x: f32, y: f32, z: f32| -> egui::Pos2 { rect.center() + iso(x, y, z) * scale };

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
                    let hidden = skip.iter().any(|s| {
                        (s[0] - ix).abs() < 0.01
                            && (s[1] - iy).abs() < 0.01
                            && (s[2] - iz).abs() < 0.01
                    });
                    if !hidden {
                        draw_wire_cube(ix, iy, iz);
                    }
                }
            }
        }

        // the moving, filled octant — wobble (rotate/scale) around its own center only
        let [fx, fy, fz] = filled;
        let anchor = project(fx + 0.5, fy + 0.5, fz + 0.5);
        let (s, c) = extra_rot.sin_cos();
        let wobble = |pt: egui::Pos2| -> egui::Pos2 {
            let d = pt - anchor;
            let d = egui::vec2(d.x * c - d.y * s, d.x * s + d.y * c) * extra_scale;
            anchor + d
        };
        let p = |dx: f32, dy: f32, dz: f32| wobble(project(fx + dx, fy + dy, fz + dz));

        let outer = |v: f32| if v < -0.5 { 0.0 } else { 1.0 };
        let (dxo, dyo, dzo) = (outer(fx), outer(fy), outer(fz));

        let face_y = vec![
            p(0.0, dyo, 0.0),
            p(1.0, dyo, 0.0),
            p(1.0, dyo, 1.0),
            p(0.0, dyo, 1.0),
        ];
        let face_x = vec![
            p(dxo, 0.0, 0.0),
            p(dxo, 0.0, 1.0),
            p(dxo, 1.0, 1.0),
            p(dxo, 1.0, 0.0),
        ];
        let face_z = vec![
            p(0.0, 0.0, dzo),
            p(1.0, 0.0, dzo),
            p(1.0, 1.0, dzo),
            p(0.0, 1.0, dzo),
        ];

        let shade = |c: egui::Color32, f: f32| {
            egui::Color32::from_rgb(
                (c.r() as f32 * f) as u8,
                (c.g() as f32 * f) as u8,
                (c.b() as f32 * f) as u8,
            )
        };
        let fill_stroke = egui::Stroke::new(0.9, accent);

        painter.add(egui::Shape::convex_polygon(
            face_y,
            shade(accent, 1.0),
            fill_stroke,
        ));
        painter.add(egui::Shape::convex_polygon(
            face_x,
            shade(accent, 0.72),
            fill_stroke,
        ));
        painter.add(egui::Shape::convex_polygon(
            face_z,
            shade(accent, 0.5),
            fill_stroke,
        ));
    }

    response
}
