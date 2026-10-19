//! What a [`ToolbarButton`](super::ToolbarButton) tells about itself: its
//! name and on/off state to screen readers, and its tooltip (the label of a
//! compact button, its hover text, and the keys of its shortcut), shown on
//! hover and while it holds keyboard focus.

use crate::ui::keyboard::focus;
use crate::ui::keyboard::keymap::{self, Action};
use crate::ui::keyboard::labels;
use egui::Response;

pub(super) struct Tip<'a> {
    pub label: &'a str,
    pub hover: Option<&'a str>,
    pub compact: bool,
    /// `Some` for an on/off switch: whether it is on.
    pub switch: Option<bool>,
    pub shortcut: Option<Action>,
}

impl Tip<'_> {
    /// Describes the button of `response` and attaches its tooltip.
    pub fn attach(self, response: Response) -> Response {
        let name = if self.label.is_empty() {
            self.hover.unwrap_or_default()
        } else {
            self.label
        };
        response.widget_info(|| match self.switch {
            Some(on) => egui::WidgetInfo::selected(egui::Role::Button, true, on, name),
            None => egui::WidgetInfo::labeled(egui::Role::Button, true, name),
        });
        let keys = self.shortcut.and_then(keymap::shortcut_of);
        let labelled = self.compact || keys.is_some();
        let Some(tip) = self.hover.or(labelled.then_some(self.label)) else {
            return response;
        };
        focus::tooltip_ui(response, |ui| labels::tooltip(ui, tip, keys))
    }
}
