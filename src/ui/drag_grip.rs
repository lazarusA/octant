//! The grip that moves a floating panel (colorbars, the Variables, Settings
//! and Dimensions panels): a `Grip` icon button that senses drags, read into
//! a `GripAction`, and dragged positions kept as fractions of the canvas so
//! a panel keeps its place when the canvas resizes. A focused grip moves its
//! panel with the arrow keys, and Enter puts it back, as a double-click.

use crate::i18n::{CommonKey, tr};
use crate::ui::icons::{Icon, IconSize, ToolbarButton};
use crate::ui::keyboard::focus;
use egui::{CursorIcon, Pos2, Rect, Response, Sense, Vec2};

/// How far one arrow key press moves a panel, in points.
const NUDGE: f32 = 16.0;

/// What the grip asks of its panel this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GripAction {
    None,
    /// Dragged by this much.
    Moved(Vec2),
    /// Double-clicked: back to the panel's default place.
    Reset,
}

/// A grip button with a `size` glyph.
pub fn button(size: IconSize) -> ToolbarButton<'static> {
    ToolbarButton::new(Icon::Grip, tr(CommonKey::Move))
        .compact(true)
        .icon_size(size)
        .sense(Sense::click_and_drag())
        .hover(tr(CommonKey::MoveHover))
}

/// Reads the grip's `response`, showing the grab cursor over it.
pub fn action(response: &Response) -> GripAction {
    if response.dragged() {
        response.ctx.set_cursor_icon(CursorIcon::Grabbing);
    } else if response.hovered() {
        response.ctx.set_cursor_icon(CursorIcon::Grab);
    }
    // A click that no mouse button made is Enter or Space.
    let keyed = response.clicked() && !response.clicked_by(egui::PointerButton::Primary);
    if response.double_clicked() || keyed {
        GripAction::Reset
    } else if response.dragged() {
        GripAction::Moved(response.drag_delta())
    } else {
        focus::arrow_on(response).map_or(GripAction::None, |d| GripAction::Moved(d * NUDGE))
    }
}

/// `point` as a fraction of `canvas`, clamped into it.
pub fn to_fraction(point: Pos2, canvas: Rect) -> Pos2 {
    let size = canvas.size().max(Vec2::splat(1.0));
    let f = (point - canvas.min) / size;
    Pos2::new(f.x.clamp(0.0, 1.0), f.y.clamp(0.0, 1.0))
}

/// The point at fraction `f` of `canvas`.
pub fn from_fraction(f: Pos2, canvas: Rect) -> Pos2 {
    canvas.min + f.to_vec2() * canvas.size()
}

/// The stored fraction `pos` after `action` moved the point `at` on
/// `canvas`: moved along, cleared by a reset, else unchanged.
pub fn apply(pos: Option<Pos2>, action: GripAction, at: Pos2, canvas: Rect) -> Option<Pos2> {
    match action {
        GripAction::None => pos,
        GripAction::Reset => None,
        GripAction::Moved(delta) => Some(to_fraction(at + delta, canvas)),
    }
}
