//! Gradient swatches drawn from one egui texture holding every registry LUT
//! (one 256-texel row per colormap id), so painting a swatch allocates nothing.

use crate::utils::colormap::{LUT_SIZE, Lut, registry};
use egui::{Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions};

#[derive(Default)]
pub struct SwatchAtlas {
    texture: Option<TextureHandle>,
    generation: u64,
    rows: usize,
}

impl SwatchAtlas {
    /// Rebuilds the texture when colormaps were added or removed.
    pub fn ensure(&mut self, ctx: &egui::Context) {
        let generation = registry::generation();
        if self.texture.is_some() && self.generation == generation {
            return;
        }
        let rows = registry::rows().max(1);
        let mut image = ColorImage::filled([LUT_SIZE, rows], Color32::BLACK);
        registry::for_each_lut(|id, lut| {
            let start = id as usize * LUT_SIZE;
            if let Some(dst) = image.pixels.get_mut(start..start + LUT_SIZE) {
                fill_row(dst, lut);
            }
        });
        match &mut self.texture {
            Some(tex) => tex.set(image, TextureOptions::LINEAR),
            None => {
                self.texture =
                    Some(ctx.load_texture("colormap_swatches", image, TextureOptions::LINEAR));
            }
        }
        self.rows = rows;
        self.generation = generation;
    }

    /// Paints colormap `id` across `rect`, optionally reversed.
    pub fn paint(&self, ui: &egui::Ui, rect: Rect, id: u32, reversed: bool) {
        let Some(tex) = &self.texture else { return };
        let painter = ui.painter();
        let v = (id as f32 + 0.5) / self.rows.max(1) as f32;
        let (u0, u1) = if reversed { (1.0, 0.0) } else { (0.0, 1.0) };
        let uv = Rect::from_min_max(Pos2::new(u0, v), Pos2::new(u1, v));
        painter.image(tex.id(), rect, uv, Color32::WHITE);
        painter.rect_stroke(
            rect,
            2.0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Inside,
        );
    }
}

/// A single-row texture for previewing an unsaved LUT (custom map editor).
#[derive(Default)]
pub struct PreviewSwatch {
    texture: Option<TextureHandle>,
}

impl PreviewSwatch {
    pub fn set(&mut self, ctx: &egui::Context, lut: &Lut) {
        let mut image = ColorImage::filled([LUT_SIZE, 1], Color32::BLACK);
        fill_row(&mut image.pixels, lut);
        match &mut self.texture {
            Some(tex) => tex.set(image, TextureOptions::LINEAR),
            None => {
                self.texture =
                    Some(ctx.load_texture("colormap_preview", image, TextureOptions::LINEAR))
            }
        }
    }

    pub fn clear(&mut self) {
        self.texture = None;
    }

    /// Paints the preview; nothing is drawn while the spec is invalid.
    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        if let Some(tex) = &self.texture {
            let uv = Rect::from_min_max(Pos2::new(0.0, 0.5), Pos2::new(1.0, 0.5));
            painter.image(tex.id(), rect, uv, Color32::WHITE);
        }
    }
}

fn fill_row(dst: &mut [Color32], lut: &Lut) {
    for (px, c) in dst.iter_mut().zip(lut.iter()) {
        *px = Color32::from_rgb(c[0], c[1], c[2]);
    }
}
