//! `egui::Ui` helpers for drawing icons, icon labels and framed icon buttons.

use super::Icon;
use super::style::{ICON_GAP, IconSize, IconTone};
use egui::{Color32, Rect, Response, Sense, Ui, WidgetText, pos2, vec2};

/// Helper extension trait for easy rendering in egui UIs.
pub trait UiIconExt {
    /// Render an icon in the default text color.
    fn icon(&mut self, icon: Icon, size: IconSize) -> Response;

    /// Render an icon in a semantic, theme-adaptive tone.
    fn icon_toned(&mut self, icon: Icon, size: IconSize, tone: IconTone) -> Response;

    /// Render an icon in an explicit color. Prefer [`Self::icon_toned`]; use
    /// this only when the color comes from the surrounding widget state.
    fn icon_colored(&mut self, icon: Icon, size: IconSize, color: Color32) -> Response;

    /// Render a framed button with an `Sm` icon and optional label text.
    fn icon_button(&mut self, icon: Icon, text: impl Into<WidgetText>) -> Response;

    /// Render a non-interactive `Sm` icon followed by label text.
    fn icon_label(&mut self, icon: Icon, text: impl Into<WidgetText>) -> Response;
}

impl UiIconExt for Ui {
    fn icon(&mut self, icon: Icon, size: IconSize) -> Response {
        let color = self.visuals().text_color();
        self.icon_colored(icon, size, color)
    }

    fn icon_toned(&mut self, icon: Icon, size: IconSize, tone: IconTone) -> Response {
        let color = tone.color(self.visuals());
        self.icon_colored(icon, size, color)
    }

    fn icon_colored(&mut self, icon: Icon, size: IconSize, color: Color32) -> Response {
        let px = size.px();
        let (rect, response) = self.allocate_exact_size(vec2(px, px), Sense::hover());
        if self.is_rect_visible(rect) {
            icon.paint(self.painter(), rect, color, self.visuals().dark_mode);
        }
        response
    }

    fn icon_button(&mut self, icon: Icon, text: impl Into<WidgetText>) -> Response {
        let font_id = egui::TextStyle::Button.resolve(self.style());
        let galley = text
            .into()
            .into_galley(self, None, self.available_width(), font_id);

        let icon_size = IconSize::Sm.px();
        let gap = if galley.is_empty() { 0.0 } else { ICON_GAP };
        let padding = self.spacing().button_padding;
        let content_size = vec2(
            icon_size + gap + galley.size().x,
            icon_size.max(galley.size().y),
        );
        let (rect, response) =
            self.allocate_exact_size(content_size + padding * 2.0, Sense::click());

        if self.is_rect_visible(rect) {
            let visuals = self.style().interact(&response);
            self.painter().rect(
                rect,
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );
            let color = visuals.text_color();
            let origin = rect.min + padding;
            paint_icon_and_text(self, icon, origin, content_size.y, galley, gap, color);
        }
        response
    }

    fn icon_label(&mut self, icon: Icon, text: impl Into<WidgetText>) -> Response {
        let font_id = egui::TextStyle::Body.resolve(self.style());
        let galley = text
            .into()
            .into_galley(self, None, self.available_width(), font_id);

        let icon_size = IconSize::Sm.px();
        let gap = if galley.is_empty() { 0.0 } else { ICON_GAP };
        let size = vec2(
            icon_size + gap + galley.size().x,
            icon_size.max(galley.size().y),
        );
        let (rect, response) = self.allocate_exact_size(size, Sense::hover());

        if self.is_rect_visible(rect) {
            let color = self.visuals().text_color();
            paint_icon_and_text(self, icon, rect.min, size.y, galley, gap, color);
        }
        response
    }
}

/// Paint an `Sm` icon and its label, both vertically centred in a row of
/// height `row_h` starting at `origin`.
fn paint_icon_and_text(
    ui: &Ui,
    icon: Icon,
    origin: egui::Pos2,
    row_h: f32,
    galley: std::sync::Arc<egui::Galley>,
    gap: f32,
    color: Color32,
) {
    let icon_size = IconSize::Sm.px();
    let icon_rect = Rect::from_min_size(
        pos2(origin.x, origin.y + (row_h - icon_size) * 0.5),
        vec2(icon_size, icon_size),
    );
    icon.paint(ui.painter(), icon_rect, color, ui.visuals().dark_mode);

    if !galley.is_empty() {
        let text_pos = pos2(
            origin.x + icon_size + gap,
            origin.y + (row_h - galley.size().y) * 0.5,
        );
        ui.painter().galley(text_pos, galley, color);
    }
}
