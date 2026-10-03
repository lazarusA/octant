//! `egui::Ui` helpers for drawing icons, framed icon buttons, the standard
//! close button and the search field.

use super::style::{ICON_GAP, IconSize, IconTone};
use super::{Icon, ToolbarButton};
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

    /// Render the standard frameless close (or clear) button: an `Sm` cross
    /// that highlights on hover and shows `hover` as its tooltip. Place close
    /// buttons in the top-right corner of their panel or window.
    fn close_button(&mut self, hover: &str) -> Response;

    /// Panel title row: `Sm` icon, bold `title`, and a close button pinned to
    /// the top-right. Returns `true` when the close button was clicked.
    fn panel_header(&mut self, icon: Icon, title: &str, close_hover: &str) -> bool;

    /// Search row: `Search` icon, a single-line field and, while `text` is
    /// non-empty, a clear button. The field fills the row unless `width` is
    /// given. Returns `true` when the text changed (typed or cleared).
    fn search_field(&mut self, text: &mut String, hint: &str, width: Option<f32>) -> bool;

    /// [`Self::search_field`] returning the text field's [`Response`], for
    /// callers that drive its focus. `changed()` also covers the clear button.
    fn search_field_response(
        &mut self,
        text: &mut String,
        hint: &str,
        width: Option<f32>,
    ) -> Response;

    /// Render a button like [`Self::icon_button`] with no background at rest
    /// and a `tone` outline, marking the primary action of its panel. The
    /// usual fill appears on hover and press.
    fn outlined_icon_button(
        &mut self,
        icon: Icon,
        text: impl Into<WidgetText>,
        tone: IconTone,
    ) -> Response;
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
        framed_icon_button(self, icon, text.into(), None)
    }

    fn close_button(&mut self, hover: &str) -> Response {
        self.add(close_button_widget(hover))
    }

    fn panel_header(&mut self, icon: Icon, title: &str, close_hover: &str) -> bool {
        self.icon(icon, IconSize::Sm);
        self.label(egui::RichText::new(title).strong());
        self.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.close_button(close_hover).clicked()
        })
        .inner
    }

    fn search_field(&mut self, text: &mut String, hint: &str, width: Option<f32>) -> bool {
        self.search_field_response(text, hint, width).changed()
    }

    fn search_field_response(
        &mut self,
        text: &mut String,
        hint: &str,
        width: Option<f32>,
    ) -> Response {
        const HOVER: &str = "Clear search";
        self.horizontal(|ui| {
            ui.icon(Icon::Search, IconSize::Sm);
            let has_text = !text.is_empty();
            // Reserve exactly the clear button's measured width.
            let clear_w = if has_text {
                close_button_widget(HOVER).width(ui) + ui.spacing().item_spacing.x
            } else {
                0.0
            };
            let field_w = width.unwrap_or(ui.available_width() - clear_w).max(60.0);
            let edit = egui::TextEdit::singleline(text)
                .hint_text(hint)
                .desired_width(field_w);
            let mut resp = ui.add(edit);
            if has_text && ui.close_button(HOVER).clicked() {
                text.clear();
                resp.mark_changed();
            }
            resp
        })
        .inner
    }

    fn outlined_icon_button(
        &mut self,
        icon: Icon,
        text: impl Into<WidgetText>,
        tone: IconTone,
    ) -> Response {
        let outline = tone.color(self.visuals());
        framed_icon_button(self, icon, text.into(), Some(outline))
    }
}

/// The standard close / clear button widget, shared by [`UiIconExt::close_button`]
/// and layouts that need its width.
fn close_button_widget(hover: &str) -> ToolbarButton<'_> {
    ToolbarButton::new(Icon::Cross, hover)
        .compact(true)
        .icon_size(IconSize::Sm)
}

/// Framed button with an `Sm` icon and optional label. When `outline` is set
/// it replaces the frame stroke (thicker while hovered) and the background is
/// only drawn while hovered or pressed.
fn framed_icon_button(
    ui: &mut Ui,
    icon: Icon,
    text: WidgetText,
    outline: Option<Color32>,
) -> Response {
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let galley = text.into_galley(ui, None, ui.available_width(), font_id);

    let icon_size = IconSize::Sm.px();
    let gap = if galley.is_empty() { 0.0 } else { ICON_GAP };
    let padding = ui.spacing().button_padding;
    let content_size = vec2(
        icon_size + gap + galley.size().x,
        icon_size.max(galley.size().y),
    );
    let (rect, response) = ui.allocate_exact_size(content_size + padding * 2.0, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let interacting = response.hovered() || response.is_pointer_button_down_on();
        let (fill, stroke) = match outline {
            Some(color) => {
                let width = if response.hovered() { 1.5 } else { 1.0 };
                let fill = if interacting {
                    visuals.bg_fill
                } else {
                    Color32::TRANSPARENT
                };
                (fill, egui::Stroke::new(width, color))
            }
            None => (visuals.bg_fill, visuals.bg_stroke),
        };
        ui.painter().rect(
            rect,
            visuals.corner_radius,
            fill,
            stroke,
            egui::StrokeKind::Inside,
        );
        let color = visuals.text_color();
        let origin = rect.min + padding;
        paint_icon_and_text(ui, icon, origin, content_size.y, galley, gap, color);
    }
    response
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
