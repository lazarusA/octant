//! The bar itself: a layer's colormap as categorical swatches or a
//! continuous gradient, with ticks and labels below it.

use super::checker;
use super::ticks::{format_scientific_tick, generate_colorbar_ticks};
use crate::app::layers::ColorStyle;
use crate::utils::colormap::{apply_color_scale_cpu, orient, registry, unscale_norm_to_value};
use egui::{Color32, Mesh, Pos2, Rect, Shape, Stroke, epaint::Vertex};

/// What a bar draws: a layer's color style and the colormap row it shows.
pub struct BarStyle<'a> {
    pub color: &'a ColorStyle,
    pub colormap: u32,
}

impl BarStyle<'_> {
    /// The color drawn at data position `t` (opacity curve included).
    fn color_at(&self, t: f32) -> Color32 {
        let rgb = registry::sample(self.colormap, orient(t, self.color.reversed));
        checker::with_alpha(rgb, self.color.alpha_at(t))
    }

    /// `value`'s position on the bar's scale.
    fn scaled(&self, value: f32) -> f32 {
        let c = self.color;
        apply_color_scale_cpu(value, c.range_min, c.range_max, c.scale_type, c.scale_param)
    }
}

/// Theme colors of the bar's outline and labels.
#[derive(Clone, Copy)]
pub struct BarColors {
    pub border: Color32,
    pub strong_text: Color32,
    pub text: Color32,
}

/// Categories shown without unique values: ten even bins of the range.
fn fallback_categories(min: f32, max: f32) -> [f32; 10] {
    let span = (max - min).max(1e-30);
    std::array::from_fn(|i| min + (0.05 + 0.1 * i as f32) * span)
}

/// Draws `bar`'s categories (`unique_vals`, else ten even bins) as swatches
/// with a tick and label at each center.
pub fn draw_categorical(
    ui: &egui::Ui,
    bar_rect: Rect,
    bar: &BarStyle<'_>,
    unique_vals: Option<&[f32]>,
    colors: BarColors,
) {
    let fallback = fallback_categories(bar.color.range_min, bar.color.range_max);
    let cats: &[f32] = unique_vals.unwrap_or(&fallback);
    let n = cats.len() as f32;
    let x_at = |t: f32| bar_rect.min.x + t * bar_rect.width();
    let mut mesh = Mesh::default();
    for (i, &val) in cats.iter().enumerate() {
        // Same bin centering as the plot shaders, so swatches match the plot.
        let bin = (bar.scaled(val).clamp(0.0, 0.999_999) * n).floor();
        let color = bar.color_at((bin + 0.5) / n);
        let (x0, x1) = (x_at(i as f32 / n), x_at((i + 1) as f32 / n));
        push_quad(&mut mesh, x0, x1, bar_rect, [color, color]);
    }
    paint_bar(ui, bar_rect, bar, mesh, colors.border);

    let divider = Stroke::new(1.0, Color32::from_black_alpha(120));
    for i in 1..cats.len() {
        let x = x_at(i as f32 / n);
        let ends = [Pos2::new(x, bar_rect.min.y), Pos2::new(x, bar_rect.max.y)];
        ui.painter().line_segment(ends, divider);
    }
    for (i, &val) in cats.iter().enumerate() {
        let t = (i as f32 + 0.5) / n;
        let label = (t > 0.12 && t < 0.88).then(|| format_scientific_tick(val));
        major_tick(ui, x_at(t), bar_rect, label.as_deref(), colors.strong_text);
    }
}

/// Draws `bar`'s colormap as a gradient over its scale, with ticks.
pub fn draw_continuous(ui: &egui::Ui, bar_rect: Rect, bar: &BarStyle<'_>, colors: BarColors) {
    const SEGMENTS: usize = 128;
    let c = bar.color;
    let x_at = |t: f32| bar_rect.min.x + t * bar_rect.width();
    let mut mesh = Mesh::default();
    let mut prev: Option<(f32, Color32)> = None;
    for i in 0..=SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let raw = unscale_norm_to_value(t, c.range_min, c.range_max, c.scale_type, c.scale_param);
        let color = bar.color_at(bar.scaled(raw));
        if let Some((x0, c0)) = prev {
            push_quad(&mut mesh, x0, x_at(t), bar_rect, [c0, color]);
        }
        prev = Some((x_at(t), color));
    }
    paint_bar(ui, bar_rect, bar, mesh, colors.border);

    let ticks = generate_colorbar_ticks(c.range_min, c.range_max, c.scale_type, c.scale_param);
    for tick in ticks {
        let x = x_at(tick.t_pos);
        if tick.is_major {
            let grid = Stroke::new(1.0, Color32::from_black_alpha(80));
            let ends = [Pos2::new(x, bar_rect.min.y), Pos2::new(x, bar_rect.max.y)];
            ui.painter().line_segment(ends, grid);
            let label = tick
                .label
                .as_deref()
                .filter(|_| tick.t_pos > 0.12 && tick.t_pos < 0.88);
            major_tick(ui, x, bar_rect, label, colors.strong_text);
        } else {
            let (y_in, y_out) = (bar_rect.max.y - 3.0, bar_rect.max.y + 3.5);
            haloed_line(ui, x, (y_in, y_out), (1.8, 1.0), colors.text);
        }
    }
}

/// Adds the quad between `x0` and `x1` spanning the bar's height, colored
/// `left` to `right`.
fn push_quad(mesh: &mut Mesh, x0: f32, x1: f32, bar: Rect, [left, right]: [Color32; 2]) {
    let idx = mesh.vertices.len() as u32;
    let corners = [
        (x0, bar.min.y, left),
        (x0, bar.max.y, left),
        (x1, bar.min.y, right),
        (x1, bar.max.y, right),
    ];
    mesh.vertices
        .extend(corners.iter().map(|&(x, y, color)| Vertex {
            pos: Pos2::new(x, y),
            uv: Pos2::ZERO,
            color,
        }));
    mesh.indices
        .extend_from_slice(&[idx, idx + 1, idx + 2, idx + 1, idx + 3, idx + 2]);
}

/// Paints the bar's mesh over a checkerboard when translucent, then its outline.
fn paint_bar(ui: &egui::Ui, bar_rect: Rect, bar: &BarStyle<'_>, mesh: Mesh, border: Color32) {
    if bar.color.is_translucent() {
        checker::paint_checker(ui.painter(), bar_rect, ui.visuals().dark_mode);
    }
    ui.painter().add(Shape::mesh(mesh));
    ui.painter().rect_stroke(
        bar_rect,
        0.0,
        Stroke::new(1.0, border),
        egui::StrokeKind::Middle,
    );
}

/// A major tick across the bar's bottom edge at `x`, with `label` below it.
fn major_tick(ui: &egui::Ui, x: f32, bar_rect: Rect, label: Option<&str>, color: Color32) {
    let (y_in, y_out) = (bar_rect.max.y - 4.5, bar_rect.max.y + 5.5);
    haloed_line(ui, x, (y_in, y_out), (2.2, 1.2), color);
    if let Some(label) = label {
        ui.painter().text(
            Pos2::new(x, y_out + 2.0),
            egui::Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(11.0),
            color,
        );
    }
}

/// A vertical tick at `x` from `y.0` to `y.1`: a dark halo of width `w.0`
/// under a `color` line of width `w.1`.
fn haloed_line(
    ui: &egui::Ui,
    x: f32,
    (y0, y1): (f32, f32),
    (halo, line): (f32, f32),
    color: Color32,
) {
    let ends = [Pos2::new(x, y0), Pos2::new(x, y1)];
    let painter = ui.painter();
    painter.line_segment(ends, Stroke::new(halo, Color32::from_black_alpha(180)));
    painter.line_segment(ends, Stroke::new(line, color));
}
