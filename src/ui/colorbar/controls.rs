//! A colorbar panel's controls: the cog opening its Color window in the
//! top-left corner, the drag grip and orientation flip in the top-right one;
//! shown while the pointer is over the panel (or the grip is held, the
//! Color window is open, or a touch on the panel revealed them), and never
//! during export.

use super::color_menu;
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::i18n::{ColormapKey as K, tr};
use crate::ui::drag_grip;
use crate::ui::icons::{Icon, IconSize, ToolbarButton};
use egui::{Align2, Rect, Vec2};

/// Glyph size of the controls.
const ICON: IconSize = IconSize::Md;
/// Side of a control: a compact `ToolbarButton` with an `ICON` glyph.
pub const CONTROL: f32 = ToolbarButton::side(ICON);
/// Inset of the controls from the panel's corners.
const INSET: f32 = 4.0;
/// Gap between the grip and the flip.
const GAP: f32 = 2.0;
/// Room the grip and flip take from the panel's right edge (and the cog,
/// with room to spare, from its left).
pub const SIDE_ROOM: f32 = INSET + 2.0 * CONTROL + GAP;
/// How far down from the panel's top the controls reach.
pub const REACH: f32 = INSET + CONTROL;

/// The grip and flip rects in panel `panel`'s top-right corner.
pub fn rects(panel: Rect) -> [Rect; 2] {
    let size = Vec2::splat(CONTROL);
    let flip = Rect::from_min_size(
        egui::pos2(panel.max.x - INSET - CONTROL, panel.min.y + INSET),
        size,
    );
    let grip = flip.translate(Vec2::new(-CONTROL - GAP, 0.0));
    [grip, flip]
}

/// The cog's rect in panel `panel`'s top-left corner.
pub fn cog_rect(panel: Rect) -> Rect {
    Rect::from_min_size(
        egui::pos2(panel.min.x + INSET, panel.min.y + INSET),
        Vec2::splat(CONTROL),
    )
}

/// Draws layer `id`'s controls on panel `panel` (pivot `pivot`, on
/// `canvas`): the cog opens and closes the Color window, dragging the grip
/// moves the panel, double-clicking it puts the panel back in its slot, and
/// the flip switches its orientation.
pub fn show(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    panel: Rect,
    (pivot, canvas): (Align2, Rect),
) {
    let dragging_key = egui::Id::unique(super::salt("colorbar_dragging", id));
    let dragging = ui.data(|d| d.get_temp::<bool>(dragging_key).unwrap_or(false));
    let color_open = color_menu::is_open(ui.ctx(), id);
    let shown = dragging || color_open || touch_revealed(ui, id, panel);
    if app.pending_export.is_some() || !(shown || ui.rect_contains_pointer(panel)) {
        return;
    }
    let cog = ToolbarButton::new(Icon::Settings, tr(K::ColorSettingsHover))
        .compact(true)
        .icon_size(ICON)
        .active(color_open);
    if ui.put(cog_rect(panel), cog).clicked() {
        color_menu::set_open(ui.ctx(), id, !color_open);
    }
    let [grip_rect, flip_rect] = rects(panel);
    let grip = ui.put(grip_rect, drag_grip::button(ICON));
    let flip = ToolbarButton::new(Icon::Orientation, tr(K::FlipOrientation))
        .compact(true)
        .icon_size(ICON);
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

/// Whether a touch revealed `panel`'s controls: touch has no hover, so a
/// touch on the panel shows them until a touch lands elsewhere.
fn touch_revealed(ui: &egui::Ui, id: LayerId, panel: Rect) -> bool {
    let key = egui::Id::unique(super::salt("colorbar_touch_revealed", id));
    let touch = ui.input(|i| {
        let pressed = i.any_touches() && i.pointer.any_pressed();
        pressed.then(|| i.pointer.press_origin()).flatten()
    });
    match touch {
        Some(at) => {
            let revealed = panel.contains(at);
            ui.data_mut(|d| d.insert_temp(key, revealed));
            revealed
        }
        None => ui.data(|d| d.get_temp(key).unwrap_or(false)),
    }
}
