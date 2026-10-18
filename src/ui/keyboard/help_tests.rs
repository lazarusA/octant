//! The shortcut list: `?` opens it and Escape closes it, its keys read per
//! platform, and it keeps its width in every language.

use super::keymap::{Action, CanvasAction, TABLE, shortcut_of};
use super::labels::{KeysBuf, write_keys};
use super::{dispatch, help};
use crate::app::OctantApp;
use crate::i18n::{Language, with_language};
use crate::ui::panel_layout::Panel;
use crate::ui::panel_width_tests::{SLACK, area_width_of};
use crate::ui::test_input::Harness;
use egui::{Id, Key, Modifiers, Rect, Ui, pos2};

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        dispatch(app, ui.ctx(), false);
        help::show(app, ui.ctx());
    }
}

#[test]
fn question_mark_opens_the_list_and_escape_closes_it() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 800.0)));
    h.settle(1, &mut run(&mut app));
    h.key(Key::Questionmark, Modifiers::SHIFT, &mut run(&mut app));
    assert!(app.layout.show_shortcuts);
    h.settle(2, &mut run(&mut app));
    h.key(Key::Escape, Modifiers::NONE, &mut run(&mut app));
    assert!(!app.layout.show_shortcuts);
}

#[test]
fn keys_read_with_each_platforms_modifier_names() {
    let keys = |action, mac| {
        let mut buf: KeysBuf = [0; 48];
        let shortcut = shortcut_of(action).expect("bound");
        write_keys(&mut buf, shortcut, mac).to_owned()
    };
    let dataset = Action::TogglePanel(Panel::Dataset);
    assert_eq!(keys(dataset, true), "Cmd+1");
    assert_eq!(keys(dataset, false), "Ctrl+1");
    assert_eq!(keys(Action::ExportDialog, false), "Ctrl+Shift+S");
    assert_eq!(keys(Action::Canvas(CanvasAction::ZoomIn), false), "+ / =");
    with_language(Language::English, || {
        assert_eq!(
            keys(Action::Canvas(CanvasAction::MoveAlt), false),
            "Shift+Arrows"
        );
    });
}

#[test]
fn every_shortcut_is_listed_once() {
    for (i, a) in TABLE.iter().enumerate() {
        for b in &TABLE[i + 1..] {
            let same_keys = a.keys == b.keys && a.mods == b.mods;
            assert!(
                !(same_keys && a.scope == b.scope),
                "{:?} and {:?} share their keys",
                a.action,
                b.action
            );
        }
    }
}

#[test]
fn the_list_keeps_its_width_in_every_language() {
    let frame = egui::Frame::window(&egui::Style::default())
        .total_margin()
        .sum()
        .x;
    let limit = help::WIDTH + frame + SLACK;
    let mut wide = Vec::new();
    for &lang in Language::ALL {
        let mut app = OctantApp::default();
        app.layout.show_shortcuts = true;
        let mut run = |ui: &mut Ui| help::show(&mut app, ui.ctx());
        let id = Id::unique("keyboard_shortcuts_window");
        let width = area_width_of(lang, id, &[], &mut run);
        if width > limit {
            wide.push((lang, width));
        }
    }
    assert!(wide.is_empty(), "wider than {limit}: {wide:?}");
}
