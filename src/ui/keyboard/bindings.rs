//! Every keyboard shortcut in one table: which keys do what, where, and how
//! the shortcut list describes it.

use super::keymap::Action::{self, *};
use super::keymap::CanvasAction::*;
use super::keymap::Keys::{self, *};
use super::keymap::{Scope, Shortcut};
use crate::i18n::KeymapKey as K;
use crate::ui::panel_layout::Panel;
use egui::{Key, Modifiers};

const NONE: Modifiers = Modifiers::NONE;
const CMD: Modifiers = Modifiers::COMMAND;
const SHIFT: Modifiers = Modifiers::SHIFT;
const CMD_SHIFT: Modifiers = CMD.plus(SHIFT);

const fn s(scope: Scope, action: Action, keys: Keys, mods: Modifiers, label: K) -> Shortcut {
    Shortcut {
        action,
        keys,
        mods,
        scope,
        label,
    }
}

#[rustfmt::skip]
pub const TABLE: &[Shortcut] = &[
    s(Scope::Global, QuickSave, One(Key::S), CMD, K::QuickSave),
    s(Scope::Global, ExportDialog, One(Key::S), CMD_SHIFT, K::ExportDialog),
    s(Scope::Global, SearchVariables, One(Key::K), CMD, K::SearchVariables),
    s(Scope::Global, PlotStaged, One(Key::Enter), CMD, K::PlotStaged),
    s(Scope::Global, NextRegion, One(Key::F6), NONE, K::NextRegion),
    s(Scope::Global, PreviousRegion, One(Key::F6), SHIFT, K::PreviousRegion),
    s(Scope::Global, FocusTopBar, One(Key::F10), NONE, K::FocusTopBar),
    s(Scope::Panel, ClosePanel, One(Key::Escape), NONE, K::ClosePanel),
    s(Scope::Free, ShowShortcuts, One(Key::Questionmark), NONE, K::ShowShortcuts),
    s(Scope::Free, ToggleHero, One(Key::H), NONE, K::ToggleHero),
    s(Scope::Free, PlotTypePicker, One(Key::P), NONE, K::PlotTypePicker),
    s(Scope::Free, GoToPanel(Panel::Dataset), One(Key::D), NONE, K::GoToDataset),
    s(Scope::Free, GoToPanel(Panel::Variables), One(Key::V), NONE, K::GoToVariables),
    s(Scope::Free, GoToPanel(Panel::Dimensions), One(Key::A), NONE, K::GoToDimensions),
    s(Scope::Free, GoToPanel(Panel::Settings), One(Key::S), NONE, K::GoToSettings),
    s(Scope::Free, GoToColorbar, One(Key::B), NONE, K::GoToColorbar),
    s(Scope::Hero, LeaveHero, One(Key::Escape), NONE, K::LeaveHero),
    s(Scope::Canvas, ToggleCrop, One(Key::C), NONE, K::ToggleCrop),
    s(Scope::Canvas, Canvas(Move), Arrows, NONE, K::Move),
    s(Scope::Canvas, Canvas(MoveAlt), Arrows, SHIFT, K::MoveAlt),
    s(Scope::Canvas, Canvas(ZoomIn), Either(Key::Plus, Key::Equals), NONE, K::ZoomIn),
    s(Scope::Canvas, Canvas(ZoomOut), One(Key::Minus), NONE, K::ZoomOut),
    s(Scope::Canvas, Canvas(ResetView), One(Key::Num0), NONE, K::ResetView),
    s(Scope::Canvas, Canvas(ToggleProbe), One(Key::Enter), NONE, K::ToggleProbe),
    s(Scope::Canvas, Canvas(LeaveProbe), One(Key::Escape), NONE, K::LeaveProbe),
    s(Scope::Playback, PlayPause, One(Key::Space), NONE, K::PlayPause),
    s(Scope::Playback, StepPrev, One(Key::Comma), NONE, K::StepPrev),
    s(Scope::Playback, StepNext, One(Key::Period), NONE, K::StepNext),
    s(Scope::Playback, FirstStep, One(Key::Home), NONE, K::FirstStep),
    s(Scope::Playback, LastStep, One(Key::End), NONE, K::LastStep),
    s(Scope::Playback, ToggleLoop, One(Key::L), NONE, K::ToggleLoop),
];
