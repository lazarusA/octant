//! Every keyboard shortcut in one table: which keys do what, and where.

use super::keymap::{Action, CanvasAction, Keys, Scope, Shortcut};
use crate::ui::panel_layout::Panel;
use egui::{Key, Modifiers};

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

const fn playback(action: Action, key: Key) -> Shortcut {
    Shortcut {
        action,
        keys: Keys::One(key),
        mods: Modifiers::NONE,
        scope: Scope::Playback,
    }
}

const fn free(action: Action, key: Key) -> Shortcut {
    Shortcut {
        action,
        keys: Keys::One(key),
        mods: Modifiers::NONE,
        scope: Scope::Free,
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
    global(Action::PlotStaged, Key::Enter, Modifiers::COMMAND),
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
    playback(Action::PlayPause, Key::Space),
    playback(Action::StepPrev, Key::Comma),
    playback(Action::StepNext, Key::Period),
    playback(Action::FirstStep, Key::Home),
    playback(Action::LastStep, Key::End),
    playback(Action::ToggleLoop, Key::L),
    free(Action::ShowShortcuts, Key::Questionmark),
    free(Action::ToggleHero, Key::H),
    free(Action::PlotTypePicker, Key::P),
    Shortcut {
        action: Action::LeaveHero,
        keys: Keys::One(Key::Escape),
        mods: Modifiers::NONE,
        scope: Scope::Hero,
    },
];
