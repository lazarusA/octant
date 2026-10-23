//! The scrolling spectrogram: a texture of [`COLUMNS`] columns written as a
//! ring, one column at a time, drawn from its oldest column so it scrolls
//! left without moving pixels.

use crate::utils::colormap::registry;
use crate::voice::spectrum::BANDS;

/// Columns kept: four seconds at 100 columns per second.
const COLUMNS: usize = 400;
const COLORMAP: &str = "matplotlib:magma";
const OPTIONS: egui::TextureOptions = egui::TextureOptions::LINEAR_REPEAT;

#[derive(Default)]
pub(super) struct Spectrogram {
    texture: Option<egui::TextureHandle>,
    /// The next column written, which is also the oldest shown.
    head: usize,
    colormap: Option<u32>,
}

impl Spectrogram {
    fn color(&mut self, t: f32) -> egui::Color32 {
        let id = *self
            .colormap
            .get_or_insert_with(|| registry::find(COLORMAP).unwrap_or_else(registry::default_id));
        registry::sample(id, t)
    }

    fn texture(&mut self, ctx: &egui::Context) -> &mut egui::TextureHandle {
        let silence = self.color(0.0);
        self.texture.get_or_insert_with(|| {
            let blank = egui::ColorImage::new([COLUMNS, BANDS], vec![silence; COLUMNS * BANDS]);
            ctx.load_texture("voice_spectrogram", blank, OPTIONS)
        })
    }

    /// Writes one column, lowest band at the bottom.
    pub(super) fn push(&mut self, ctx: &egui::Context, column: &[f32; BANDS]) {
        let pixels = column.iter().rev().map(|&t| self.color(t)).collect();
        let head = self.head;
        self.texture(ctx).set_partial(
            [head, 0],
            egui::ColorImage::new([1, BANDS], pixels),
            OPTIONS,
        );
        self.head = (head + 1) % COLUMNS;
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, size: egui::Vec2) {
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let u = self.head as f32 / COLUMNS as f32;
        let uv = egui::Rect::from_min_max(egui::pos2(u, 0.0), egui::pos2(u + 1.0, 1.0));
        let id = self.texture(ui.ctx()).id();
        let painter = ui.painter();
        painter.image(id, rect, uv, egui::Color32::WHITE);
        painter.rect_stroke(
            rect,
            2.0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Outside,
        );
    }
}
