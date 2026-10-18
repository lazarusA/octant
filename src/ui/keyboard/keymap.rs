//! Every keyboard shortcut in one table: what each key does, and where it
//! works. The dispatcher, the canvas and the shortcut list all read it, so a
//! key is bound in one place.

use crate::ui::panel_layout::Panel;
use egui::{Context, Event, Key, Modifiers, Vec2};

/// What a shortcut does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    QuickSave,
    ExportDialog,
    ToggleCrop,
    TogglePanel(Panel),
    SearchVariables,
    /// Closes the panel keyboard focus was in.
    ClosePanel,
    NextRegion,
    PreviousRegion,
    FocusTopBar,
    /// Run by the canvas as it draws, knowing the plot's geometry.
    Canvas(CanvasAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasAction {
    /// Pans a 2D plot or orbits a 3D one; moves the probe or the crop box.
    Move,
    /// Pans further (2D) or pans (3D); moves the probe further or resizes
    /// the crop box.
    MoveAlt,
    ZoomIn,
    ZoomOut,
    ResetView,
    ToggleProbe,
    /// Stops the probe, else lets the canvas go.
    LeaveProbe,
}

/// Where a shortcut works.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Modifier shortcuts and function keys: always, also while typing.
    Global,
    /// Escape in a floating panel, once the widget focused there let it go
    /// and no popup took it.
    Panel,
    /// Single keys on a shown plot: only while nothing or the canvas holds
    /// focus and no popup or modal window is open.
    Canvas,
}

/// The keys a shortcut answers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keys {
    One(Key),
    /// Either key, as `+` and `=` share a key on many layouts.
    Either(Key, Key),
    /// The four arrows; the shortcut reads which one as a direction.
    Arrows,
}

#[derive(Clone, Copy, Debug)]
pub struct Shortcut {
    pub action: Action,
    pub keys: Keys,
    pub mods: Modifiers,
    pub scope: Scope,
}

const fn global(action: Action, key: Key, mods: Modifiers) -> Shortcut {
    Shortcut {
        action,
        keys: Keys::One(key),
        mods,
        scope: Scope::Global,
    }
}

const fn canvas(action: Action, keys: Keys) -> Shortcut {
    Shortcut {
        action,
        keys,
        mods: Modifiers::NONE,
        scope: Scope::Canvas,
    }
}

const CMD_SHIFT: Modifiers = Modifiers::COMMAND.plus(Modifiers::SHIFT);

pub const TABLE: &[Shortcut] = &[
    global(Action::QuickSave, Key::S, Modifiers::COMMAND),
    global(Action::ExportDialog, Key::S, CMD_SHIFT),
    global(
        Action::TogglePanel(Panel::Dataset),
        Key::Num1,
        Modifiers::COMMAND,
    ),
    global(
        Action::TogglePanel(Panel::Variables),
        Key::Num2,
        Modifiers::COMMAND,
    ),
    global(
        Action::TogglePanel(Panel::Dimensions),
        Key::Num3,
        Modifiers::COMMAND,
    ),
    global(
        Action::TogglePanel(Panel::Settings),
        Key::Num4,
        Modifiers::COMMAND,
    ),
    global(Action::SearchVariables, Key::K, Modifiers::COMMAND),
    Shortcut {
        action: Action::ClosePanel,
        keys: Keys::One(Key::Escape),
        mods: Modifiers::NONE,
        scope: Scope::Panel,
    },
    global(Action::NextRegion, Key::F6, Modifiers::NONE),
    global(Action::PreviousRegion, Key::F6, Modifiers::SHIFT),
    global(Action::FocusTopBar, Key::F10, Modifiers::NONE),
    canvas(Action::ToggleCrop, Keys::One(Key::C)),
    canvas(Action::Canvas(CanvasAction::Move), Keys::Arrows),
    Shortcut {
        mods: Modifiers::SHIFT,
        ..canvas(Action::Canvas(CanvasAction::MoveAlt), Keys::Arrows)
    },
    canvas(
        Action::Canvas(CanvasAction::ZoomIn),
        Keys::Either(Key::Plus, Key::Equals),
    ),
    canvas(Action::Canvas(CanvasAction::ZoomOut), Keys::One(Key::Minus)),
    canvas(
        Action::Canvas(CanvasAction::ResetView),
        Keys::One(Key::Num0),
    ),
    canvas(
        Action::Canvas(CanvasAction::ToggleProbe),
        Keys::One(Key::Enter),
    ),
    canvas(
        Action::Canvas(CanvasAction::LeaveProbe),
        Keys::One(Key::Escape),
    ),
];

/// An arrow key's direction on screen (y down).
pub fn arrow_direction(key: Key) -> Option<Vec2> {
    match key {
        Key::ArrowLeft => Some(-Vec2::X),
        Key::ArrowRight => Some(Vec2::X),
        Key::ArrowUp => Some(-Vec2::Y),
        Key::ArrowDown => Some(Vec2::Y),
        _ => None,
    }
}

impl Keys {
    /// Whether `key` is one of these keys: the arrow's direction, zero for
    /// the others.
    fn direction(self, key: Key) -> Option<Vec2> {
        match self {
            Keys::One(k) => (k == key).then_some(Vec2::ZERO),
            Keys::Either(a, b) => (a == key || b == key).then_some(Vec2::ZERO),
            Keys::Arrows => arrow_direction(key),
        }
    }
}

/// Punctuation reached with Shift on some layouts (`+`, `?`) matches with or
/// without it; every other key needs its exact modifiers, so Cmd+Shift+S
/// never counts as Cmd+S.
fn modifiers_match(key: Key, pressed: Modifiers, wanted: Modifiers) -> bool {
    let shift_free = matches!(
        key,
        Key::Plus | Key::Equals | Key::Minus | Key::Questionmark | Key::Comma | Key::Period
    );
    let pressed = if shift_free {
        Modifiers {
            shift: wanted.shift,
            ..pressed
        }
    } else {
        pressed
    };
    pressed.matches_exact(wanted)
}

/// Takes the first press of `shortcut` from this frame's input (key repeat
/// included), so no widget reads it after; the arrow's direction, or zero.
pub fn take(ctx: &Context, shortcut: &Shortcut) -> Option<Vec2> {
    ctx.input_mut(|i| {
        let (at, direction) = i.events.iter().enumerate().find_map(|(at, e)| match e {
            Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } if modifiers_match(*key, *modifiers, shortcut.mods) => {
                shortcut.keys.direction(*key).map(|d| (at, d))
            }
            _ => None,
        })?;
        i.events.remove(at);
        Some(direction)
    })
}

/// Takes the first press of any shortcut for `action` whose scope `allows`.
pub fn take_action(ctx: &Context, action: Action, allows: impl Fn(Scope) -> bool) -> Option<Vec2> {
    TABLE
        .iter()
        .filter(|s| s.action == action && allows(s.scope))
        .find_map(|s| take(ctx, s))
}
