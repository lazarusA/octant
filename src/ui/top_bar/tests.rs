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

    /// The bar drawn and settled, with `F10` pressed to enter it.
    fn entered() -> (OctantApp, Harness) {
        let mut app = OctantApp::default();
        let mut h = Harness::new(SCREEN);
        h.settle(2, &mut run(&mut app));
        press(&mut h, &mut app, Key::F10);
        (app, h)
    }

    /// Presses `key`; the left edge of the widget focused then.
    fn press(h: &mut Harness, app: &mut OctantApp, key: Key) -> Option<f32> {
        h.press(key, Modifiers::NONE, &mut run(app))
            .map(|f| f.rect.left())
    }

    #[test]
    fn arrows_walk_the_bar_in_visual_order() {
        let (mut app, mut h) = entered();
        let mut lefts = vec![focus::last(&h.ctx).map(|f| f.rect.left())];
        for _ in 0..20 {
            let left = press(&mut h, &mut app, Key::ArrowRight);
            if lefts.last() == Some(&left) {
                break;
            }
            lefts.push(left);
        }
        assert!(
            (8..20).contains(&lefts.len()),
            "every item is reached once: {lefts:?}"
        );
        assert!(lefts.windows(2).all(|w| w[0] < w[1]), "{lefts:?}");
        let [first, .., before_last, last] = lefts[..] else {
            panic!("a bar of items");
        };
        assert_eq!(press(&mut h, &mut app, Key::Home), first);
        assert_eq!(press(&mut h, &mut app, Key::End), last);
        assert_eq!(press(&mut h, &mut app, Key::ArrowLeft), before_last);
    }

    #[test]
    fn down_opens_a_menu_and_enters_it() {
        let (mut app, mut h) = entered();
        // The language menu sits left of the theme toggle, the last item.
        press(&mut h, &mut app, Key::End);
        press(&mut h, &mut app, Key::ArrowLeft);
        let button = focus::last(&h.ctx).expect("the language button");
        let inside = h.press(Key::ArrowDown, Modifiers::NONE, &mut run(&mut app));
        assert!(egui::Popup::is_any_open(&h.ctx));
        assert_ne!(
            inside.map(|f| f.layer),
            Some(button.layer),
            "focus is in the popup"
        );
    }

    #[test]
    fn escape_leaves_the_bar() {
        // A bar item keeps Escape from egui; the bar lets focus go itself.
        let (mut app, mut h) = entered();
        assert!(focus::last(&h.ctx).is_some());
        assert_eq!(press(&mut h, &mut app, Key::Escape), None);
    }
}
