use egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2, ecolor::Hsva};
use std::sync::Arc;

/// Custom rendering closure type for `ColorShape::Custom`.
pub type CustomShapeFn = Arc<dyn Fn(&egui::Painter, Rect, Color32, Stroke) + Send + Sync>;

/// Shape definitions for custom color picker trigger buttons.
#[derive(Clone)]
pub enum ColorShape {
    /// Left-pointing triangle (◁) filling the rect
    LeftTriangle,
    /// Right-pointing triangle (▷) filling the rect
    RightTriangle,
    /// Up-pointing triangle (△) filling the rect
    UpTriangle,
    /// Down-pointing triangle (▽) filling the rect
    DownTriangle,
    /// Circle centered inside the rect
    Circle,
    /// Rounded rectangle with specified corner radius
    Rect(f32),
    /// Custom polygon points normalized to `[0.0, 1.0]` relative to the rect bounding box
    NormalizedPolygon(Vec<Pos2>),
    /// Custom drawing callback allowing arbitrary rendering logic
    Custom(CustomShapeFn),
}

impl ColorShape {
    /// Convenient constructor for arbitrary custom shape rendering logic
    pub fn custom(
        f: impl Fn(&egui::Painter, Rect, Color32, Stroke) + Send + Sync + 'static,
    ) -> Self {
        Self::Custom(Arc::new(f))
    }

    /// Helper to paint the shape with given color and stroke inside `rect`.
    pub fn paint(&self, painter: &egui::Painter, rect: Rect, color: Color32, stroke: Stroke) {
        match self {
            Self::LeftTriangle => {
                let tip = Pos2::new(rect.min.x, rect.center().y);
                let top = Pos2::new(rect.max.x, rect.min.y);
                let bottom = Pos2::new(rect.max.x, rect.max.y);
                painter.add(Shape::convex_polygon(vec![tip, top, bottom], color, stroke));
            }
            Self::RightTriangle => {
                let top = Pos2::new(rect.min.x, rect.min.y);
                let tip = Pos2::new(rect.max.x, rect.center().y);
                let bottom = Pos2::new(rect.min.x, rect.max.y);
                painter.add(Shape::convex_polygon(vec![top, tip, bottom], color, stroke));
            }
            Self::UpTriangle => {
                let tip = Pos2::new(rect.center().x, rect.min.y);
                let bottom_right = Pos2::new(rect.max.x, rect.max.y);
                let bottom_left = Pos2::new(rect.min.x, rect.max.y);
                painter.add(Shape::convex_polygon(
                    vec![tip, bottom_right, bottom_left],
                    color,
                    stroke,
                ));
            }
            Self::DownTriangle => {
                let top_left = Pos2::new(rect.min.x, rect.min.y);
                let top_right = Pos2::new(rect.max.x, rect.min.y);
                let tip = Pos2::new(rect.center().x, rect.max.y);
                painter.add(Shape::convex_polygon(
                    vec![top_left, top_right, tip],
                    color,
                    stroke,
                ));
            }
            Self::Circle => {
                let radius = rect.width().min(rect.height()) / 2.0;
                painter.circle(rect.center(), radius, color, stroke);
            }
            Self::Rect(rounding) => {
                painter.rect(rect, *rounding, color, stroke, egui::StrokeKind::Middle);
            }
            Self::NormalizedPolygon(normalized_pts) => {
                let mapped_pts: Vec<Pos2> = normalized_pts
                    .iter()
                    .map(|p| {
                        Pos2::new(
                            rect.min.x + p.x * rect.width(),
                            rect.min.y + p.y * rect.height(),
                        )
                    })
                    .collect();
                if mapped_pts.len() >= 3 {
                    painter.add(Shape::convex_polygon(mapped_pts, color, stroke));
                }
            }
            Self::Custom(cb) => {
                (cb)(painter, rect, color, stroke);
            }
        }
    }
}

/// A flexible color picker button widget that supports arbitrary custom vector shapes
/// and triggers a sleek, focused color picker popup.
pub struct ShapeColorPicker<'a> {
    id_salt: egui::Id,
    color: &'a mut [f32; 4],
    shape: ColorShape,
    size: Option<Vec2>,
    title: String,
    tooltip: Option<String>,
    anchor_offset: Vec2,
}

impl<'a> ShapeColorPicker<'a> {
    /// Creates a new `ShapeColorPicker` for an RGBA float array `[f32; 4]`.
    pub fn new(
        id_salt: impl std::hash::Hash + std::fmt::Debug,
        color: &'a mut [f32; 4],
        shape: ColorShape,
    ) -> Self {
        Self {
            id_salt: egui::Id::new(id_salt),
            color,
            shape,
            size: None,
            title: "Color Picker".to_string(),
            tooltip: None,
            anchor_offset: Vec2::new(-70.0, -250.0),
        }
    }

    /// Sets the header title displayed inside the popup.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets an optional hover tooltip on the shape button.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Sets an explicit widget size when rendered with `show()`.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the popup anchor offset relative to `rect.min`.
    pub fn anchor_offset(mut self, offset: Vec2) -> Self {
        self.anchor_offset = offset;
        self
    }

    /// Displays the color picker widget at a specified pre-calculated `Rect`.
    pub fn show_at(self, ui: &mut egui::Ui, rect: Rect) -> egui::Response {
        let resp = ui.interact(rect, self.id_salt.with("btn"), egui::Sense::click());
        let resp = if let Some(tip) = &self.tooltip {
            resp.on_hover_text(tip)
        } else {
            resp
        };

        let style = ui.style();
        let stroke = if resp.hovered() {
            egui::Stroke::new(1.5_f32, style.visuals.widgets.active.fg_stroke.color)
        } else {
            egui::Stroke::new(
                1.0_f32,
                style.visuals.widgets.noninteractive.fg_stroke.color,
            )
        };

        let color_c32 = Color32::from_rgba_unmultiplied(
            (self.color[0] * 255.0).round() as u8,
            (self.color[1] * 255.0).round() as u8,
            (self.color[2] * 255.0).round() as u8,
            (self.color[3] * 255.0).round() as u8,
        );

        // Paint the shape
        self.shape.paint(ui.painter(), rect, color_c32, stroke);

        // Popup open/close state
        let popup_open_id = self.id_salt.with("popup_open");
        let popup_area_id = self.id_salt.with("popup_area");

        if resp.clicked() {
            let is_open = ui
                .data(|d| d.get_temp::<bool>(popup_open_id))
                .unwrap_or(false);
            ui.data_mut(|d| d.insert_temp(popup_open_id, !is_open));
        }

        let popup_pos = rect.min + self.anchor_offset;
        show_clean_color_picker_popup(
            ui,
            self.id_salt,
            self.color,
            popup_open_id,
            popup_area_id,
            popup_pos,
        );

        resp
    }

    /// Allocates layout space and displays the color picker widget.
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let size = self.size.unwrap_or(Vec2::splat(16.0));
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
        let resp = if let Some(tip) = &self.tooltip {
            resp.on_hover_text(tip)
        } else {
            resp
        };

        let style = ui.style();
        let stroke = if resp.hovered() {
            egui::Stroke::new(1.5_f32, style.visuals.widgets.active.fg_stroke.color)
        } else {
            egui::Stroke::new(
                1.0_f32,
                style.visuals.widgets.noninteractive.fg_stroke.color,
            )
        };

        let color_c32 = Color32::from_rgba_unmultiplied(
            (self.color[0] * 255.0).round() as u8,
            (self.color[1] * 255.0).round() as u8,
            (self.color[2] * 255.0).round() as u8,
            (self.color[3] * 255.0).round() as u8,
        );

        self.shape.paint(ui.painter(), rect, color_c32, stroke);

        let popup_open_id = self.id_salt.with("popup_open");
        let popup_area_id = self.id_salt.with("popup_area");

        if resp.clicked() {
            let is_open = ui
                .data(|d| d.get_temp::<bool>(popup_open_id))
                .unwrap_or(false);
            ui.data_mut(|d| d.insert_temp(popup_open_id, !is_open));
        }

        let popup_pos = rect.min + self.anchor_offset;
        show_clean_color_picker_popup(
            ui,
            self.id_salt,
            self.color,
            popup_open_id,
            popup_area_id,
            popup_pos,
        );

        resp
    }
}

/// Color space selection for RGB channel displays (Byte 0-255 vs Float 0-1)
#[derive(Clone, Copy, Debug, PartialEq)]
enum ColorGammaSpace {
    Byte,  // 0 - 255
    Float, // 0.0 - 1.0
}

/// Renders the streamlined color picker popup:
/// - Top: Gamma space options (0-255 / 0-1), RGB channel inputs, and Copy button
/// - Middle: Full-width 2D Saturation/Value color area
/// - Bottom: Full-width Hue spectrum bar and Transparency (Alpha) bar
fn show_clean_color_picker_popup(
    ui: &mut egui::Ui,
    id_salt: egui::Id,
    color_rgba: &mut [f32; 4],
    popup_open_id: egui::Id,
    popup_area_id: egui::Id,
    anchor_pos: Pos2,
) {
    let mut is_open = ui
        .data(|d| d.get_temp::<bool>(popup_open_id))
        .unwrap_or(false);
    if !is_open {
        return;
    }

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

                // =========================================================================
                // TOP SECTION: RGB gamma space mode (0-255 vs 0-1), Copy button & RGB edits
                // =========================================================================
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.selectable_value(&mut gamma_space, ColorGammaSpace::Byte, "0-255");
                    ui.selectable_value(&mut gamma_space, ColorGammaSpace::Float, "0-1");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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

                        if ui
                            .small_button("📋 Copy")
                            .on_hover_text("Copy color values to clipboard")
                            .clicked()
                        {
                            ui.ctx().copy_text(copy_text);
                        }
                    });
                });

                ui.add_space(3.0);

                // RGB Numeric Inputs
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    match gamma_space {
                        ColorGammaSpace::Byte => {
                            let mut r = (color_rgba[0] * 255.0).round() as u8;
                            let mut g = (color_rgba[1] * 255.0).round() as u8;
                            let mut b = (color_rgba[2] * 255.0).round() as u8;
                            let mut a = (color_rgba[3] * 255.0).round() as u8;
                            let mut rgb_changed = false;

                            ui.label(
                                egui::RichText::new("R")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(255, 100, 100)),
                            );
                            rgb_changed |= ui
                                .add(egui::DragValue::new(&mut r).range(0..=255).speed(1.0))
                                .changed();
                            ui.label(
                                egui::RichText::new("G")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(100, 255, 100)),
                            );
                            rgb_changed |= ui
                                .add(egui::DragValue::new(&mut g).range(0..=255).speed(1.0))
                                .changed();
                            ui.label(
                                egui::RichText::new("B")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(100, 150, 255)),
                            );
                            rgb_changed |= ui
                                .add(egui::DragValue::new(&mut b).range(0..=255).speed(1.0))
                                .changed();
                            ui.label(
                                egui::RichText::new("A")
                                    .small()
                                    .strong()
                                    .color(Color32::from_gray(180)),
                            );
                            rgb_changed |= ui
                                .add(egui::DragValue::new(&mut a).range(0..=255).speed(1.0))
                                .changed();

                            if rgb_changed {
                                *color_rgba = [
                                    r as f32 / 255.0,
                                    g as f32 / 255.0,
                                    b as f32 / 255.0,
                                    a as f32 / 255.0,
                                ];
                                hsva = Hsva::from_rgba_unmultiplied(
                                    color_rgba[0],
                                    color_rgba[1],
                                    color_rgba[2],
                                    color_rgba[3],
                                );
                                changed = true;
                            }
                        }
                        ColorGammaSpace::Float => {
                            let mut r = color_rgba[0];
                            let mut g = color_rgba[1];
                            let mut b = color_rgba[2];
                            let mut a = color_rgba[3];
                            let mut rgb_changed = false;

                            ui.label(
                                egui::RichText::new("R")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(255, 100, 100)),
                            );
                            rgb_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut r)
                                        .range(0.0..=1.0)
                                        .speed(0.01)
                                        .max_decimals(2),
                                )
                                .changed();
                            ui.label(
                                egui::RichText::new("G")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(100, 255, 100)),
                            );
                            rgb_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut g)
                                        .range(0.0..=1.0)
                                        .speed(0.01)
                                        .max_decimals(2),
                                )
                                .changed();
                            ui.label(
                                egui::RichText::new("B")
                                    .small()
                                    .strong()
                                    .color(Color32::from_rgb(100, 150, 255)),
                            );
                            rgb_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut b)
                                        .range(0.0..=1.0)
                                        .speed(0.01)
                                        .max_decimals(2),
                                )
                                .changed();
                            ui.label(
                                egui::RichText::new("A")
                                    .small()
                                    .strong()
                                    .color(Color32::from_gray(180)),
                            );
                            rgb_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut a)
                                        .range(0.0..=1.0)
                                        .speed(0.01)
                                        .max_decimals(2),
                                )
                                .changed();

                            if rgb_changed {
                                *color_rgba = [r, g, b, a];
                                hsva = Hsva::from_rgba_unmultiplied(
                                    color_rgba[0],
                                    color_rgba[1],
                                    color_rgba[2],
                                    color_rgba[3],
                                );
                                changed = true;
                            }
                        }
                    }
                });

                ui.add_space(5.0);

                // =========================================================================
                // MIDDLE SECTION: 2D Saturation / Value Color Area (Expands full container)
                // =========================================================================
                let available_w = ui.available_width();
                let sv_size = Vec2::new(available_w, 165.0);
                let (sv_rect, sv_resp) = ui.allocate_exact_size(sv_size, egui::Sense::drag());

                if (sv_resp.dragged() || sv_resp.clicked())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                {
                    hsva.s = ((pos.x - sv_rect.min.x) / sv_rect.width()).clamp(0.0_f32, 1.0_f32);
                    hsva.v = (1.0_f32 - (pos.y - sv_rect.min.y) / sv_rect.height())
                        .clamp(0.0_f32, 1.0_f32);
                    changed = true;
                }

                // Paint 2D Saturation-Value mesh
                let mut sv_mesh = egui::Mesh::default();
                let c_tl = Color32::WHITE;
                let c_tr = Color32::from(Hsva::new(hsva.h, 1.0, 1.0, 1.0));
                let c_bl = Color32::BLACK;
                let c_br = Color32::BLACK;

                sv_mesh.colored_vertex(sv_rect.left_top(), c_tl);
                sv_mesh.colored_vertex(sv_rect.right_top(), c_tr);
                sv_mesh.colored_vertex(sv_rect.right_bottom(), c_br);
                sv_mesh.colored_vertex(sv_rect.left_bottom(), c_bl);
                sv_mesh.add_triangle(0, 1, 2);
                sv_mesh.add_triangle(0, 2, 3);
                ui.painter().add(Shape::mesh(sv_mesh));

                ui.painter().rect_stroke(
                    sv_rect,
                    2.0,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    egui::StrokeKind::Inside,
                );

                // Cursor in 2D field
                let cursor_pos = Pos2::new(
                    sv_rect.min.x + hsva.s * sv_rect.width(),
                    sv_rect.min.y + (1.0 - hsva.v) * sv_rect.height(),
                );
                ui.painter().circle(
                    cursor_pos,
                    5.0,
                    Color32::TRANSPARENT,
                    Stroke::new(2.0, Color32::WHITE),
                );
                ui.painter().circle(
                    cursor_pos,
                    6.0,
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, Color32::BLACK),
                );

                ui.add_space(5.0);

                // =========================================================================
                // BOTTOM SECTION: Hue Spectrum Bar followed by Transparency (Alpha) Bar
                // =========================================================================
                // 1. Hue Bar
                let hue_size = Vec2::new(available_w, 15.0);
                let (hue_rect, hue_resp) = ui.allocate_exact_size(hue_size, egui::Sense::drag());

                if (hue_resp.dragged() || hue_resp.clicked())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                {
                    hsva.h = ((pos.x - hue_rect.min.x) / hue_rect.width()).clamp(0.0_f32, 1.0_f32);
                    changed = true;
                }

                // Paint 6-segment rainbow spectrum
                let num_segments = 6;
                let mut hue_mesh = egui::Mesh::default();
                for i in 0..num_segments {
                    let h0 = i as f32 / num_segments as f32;
                    let h1 = (i + 1) as f32 / num_segments as f32;
                    let x0 = hue_rect.min.x + h0 * hue_rect.width();
                    let x1 = hue_rect.min.x + h1 * hue_rect.width();
                    let r = Rect::from_min_max(
                        Pos2::new(x0, hue_rect.min.y),
                        Pos2::new(x1, hue_rect.max.y),
                    );

                    let c0 = Color32::from(Hsva::new(h0, 1.0, 1.0, 1.0));
                    let c1 = Color32::from(Hsva::new(h1, 1.0, 1.0, 1.0));

                    let idx = hue_mesh.vertices.len() as u32;
                    hue_mesh.colored_vertex(r.left_top(), c0);
                    hue_mesh.colored_vertex(r.right_top(), c1);
                    hue_mesh.colored_vertex(r.right_bottom(), c1);
                    hue_mesh.colored_vertex(r.left_bottom(), c0);
                    hue_mesh.add_triangle(idx, idx + 1, idx + 2);
                    hue_mesh.add_triangle(idx, idx + 2, idx + 3);
                }
                ui.painter().add(Shape::mesh(hue_mesh));

                ui.painter().rect_stroke(
                    hue_rect,
                    2.0,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    egui::StrokeKind::Inside,
                );

                // Cursor on Hue Bar
                let hue_cursor_x = hue_rect.min.x + hsva.h * hue_rect.width();
                let hue_cursor_rect = Rect::from_center_size(
                    Pos2::new(hue_cursor_x, hue_rect.center().y),
                    Vec2::new(5.0, hue_rect.height() + 4.0),
                );
                ui.painter().rect(
                    hue_cursor_rect,
                    2.0,
                    Color32::WHITE,
                    Stroke::new(1.0, Color32::BLACK),
                    egui::StrokeKind::Middle,
                );

                ui.add_space(4.0);

                // 2. Transparency (Alpha) Bar
                let alpha_size = Vec2::new(available_w, 15.0);
                let (alpha_rect, alpha_resp) =
                    ui.allocate_exact_size(alpha_size, egui::Sense::drag());

                if (alpha_resp.dragged() || alpha_resp.clicked())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                {
                    hsva.a =
                        ((pos.x - alpha_rect.min.x) / alpha_rect.width()).clamp(0.0_f32, 1.0_f32);
                    changed = true;
                }

                // Checkered transparent background for alpha bar
                let grid_size = 5.0;
                let mut check_x = alpha_rect.min.x;
                while check_x < alpha_rect.max.x {
                    let mut check_y = alpha_rect.min.y;
                    let col = ((check_x - alpha_rect.min.x) / grid_size) as usize;
                    while check_y < alpha_rect.max.y {
                        let row = ((check_y - alpha_rect.min.y) / grid_size) as usize;
                        let c = if (row + col).is_multiple_of(2) {
                            Color32::from_gray(180)
                        } else {
                            Color32::from_gray(240)
                        };
                        let r = Rect::from_min_max(
                            Pos2::new(check_x, check_y),
                            Pos2::new(
                                (check_x + grid_size).min(alpha_rect.max.x),
                                (check_y + grid_size).min(alpha_rect.max.y),
                            ),
                        );
                        ui.painter().rect_filled(r, 0.0, c);
                        check_y += grid_size;
                    }
                    check_x += grid_size;
                }

                // Alpha gradient over checkered background
                let mut alpha_mesh = egui::Mesh::default();
                let c_trans = Color32::from(Hsva::new(hsva.h, hsva.s, hsva.v, 0.0));
                let c_opaque = Color32::from(Hsva::new(hsva.h, hsva.s, hsva.v, 1.0));
                alpha_mesh.colored_vertex(alpha_rect.left_top(), c_trans);
                alpha_mesh.colored_vertex(alpha_rect.right_top(), c_opaque);
                alpha_mesh.colored_vertex(alpha_rect.right_bottom(), c_opaque);
                alpha_mesh.colored_vertex(alpha_rect.left_bottom(), c_trans);
                alpha_mesh.add_triangle(0, 1, 2);
                alpha_mesh.add_triangle(0, 2, 3);
                ui.painter().add(Shape::mesh(alpha_mesh));

                ui.painter().rect_stroke(
                    alpha_rect,
                    2.0,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                    egui::StrokeKind::Inside,
                );

                // Cursor on Alpha Bar
                let alpha_cursor_x = alpha_rect.min.x + hsva.a * alpha_rect.width();
                let alpha_cursor_rect = Rect::from_center_size(
                    Pos2::new(alpha_cursor_x, alpha_rect.center().y),
                    Vec2::new(5.0, alpha_rect.height() + 4.0),
                );
                ui.painter().rect(
                    alpha_cursor_rect,
                    2.0,
                    Color32::WHITE,
                    Stroke::new(1.0, Color32::BLACK),
                    egui::StrokeKind::Middle,
                );

                // If HSVA changed via 2D area / Hue / Alpha bars, update the output RGBA array
                if changed {
                    let rgba_unmult = egui::ecolor::Rgba::from(hsva).to_rgba_unmultiplied();
                    *color_rgba = rgba_unmult;
                }
            });

            // Close popup on click outside
            if ui.input(|i| i.pointer.any_pressed())
                && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                && !frame_resp.response.rect.contains(pos)
            {
                is_open = false;
            }
        });

    ui.data_mut(|d| {
        d.insert_temp(popup_open_id, is_open);
        d.insert_temp(hsva_id, hsva);
        d.insert_temp(gamma_space_id, gamma_space);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_color_picker_hsva_conversion() {
        let rgba = [1.0, 0.0, 0.0, 1.0];
        let hsva = Hsva::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
        assert!((hsva.h - 0.0).abs() < 1e-3 || (hsva.h - 1.0).abs() < 1e-3);
        assert!((hsva.s - 1.0).abs() < 1e-3);
        assert!((hsva.v - 1.0).abs() < 1e-3);
        assert!((hsva.a - 1.0).abs() < 1e-3);

        let roundtrip = egui::ecolor::Rgba::from(hsva).to_rgba_unmultiplied();
        assert!((roundtrip[0] - 1.0).abs() < 1e-3);
        assert!((roundtrip[1] - 0.0).abs() < 1e-3);
        assert!((roundtrip[2] - 0.0).abs() < 1e-3);
        assert!((roundtrip[3] - 1.0).abs() < 1e-3);
    }
}
