//! What a [`ToolbarButton`](super::ToolbarButton) tells about itself: its
//! name and on/off state to screen readers, and its tooltip (the label of a
//! compact button, its hover text, and the keys of its shortcut), shown on
//! hover and while it holds keyboard focus.

use crate::i18n::{KeymapKey, tr};
use crate::ui::keyboard::focus;
use crate::ui::keyboard::keymap::{self, Action, Shortcut};
use crate::ui::keyboard::labels::{self, KeysBuf};
use egui::{Response, Ui};

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
        let tooltip = |ui: &mut Ui| tooltip(ui, tip, keys);
        // Keyboard focus shows the tooltip a pointer would, so an icon-only
        // button still says what it does.
        if focus::shows_tooltip(&response) {
            response.show_tooltip_ui(tooltip);
        }
        response.on_hover_ui(tooltip)
    }
}

/// `tip`, with the keys of `shortcut` after it.
fn tooltip(ui: &mut Ui, tip: &str, shortcut: Option<&Shortcut>) {
    let Some(shortcut) = shortcut else {
        ui.label(tip);
        return;
    };
    let mut keys_buf: KeysBuf = [0; 48];
    let keys = labels::write_keys(&mut keys_buf, shortcut, labels::is_mac(ui.ctx()));
    let mut buf = [0u8; 256];
    ui.label(crate::i18n::fill(
        &mut buf,
        tr(KeymapKey::Hint),
        &[&tip, &keys],
    ));
}
