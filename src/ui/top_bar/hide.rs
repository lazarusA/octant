//! Hiding the top bar, as the bottom bar collapses: dragging the bar's lower
//! edge up hides it to a thin strip; a click or tap on the strip (marked by a
//! down chevron), dragging it down, Enter or Space shows the bar again.

use crate::i18n::{CommonKey, tr};
use crate::ui::icons::{Icon, IconSize, IconTone};
use crate::ui::keyboard::keymap::{self, Action};
use crate::ui::keyboard::regions::{self, Region};
use crate::ui::keyboard::{focus, labels};

/// Height of the strip a hidden top bar leaves: the collapsed bottom bar's,
/// tall enough to tap on a phone.
const STRIP_HEIGHT: f32 = 20.0;

/// The panel the strip draws in, for `Panel::show_switched`.
pub(super) fn strip_panel(ui: &egui::Ui) -> egui::Panel {
    egui::Panel::top("octant_top_bar_hidden")
        .resizable(true)
        .exact_size(STRIP_HEIGHT)
        .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(0.0))
}

/// The strip in the hidden bar's place, the top bar's region for F6 while
/// hidden; whether it asks to show the bar.
pub(super) fn show_strip(ui: &mut egui::Ui) -> bool {
    let rect = ui.max_rect();
    let id = ui.scope_id().with("show_top_bar");
    let response = ui
        .interact(rect, id, egui::Sense::click_and_drag())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    focus::paint_ring(ui, &response, 0.0);

    let tone = if response.hovered() || response.has_focus() {
        IconTone::Strong
    } else {
        IconTone::Muted
    };
    let chevron = egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(IconSize::Xs.px()));
    let visuals = ui.visuals();
    Icon::ChevronDown.paint(
        ui.painter(),
        chevron,
        tone.color(visuals),
        visuals.dark_mode,
    );
    show_tooltip(ui, &response, chevron);
    regions::register(ui.ctx(), Region::TopBar, response.id, ui.layer_id(), rect);

    let show = response.clicked() || (response.dragged() && response.drag_delta().y > 1.0);
    // After `register` above, so focus goes into the bar, not this strip.
    if show && focus::keyboard_focused(&response) {
        regions::enter_when_drawn(ui.ctx(), Region::TopBar);
    }
    show
}

/// The strip's tooltip, centered under its chevron: egui's own would sit at
/// the strip's left end. Shown on hover and under keyboard focus.
fn show_tooltip(ui: &egui::Ui, strip: &egui::Response, chevron: egui::Rect) {
    let open = egui::Tooltip::should_show_tooltip(strip, false)
        || (focus::keyboard_focused(strip) && !strip.hovered());
    egui::Popup::new(
        strip.id.with("tooltip"),
        ui.ctx().clone(),
        chevron,
        ui.layer_id(),
    )
    .kind(egui::PopupKind::Tooltip)
    .align(egui::RectAlign::BOTTOM)
    .gap(4.0)
    .sense(egui::Sense::hover())
    .open(open)
    .show(|ui| {
        let keys = keymap::shortcut_of(Action::ToggleTopBar);
        labels::tooltip(ui, tr(CommonKey::ShowTopBarHover), keys);
    });
}
