//! The shortcut list: `?` opens it and Escape closes it, its keys read per
//! platform, and it keeps its width in every language.

use super::keymap::{Action, CanvasAction, shortcut_of};
use super::labels::{KeysBuf, write_keys};
use super::{dispatch, help};
use crate::app::OctantApp;
use crate::i18n::{Language, with_language};
use crate::ui::panel_layout::Panel;
use crate::ui::panel_width_tests::assert_window_width;
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Rect, Ui, pos2};

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
    assert_eq!(keys(Action::SearchVariables, true), "Cmd+K");
    assert_eq!(keys(Action::SearchVariables, false), "Ctrl+K");
    assert_eq!(keys(Action::GoToPanel(Panel::Dataset), false), "D");
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
fn the_list_keeps_its_width_in_every_language() {
    let make_app = || {
        let mut app = OctantApp::default();
        app.layout.show_shortcuts = true;
        app
    };
    assert_window_width(
        "keyboard_shortcuts_window",
        help::WIDTH,
        make_app,
        help::show,
    );
}
