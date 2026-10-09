//! A floating panel's title row: its icon and bold title, then the close
//! button in the top-right corner with the panel's drag grip left of it.

use crate::ui::drag_grip::{self, GripAction};
use crate::ui::icons::{Icon, IconSize, UiIconExt};

/// What a panel header's buttons asked for this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelHeader {
    pub close: bool,
    pub grip: GripAction,
}

impl Default for PanelHeader {
    fn default() -> Self {
        Self {
            close: false,
            grip: GripAction::None,
        }
    }
}

/// `icon`, bold `title`, then the grip and close button on the right.
pub fn show(ui: &mut egui::Ui, icon: Icon, title: &str, close_hover: &str) -> PanelHeader {
    ui.icon(icon, IconSize::Sm);
    ui.label(egui::RichText::new(title).strong());
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        buttons(ui, close_hover)
    })
    .inner
}

/// The close button, then the grip left of it, in a right-to-left `ui`.
pub fn buttons(ui: &mut egui::Ui, close_hover: &str) -> PanelHeader {
    let close = ui.close_button(close_hover).clicked();
    let grip = ui.add(drag_grip::button(IconSize::Sm));
    PanelHeader {
        close,
        grip: drag_grip::action(&grip),
    }
}
