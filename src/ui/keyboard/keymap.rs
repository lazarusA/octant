//! What keyboard shortcuts are: their actions, the scopes they work in and
//! the keys they answer to, and how a press is matched and taken. The one
//! table of shortcuts is `bindings::TABLE`; the dispatcher, the canvas and
//! the shortcut list all read it, so a key is bound in one place.

use crate::i18n::KeymapKey;
use crate::ui::panel_layout::Panel;
use egui::{Context, Event, Key, Modifiers, Vec2};

/// What a shortcut does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    QuickSave,
    ExportDialog,
    ToggleCrop,
    /// Opens a panel and moves focus into it; moves focus into it when it
    /// is open; closes it when focus is already inside.
    GoToPanel(Panel),
    /// Moves focus to the next colorbar's bar.
    GoToColorbar,
    SearchVariables,
    /// Closes the panel keyboard focus was in.
    ClosePanel,
    NextRegion,
    PreviousRegion,
    FocusTopBar,
    PlayPause,
    StepPrev,
    StepNext,
    FirstStep,
    LastStep,
    ToggleLoop,
    ShowShortcuts,
    /// Shows the landing screen, or the plot again.
    ToggleHero,
    /// Leaves the landing screen for the plot.
    LeaveHero,
    /// Opens the centered plot type picker.
    PlotTypePicker,
    /// Plots the staged variable, as the Dimensions panel's Plot Data.
    PlotStaged,
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
    /// and no popup took it (`focus::escaped_area`).
    Panel,
    /// Single keys while no text field takes the keys and no popup or
    /// modal window is open, wherever focus is.
    Free,
    /// Escape on the landing screen while a plot waits behind it, once no
    /// widget kept Escape for itself.
    Hero,
    /// Single keys on a shown plot while nothing or the canvas holds focus
    /// (a focused widget may read arrows, Space or Enter itself) and no popup
    /// or modal window is open.
    Canvas,
    /// Canvas keys of a plot with an animated dimension.
    Playback,
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
    /// What the shortcut list says it does.
    pub label: KeymapKey,
}

pub use super::bindings::TABLE;

/// The first shortcut bound to `action`, as tooltips name it.
pub fn shortcut_of(action: Action) -> Option<&'static Shortcut> {
    TABLE.iter().find(|s| s.action == action)
}

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
    let shift_free = matches!(key, Key::Plus | Key::Questionmark);
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

/// A key press's key and modifiers (key repeat included); `None` for every
/// other event.
pub(super) fn key_press(event: &Event) -> Option<(Key, Modifiers)> {
    match event {
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => Some((*key, *modifiers)),
        _ => None,
    }
}

/// Takes the first press of `shortcut` from this frame's input (key repeat
/// included), so no widget reads it after; the arrow's direction, or zero.
pub fn take(ctx: &Context, shortcut: &Shortcut) -> Option<Vec2> {
    ctx.input_mut(|i| {
        let (at, direction) = i.events.iter().enumerate().find_map(|(at, e)| {
            let (key, modifiers) = key_press(e)?;
            modifiers_match(key, modifiers, shortcut.mods)
                .then(|| shortcut.keys.direction(key).map(|d| (at, d)))
                .flatten()
        })?;
        i.events.remove(at);
        Some(direction)
    })
}

/// Whether a key was pressed this frame: shortcuts are looked up only then.
pub fn any_pressed(ctx: &Context) -> bool {
    ctx.input(|i| i.events.iter().any(|e| key_press(e).is_some()))
}

/// Takes the first press of any shortcut for `action` whose scope `allows`.
pub fn take_action(ctx: &Context, action: Action, allows: impl Fn(Scope) -> bool) -> Option<Vec2> {
    TABLE
        .iter()
        .filter(|s| s.action == action && allows(s.scope))
        .find_map(|s| take(ctx, s))
}
