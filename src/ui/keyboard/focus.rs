//! Keyboard focus shared by every widget that navigates with keys. egui
//! moves focus spatially on arrow keys; widgets that navigate themselves
//! claim the arrows so only their own logic moves focus. Focus rings show
//! only while the keyboard is in use (`visible`), and the widget focused
//! when each frame ended is kept (`last`): egui drops focus on Escape before
//! the app runs, so the panel Escape closes is read from it. Focus across
//! windows opening and closing is `windows`'.

use super::keymap::key_press;
use egui::{Context, Event, EventFilter, FocusDirection, Id, Key, LayerId, Rect, Response, Ui};

/// While focused, the widget owns every arrow key and Escape; Tab still moves
/// focus on as usual.
const ARROW_FILTER: EventFilter = EventFilter {
    tab: false,
    horizontal_arrows: true,
    vertical_arrows: true,
    escape: true,
};

const ARROWS: [Key; 4] = [
    Key::ArrowUp,
    Key::ArrowDown,
    Key::ArrowLeft,
    Key::ArrowRight,
];

/// Focus `id` from keyboard navigation (or a click). egui only accepts the
/// key filter once the widget has held focus for a frame, so repaint at once
/// to claim the keys before the next press.
pub fn request(ctx: &Context, id: Id) {
    ctx.memory_mut(|m| {
        m.request_focus(id);
        m.move_focus(FocusDirection::None);
    });
    ctx.request_repaint();
}

/// Call every frame the widget `id` holds focus: keeps the arrow keys and
/// Escape claimed, and cancels egui's spatial move on frames where the
/// filter is not in place yet.
pub fn claim_arrows(ctx: &Context, id: Id) {
    claim(ctx, id, ARROW_FILTER);
}

/// [`claim_arrows`] leaving Escape to egui, which then drops focus: for a
/// panel's grip, so Escape closes the panel.
pub fn claim_arrows_only(ctx: &Context, id: Id) {
    let filter = EventFilter {
        escape: false,
        ..ARROW_FILTER
    };
    claim(ctx, id, filter);
}

fn claim(ctx: &Context, id: Id, filter: EventFilter) {
    let arrow = ctx.input(|i| ARROWS.iter().any(|&k| i.key_pressed(k)));
    ctx.memory_mut(|m| {
        m.set_focus_lock_filter(id, filter);
        if arrow {
            m.move_focus(FocusDirection::None);
        }
    });
}

/// The first of `keys` pressed this frame (key repeat included) without
/// modifiers. Reads each key event's own modifiers, so Shift+Down or
/// Cmd+Down never count as a plain Down.
pub fn pressed<const N: usize>(ctx: &Context, keys: [Key; N]) -> Option<Key> {
    ctx.input(|i| {
        i.events
            .iter()
            .filter_map(key_press)
            .find(|(key, modifiers)| modifiers.is_none() && keys.contains(key))
            .map(|(key, _)| key)
    })
}

/// The item a list key moves to from item `at` of `len`: Up or Left the
/// previous, Down or Right the next, Home the first, End the last; `None`
/// past either end and for other keys.
pub fn step_index(key: Key, at: usize, len: usize) -> Option<usize> {
    let last = len.checked_sub(1)?;
    match key {
        Key::ArrowUp | Key::ArrowLeft => at.checked_sub(1),
        Key::ArrowDown | Key::ArrowRight => (at < last).then_some(at + 1),
        Key::Home => Some(0),
        Key::End => Some(last),
        _ => None,
    }
}

/// The arrow pressed this frame without modifiers, as a direction (y down).
pub fn pressed_arrow(ctx: &Context) -> Option<egui::Vec2> {
    super::keymap::arrow_direction(pressed(ctx, ARROWS)?)
}

/// The arrow pressed on `response` while it holds focus, keeping the arrows
/// from egui's own focus moves (Escape still drops focus).
pub fn arrow_on(response: &Response) -> Option<egui::Vec2> {
    if !response.has_focus() {
        return None;
    }
    claim_arrows_only(&response.ctx, response.id);
    pressed_arrow(&response.ctx)
}

fn visible_id() -> Id {
    Id::unique("keyboard_focus_visible")
}

fn advance_id() -> Id {
    Id::unique("keyboard_focus_advance")
}

fn last_id() -> Id {
    Id::unique("keyboard_focus_last")
}

/// Call once per frame before any widget: records whether the keyboard or
/// the pointer was used last, and moves focus on as Tab would when
/// [`advance_next_frame`] asked for it.
pub fn begin_frame(ctx: &Context) {
    let keyboard = ctx.input(|i| {
        i.events.iter().rev().find_map(|e| match e {
            Event::Key { pressed: true, .. } => Some(true),
            Event::PointerButton { pressed: true, .. } | Event::Touch { .. } => Some(false),
            _ => None,
        })
    });
    let advance = ctx.data_mut(|d| {
        if let Some(keyboard) = keyboard {
            d.insert_temp(visible_id(), keyboard);
        }
        d.remove_temp::<bool>(advance_id()).is_some()
    });
    if advance {
        ctx.memory_mut(|m| m.move_focus(FocusDirection::Next));
    }
}

/// Whether focus moves by keyboard: rings show only then, not after clicks.
pub fn visible(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp(visible_id()).unwrap_or(false))
}

/// Next frame, move focus on from the focused widget as Tab would: into a
/// popup its button opened this frame, drawn right after it.
pub fn advance_next_frame(ctx: &Context) {
    ctx.data_mut(|d| d.insert_temp(advance_id(), true));
    ctx.request_repaint();
}

/// Whether `response` holds focus the keyboard gave it.
pub fn keyboard_focused(response: &Response) -> bool {
    response.has_focus() && visible(&response.ctx)
}

/// `add_contents` as `response`'s tooltip, on hover and under keyboard
/// focus, so a widget that says what it is only on hover still says it.
pub fn tooltip_ui(response: Response, add_contents: impl Fn(&mut Ui)) -> Response {
    if keyboard_focused(&response) && !response.hovered() {
        response.show_tooltip_ui(&add_contents);
    }
    response.on_hover_ui(add_contents)
}

/// [`paint_ring`] and [`tooltip_ui`] together: a custom widget that shows
/// keyboard focus and says what it is on hover and under keyboard focus.
pub fn ring_and_tooltip(
    ui: &Ui,
    response: Response,
    radius: impl Into<egui::CornerRadius>,
    add_contents: impl Fn(&mut Ui),
) -> Response {
    paint_ring(ui, &response, radius);
    tooltip_ui(response, add_contents)
}

/// Keyboard focus outline drawn inside `response`'s rect while it holds
/// keyboard focus, shared by every custom widget so focus looks the same
/// everywhere.
pub fn paint_ring(ui: &Ui, response: &Response, radius: impl Into<egui::CornerRadius>) {
    if keyboard_focused(response) {
        let stroke = ui.visuals().widgets.hovered.bg_stroke;
        ui.painter()
            .rect_stroke(response.rect, radius, stroke, egui::StrokeKind::Inside);
    }
}

/// The response of the widget holding focus, as last drawn.
fn focused_response(ctx: &Context) -> Option<Response> {
    let focused = ctx.memory(|m| m.focused())?;
    ctx.read_response(focused)
}

/// Whether the widget holding focus is drawn in the area (`egui::Area`,
/// window) `area`.
pub fn in_area(ctx: &Context, area: Id) -> bool {
    focused_response(ctx).is_some_and(|r| r.layer_id.id == area)
}

/// The area Escape took focus from this frame: egui drops focus on Escape
/// before the app runs, unless the widget keeps Escape for itself. `None`
/// while a popup is open, as Escape closes the popup.
pub fn escaped_area(ctx: &Context) -> Option<Id> {
    let escaped = ctx.input(|i| i.key_pressed(Key::Escape))
        && ctx.memory(|m| m.focused()).is_none()
        && !egui::Popup::is_any_open(ctx);
    escaped.then(|| last(ctx).map(|l| l.layer.id)).flatten()
}

/// The widget focused when a frame ended: where it was drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LastFocus {
    pub id: Id,
    pub layer: LayerId,
    pub rect: Rect,
}

/// Call once per frame after every widget: keeps where focus is for the next
/// frame, read before egui's Escape handling is undone.
pub fn end_frame(ctx: &Context) {
    let last = focused_response(ctx).map(|r| LastFocus {
        id: r.id,
        layer: r.layer_id,
        rect: r.rect,
    });
    ctx.data_mut(|d| match last {
        Some(last) => {
            d.insert_temp(last_id(), last);
        }
        None => d.remove::<LastFocus>(last_id()),
    });
}

/// The widget focused when the last frame ended.
pub fn last(ctx: &Context) -> Option<LastFocus> {
    ctx.data(|d| d.get_temp(last_id()))
}
