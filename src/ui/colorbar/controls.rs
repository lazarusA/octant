//! A colorbar panel's drag grip and orientation flip, in its top-right
//! corner while the pointer is over the panel (or the grip is held), and
//! never during export.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::ui::drag_grip;
use crate::ui::icons::{Icon, IconSize, ToolbarButton};
use egui::{Align2, Rect, Vec2};

/// Side of a control: an `Xs` glyph in a `ToolbarButton`'s padding.
pub const CONTROL: f32 = IconSize::Xs.px() + 6.0;
/// Inset of the controls from the panel's top-right corner.
const INSET: f32 = 4.0;

/// The grip and flip rects in panel `panel`'s top-right corner.
pub fn rects(panel: Rect) -> [Rect; 2] {
    let size = Vec2::splat(CONTROL);
    let flip = Rect::from_min_size(
        egui::pos2(panel.max.x - INSET - CONTROL, panel.min.y + INSET),
        size,
    );
    let grip = flip.translate(Vec2::new(-CONTROL - 2.0, 0.0));
    [grip, flip]
}

/// Draws layer `id`'s controls on panel `panel` (pivot `pivot`, on
/// `canvas`): dragging the grip moves the panel, double-clicking it puts the
/// panel back in its slot, and the flip switches its orientation.
pub fn show(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    panel: Rect,
    (pivot, canvas): (Align2, Rect),
) {
    let dragging_key = egui::Id::new(super::salt("colorbar_dragging", id));
    let dragging = ui.data(|d| d.get_temp::<bool>(dragging_key).unwrap_or(false));
    if app.pending_export.is_some() || !(dragging || ui.rect_contains_pointer(panel)) {
        return;
    }
    let [grip_rect, flip_rect] = rects(panel);
    let grip = ui.put(grip_rect, drag_grip::button(IconSize::Xs));
    let flip = ToolbarButton::new(Icon::Orientation, "Flip orientation")
        .compact(true)
        .icon_size(IconSize::Xs);
    let flip = ui.put(flip_rect, flip);
    // Held from the press on, so a fast drag leaving the panel keeps its grip.
    let held = grip.dragged() || grip.is_pointer_button_down_on();
    ui.data_mut(|d| d.insert_temp(dragging_key, held));
    let action = drag_grip::action(&grip);
    let Some(placement) = app.layers.get_mut(id).map(|l| &mut l.colorbar) else {
        return;
    };
    let at = pivot.pos_in_rect(&panel);
    placement.pos = drag_grip::apply(placement.pos, action, at, canvas);
    if flip.clicked() {
        placement.orientation = placement.orientation.flipped();
    }
}
