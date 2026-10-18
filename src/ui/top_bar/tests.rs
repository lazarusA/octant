use super::layout::{COLLAPSE_ORDER, ITEM_COUNT, TopBarItem};

#[test]
fn test_collapse_order_lists_every_item_once() {
    let mut seen = [false; ITEM_COUNT];
    for item in COLLAPSE_ORDER {
        assert!(!seen[item as usize], "{item:?} listed twice");
        seen[item as usize] = true;
    }
    assert!(seen.iter().all(|&s| s));
}

#[test]
fn test_brand_collapses_last() {
    assert_eq!(COLLAPSE_ORDER.last(), Some(&TopBarItem::Brand));
}

mod keys {
    use crate::app::OctantApp;
    use crate::ui::keyboard::{self, focus};
    use crate::ui::test_input::Harness;
    use egui::{Key, Modifiers, Rect, Ui, pos2};

    const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1400.0, 300.0));

    fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
        |ui| {
            keyboard::dispatch(app, ui.ctx(), false);
            super::super::show_top_bar(app, ui);
            focus::end_frame(ui.ctx());
        }
    }

    fn focused_left(h: &Harness) -> f32 {
        focus::last(&h.ctx)
            .expect("a widget is focused")
            .rect
            .left()
    }

    fn press(h: &mut Harness, app: &mut OctantApp, key: Key) {
        h.key(key, Modifiers::NONE, &mut run(app));
        h.settle(1, &mut run(app));
    }

    #[test]
    fn arrows_walk_the_bar_in_visual_order() {
        let mut app = OctantApp::default();
        let mut h = Harness::new(SCREEN);
        h.settle(2, &mut run(&mut app));
        press(&mut h, &mut app, Key::F10);
        let mut lefts = vec![focused_left(&h)];
        loop {
            press(&mut h, &mut app, Key::ArrowRight);
            let left = focused_left(&h);
            if left == *lefts.last().expect("started") {
                break;
            }
            lefts.push(left);
        }
        assert!(lefts.len() >= 8, "every item is reached: {lefts:?}");
        assert!(lefts.windows(2).all(|w| w[0] < w[1]), "{lefts:?}");

        press(&mut h, &mut app, Key::Home);
        assert_eq!(focused_left(&h), lefts[0]);
        press(&mut h, &mut app, Key::End);
        assert_eq!(focused_left(&h), *lefts.last().expect("items"));
        press(&mut h, &mut app, Key::ArrowLeft);
        assert_eq!(focused_left(&h), lefts[lefts.len() - 2]);
    }

    #[test]
    fn down_opens_a_menu_and_enters_it() {
        let mut app = OctantApp::default();
        let mut h = Harness::new(SCREEN);
        h.settle(2, &mut run(&mut app));
        press(&mut h, &mut app, Key::F10);
        // The language menu sits left of the theme toggle, the last item.
        press(&mut h, &mut app, Key::End);
        press(&mut h, &mut app, Key::ArrowLeft);
        let button = focus::last(&h.ctx).expect("the language button");
        press(&mut h, &mut app, Key::ArrowDown);
        h.settle(1, &mut run(&mut app));
        assert!(egui::Popup::is_any_open(&h.ctx));
        let inside = focus::last(&h.ctx).expect("focus moved into the menu");
        assert_ne!(inside.layer, button.layer, "focus is in the popup");
    }

    #[test]
    fn escape_leaves_the_bar() {
        let mut app = OctantApp::default();
        let mut h = Harness::new(SCREEN);
        h.settle(2, &mut run(&mut app));
        press(&mut h, &mut app, Key::F10);
        assert!(focus::last(&h.ctx).is_some());
        press(&mut h, &mut app, Key::Escape);
        assert_eq!(focus::last(&h.ctx), None);
    }
}
