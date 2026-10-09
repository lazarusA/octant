//! A layer's colorbar label, editable in place. Idle, it is drawn from a
//! stack buffer; a click starts an edit whose text lives in egui temp memory
//! until the field loses focus, so only an edit allocates. Every change is
//! applied as it is typed, so the layer's other label views follow along.

use crate::app::layers::{LABEL_BUF, Layer};

/// How the label looks: framed like a text field (settings) or bare and
/// centered (colorbar titles).
#[derive(Clone, Copy)]
pub struct LabelEditor {
    pub id: egui::Id,
    pub width: f32,
    pub framed: bool,
}

impl LabelEditor {
    /// Draws `layer`'s label. Returns the new custom label whenever an edit
    /// changes the text or ends: `Some(None)` for the default (an empty text
    /// or the default itself).
    pub fn show(&self, ui: &mut egui::Ui, layer: &Layer) -> Option<Option<String>> {
        let mut default_buf = [0u8; LABEL_BUF];
        let default = layer.write_default_label(&mut default_buf);
        let shown = layer.color.custom_label.as_deref().unwrap_or(default);
        let editing = ui.data(|d| d.get_temp::<String>(self.id));
        match editing {
            Some(text) => self.edit(ui, text, default),
            None => {
                if self.idle(ui, shown).clicked() {
                    ui.data_mut(|d| d.insert_temp(self.id, shown.to_owned()));
                    ui.memory_mut(|m| m.request_focus(self.id.with("field")));
                }
                None
            }
        }
    }

    /// The label as text: framed like a field, or bare and centered.
    fn idle(&self, ui: &mut egui::Ui, shown: &str) -> egui::Response {
        let size = egui::vec2(self.width, ui.spacing().interact_size.y);
        let text = egui::RichText::new(shown);
        let response = if self.framed {
            let button = egui::Button::new(text).frame(true).truncate();
            ui.add_sized(size, button)
        } else {
            let label = egui::Label::new(text)
                .truncate()
                .sense(egui::Sense::click());
            ui.add_sized(size, label)
        };
        response
            .on_hover_cursor(egui::CursorIcon::Text)
            .on_hover_text("Colorbar title. Click to edit.")
    }

    /// The field while editing `text`; the custom label when the text changes
    /// or focus leaves.
    fn edit(&self, ui: &mut egui::Ui, mut text: String, default: &str) -> Option<Option<String>> {
        let mut field = egui::TextEdit::singleline(&mut text)
            .id(self.id.with("field"))
            .hint_text(default)
            .desired_width(self.width);
        if !self.framed {
            field = field
                .horizontal_align(egui::Align::Center)
                .frame(egui::Frame::NONE);
        }
        let response = ui.add(field);
        if response.lost_focus() {
            ui.data_mut(|d| d.remove::<String>(self.id));
            return Some(custom_label(text, default));
        }
        let edited = response
            .changed()
            .then(|| custom_label(text.clone(), default));
        ui.data_mut(|d| d.insert_temp(self.id, text));
        edited
    }
}

/// `text` as a custom label, or `None` (the default) when it is empty or
/// the default itself.
fn custom_label(text: String, default: &str) -> Option<String> {
    let custom = !text.trim().is_empty() && text != default;
    custom.then_some(text)
}
