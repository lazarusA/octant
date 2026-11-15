//! A colorbar panel's controls: the cog opening its Color window and the
//! info button opening its variable's attributes in the top-left corner, the drag grip and orientation flip in the top-right one;
//! shown while the pointer is over the panel (or the grip is held, the
//! Color window is open, keyboard focus is in the panel, or a touch on the
//! panel revealed them), and never during export. The panel registers as
//! part of the keyboard's colorbars region, entered at its bar.

use super::color_menu;
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::i18n::{ColormapKey as K, VariablesKey, tr};
use crate::ui::drag_grip;
use crate::ui::icons::{Icon, IconSize, ToolbarButton};
use crate::ui::keyboard::{focus, regions};
use crate::ui::variables_panel::show_attributes;
use egui::{Align2, Rect, Vec2};

/// Glyph size of the controls.
const ICON: IconSize = IconSize::Md;
/// Side of a control: a compact `ToolbarButton` with an `ICON` glyph.
pub const CONTROL: f32 = ToolbarButton::side(ICON);
/// Inset of the controls from the panel's corners.
const INSET: f32 = 4.0;
/// Gap between the grip and the flip, and between the cog and the info button.
const GAP: f32 = 2.0;
/// Width of the attributes popup the info button opens.
const INFO_POPUP_W: f32 = 320.0;
/// Room the grip and flip take from the panel's right edge (and the cog and
/// info button from its left).
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

/// The info button's rect, right of the cog.
pub fn info_rect(panel: Rect) -> Rect {
    cog_rect(panel).translate(Vec2::new(CONTROL + GAP, 0.0))
}

/// Draws layer `id`'s controls on panel `panel` (pivot `pivot`, on
/// `canvas`): the cog opens and closes the Color window, the info button
/// the variable's attributes, dragging the grip
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
    let info_id = egui::Id::unique(super::salt("colorbar_info", id));
    let info_open = egui::Popup::is_id_open(ui.ctx(), info_id);
    let (ctx, layer) = (ui.ctx(), ui.layer_id());
    regions::register(
        ctx,
        regions::Region::Colorbars,
        super::bar_id(id),
        layer,
        panel,
    );
    let focused = focus::in_area(ctx, layer.id);
    let shown = dragging || color_open || info_open || focused || touch_revealed(ui, id, panel);
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
    info_button(app, ui, id, panel, info_id);
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

/// The info button right of the cog, opening layer `id`'s variable's
/// attributes in popup `popup_id`; left out when it has none.
fn info_button(app: &OctantApp, ui: &mut egui::Ui, id: LayerId, panel: Rect, popup_id: egui::Id) {
    let layer = app.layers.get(id);
    let Some(info) = layer.and_then(|l| l.selection().variable_info()) else {
        return;
    };
    if info.attributes.is_empty() {
        return;
    }
    let button = ToolbarButton::new(Icon::Info, tr(VariablesKey::InfoAttributes))
        .compact(true)
        .icon_size(ICON)
        .active(egui::Popup::is_id_open(ui.ctx(), popup_id));
    let response = ui.put(info_rect(panel), button);
    let popup = egui::Popup::from_toggle_button_response(&response).id(popup_id);
    show_attributes(popup, INFO_POPUP_W, info);
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
