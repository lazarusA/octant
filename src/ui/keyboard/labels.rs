//! How shortcuts read: what each does and where it works, translated, and
//! its keys (`Cmd+1`, `Ctrl+1`, `Shift+Arrows`) written into a stack
//! buffer. Key and modifier names stay English, as printed on keyboards.

use super::keymap::{Action, CanvasAction, Keys, Scope, Shortcut};
use crate::i18n::{KeymapKey as K, tr};
use crate::ui::panel_layout::Panel;
use crate::utils::stack_str;
use egui::{Key, ModifierNames, Modifiers};
use std::fmt;

/// Stack buffer for a shortcut's keys.
pub type KeysBuf = [u8; 48];

pub fn describe(action: Action) -> K {
    match action {
        Action::QuickSave => K::QuickSave,
        Action::ExportDialog => K::ExportDialog,
        Action::ToggleCrop => K::ToggleCrop,
        Action::TogglePanel(Panel::Dataset) => K::ToggleDataset,
        Action::TogglePanel(Panel::Variables) => K::ToggleVariables,
        Action::TogglePanel(Panel::Dimensions) => K::ToggleDimensions,
        Action::TogglePanel(Panel::Settings) => K::ToggleSettings,
        Action::SearchVariables => K::SearchVariables,
        Action::ClosePanel => K::ClosePanel,
        Action::NextRegion => K::NextRegion,
        Action::PreviousRegion => K::PreviousRegion,
        Action::FocusTopBar => K::FocusTopBar,
        Action::PlayPause => K::PlayPause,
        Action::StepPrev => K::StepPrev,
        Action::StepNext => K::StepNext,
        Action::FirstStep => K::FirstStep,
        Action::LastStep => K::LastStep,
        Action::ToggleLoop => K::ToggleLoop,
        Action::ShowShortcuts => K::ShowShortcuts,
        Action::Canvas(CanvasAction::Move) => K::Move,
        Action::Canvas(CanvasAction::MoveAlt) => K::MoveAlt,
        Action::Canvas(CanvasAction::ZoomIn) => K::ZoomIn,
        Action::Canvas(CanvasAction::ZoomOut) => K::ZoomOut,
        Action::Canvas(CanvasAction::ResetView) => K::ResetView,
        Action::Canvas(CanvasAction::ToggleProbe) => K::ToggleProbe,
        Action::Canvas(CanvasAction::LeaveProbe) => K::LeaveProbe,
    }
}

/// Every scope, in the order the shortcut list shows them.
pub const SCOPES: [Scope; 5] = [
    Scope::Global,
    Scope::Free,
    Scope::Canvas,
    Scope::Playback,
    Scope::Panel,
];

pub fn scope_title(scope: Scope) -> K {
    match scope {
        Scope::Global => K::ScopeGlobal,
        Scope::Panel => K::ScopePanel,
        Scope::Free => K::ScopeFree,
        Scope::Canvas => K::ScopeCanvas,
        Scope::Playback => K::ScopePlayback,
    }
}

/// Modifiers as `Cmd+Shift+` (`Ctrl+` off macOS).
struct ModifierText {
    mods: Modifiers,
    mac: bool,
}

impl fmt::Display for ModifierText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = ModifierNames::NAMES;
        let m = self.mods;
        let command = m.command || m.mac_cmd || m.ctrl;
        let parts = [
            (command, if self.mac { names.mac_cmd } else { names.ctrl }),
            (m.shift, names.shift),
            (m.alt, if self.mac { names.mac_alt } else { names.alt }),
        ];
        for (_, name) in parts.into_iter().filter(|(held, _)| *held) {
            write!(f, "{name}{}", names.concat)?;
        }
        Ok(())
    }
}

/// A key's name; `Minus` spelled as the key's plain hyphen.
fn key_name(key: Key) -> &'static str {
    match key {
        Key::Minus => "-",
        key => key.symbol_or_name(),
    }
}

struct KeysText(Keys);

impl fmt::Display for KeysText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Keys::One(key) => f.write_str(key_name(key)),
            Keys::Either(a, b) => write!(f, "{} / {}", key_name(a), key_name(b)),
            Keys::Arrows => f.write_str(tr(K::Arrows)),
        }
    }
}

/// `shortcut`'s keys, with macOS (`mac`) or other modifier names.
pub fn write_keys<'a>(buf: &'a mut KeysBuf, shortcut: &Shortcut, mac: bool) -> &'a str {
    let mods = ModifierText {
        mods: shortcut.mods,
        mac,
    };
    stack_str(buf, format_args!("{mods}{}", KeysText(shortcut.keys)))
}

/// Whether modifier names should read as on macOS.
pub fn is_mac(ctx: &egui::Context) -> bool {
    ctx.os() == egui::os::OperatingSystem::Mac
}
