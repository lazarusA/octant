use super::layout::{COLLAPSE_ORDER, ITEM_COUNT, TopBarItem, overflows};
use crate::ui::toolbar::ItemWidths;

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

#[test]
fn test_overflows_only_when_the_icons_alone_do_not_fit() {
    let w = ItemWidths {
        full: 100.0,
        compact: 30.0,
    };
    let widths = [w; ITEM_COUNT];
    // Every item but Status, which the check leaves out.
    let icons = 30.0 * (ITEM_COUNT - 1) as f32;
    assert!(!overflows(&widths, icons));
    assert!(overflows(&widths, icons - 1.0));
    // A fetch's status never moves the items into the More menu.
    let mut fetching = widths;
    fetching[TopBarItem::Status as usize].compact = 200.0;
    assert!(!overflows(&fetching, icons));
}

mod keys {
    use crate::app::OctantApp;
    use crate::ui::keyboard::{self, focus};
    use crate::ui::test_input::Harness;
    use egui::{Key, Modifiers, Rect, Ui, pos2};

    const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1400.0, 300.0));
    /// A phone held upright.
    const PHONE: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(360.0, 640.0));

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

    #[test]
    fn t_hides_the_bar_and_f10_shows_it_with_focus_inside() {
        let mut app = OctantApp::default();
        let mut h = Harness::new(SCREEN);
        h.settle(2, &mut run(&mut app));
        press(&mut h, &mut app, Key::T);
        assert!(!app.layout.show_top_bar);
        let focused = press(&mut h, &mut app, Key::F10);
        assert!(app.layout.show_top_bar);
        assert!(focused.is_some(), "focus enters the bar shown again");
    }

    #[test]
    fn clicking_the_strip_shows_the_bar() {
        let mut app = OctantApp::default();
        app.layout.show_top_bar = false;
        let mut h = Harness::new(SCREEN);
        h.settle(2, &mut run(&mut app));
        h.click(pos2(700.0, 7.0), &mut run(&mut app));
        assert!(app.layout.show_top_bar);
    }

    /// A one-value block of `variable` in the block cache, so the Cache
    /// menu lists it.
    fn cache_variable(app: &mut OctantApp, variable: &str) {
        use crate::data::blocks::BlockCacheKey;
        use crate::data::octant_block::OctantBlock;
        use crate::data::slice_request::{DimensionSelection, SliceRequest};
        let slice = SliceRequest::new(variable, vec![DimensionSelection::range(0, 1)]);
        let block = OctantBlock::new(
            variable.to_string(),
            vec![1],
            vec!["x".into()],
            vec![0],
            vec![0.0],
            Default::default(),
            std::collections::HashMap::new(),
        )
        .expect("block");
        app.block_cache
            .put(BlockCacheKey::new("dataset", &slice), block);
    }

    #[test]
    fn a_phone_wide_bar_puts_the_right_items_in_the_more_menu() {
        let mut app = OctantApp::default();
        cache_variable(&mut app, "10m_u_component_of_wind_at_100_metres");
        let mut h = Harness::new(PHONE);
        h.settle(2, &mut run(&mut app));
        press(&mut h, &mut app, Key::F10);
        // The last item: the More menu, not the theme toggle.
        press(&mut h, &mut app, Key::End);
        let more = focus::last(&h.ctx).expect("the More button").rect;
        h.press(Key::ArrowDown, Modifiers::NONE, &mut run(&mut app));
        assert!(egui::Popup::is_any_open(&h.ctx));
        let cache_row = pos2(more.left() - 40.0, more.bottom() + 16.0);
        let menu_width = |h: &Harness| h.area(h.top_at(cache_row).expect("the menu")).width();
        let closed = menu_width(&h);

        // Clicking its Cache section, the first row, keeps the menu open,
        // widened to at most 80 % of the window.
        h.click(cache_row, &mut run(&mut app));
        assert!(egui::Popup::is_any_open(&h.ctx));
        h.settle(20, &mut run(&mut app));
        let open = menu_width(&h);
        let height = |h: &Harness| h.area(h.top_at(cache_row).expect("the menu")).height();
        assert!(height(&h) > 300.0, "grows to fit the cache: {}", height(&h));
        assert!(height(&h) <= PHONE.height() * 0.8 + 0.5);
        assert!(open > closed, "{open} > {closed}");
        assert!(open <= PHONE.width() * 0.8 + 0.5, "{open}");

        // Closing it narrows the menu again, once the section has folded.
        h.click(cache_row, &mut run(&mut app));
        h.settle(20, &mut run(&mut app));
        assert!(egui::Popup::is_any_open(&h.ctx));
        assert_eq!(menu_width(&h), closed);
    }

    #[test]
    fn dragging_the_lower_edge_up_hides_the_bar_and_down_shows_it() {
        let mut app = OctantApp::default();
        let mut h = Harness::new(SCREEN);
        h.settle(2, &mut run(&mut app));
        h.drag(
            pos2(700.0, 34.0),
            egui::vec2(0.0, -30.0),
            &mut run(&mut app),
        );
        assert!(!app.layout.show_top_bar);
        h.settle(10, &mut run(&mut app));
        h.drag(pos2(700.0, 10.0), egui::vec2(0.0, 30.0), &mut run(&mut app));
        assert!(app.layout.show_top_bar);
    }

    #[test]
    fn t_keeps_focus_in_the_bar_or_its_strip() {
        let (mut app, mut h) = entered();
        let on_strip = press(&mut h, &mut app, Key::T);
        assert!(!app.layout.show_top_bar);
        assert!(on_strip.is_some(), "focus moves to the strip");
        let in_bar = press(&mut h, &mut app, Key::T);
        assert!(app.layout.show_top_bar);
        assert!(in_bar.is_some(), "focus moves into the bar");
    }

    #[test]
    fn the_shown_bars_edge_cursor_points_up_and_the_strips_down() {
        // The cursor the bar sets as it draws, read before the frame hands
        // its output on.
        let edge_cursor = |shown: bool, edge_y: f32| {
            let mut app = OctantApp::default();
            app.layout.show_top_bar = shown;
            let mut cursor = egui::CursorIcon::Default;
            let mut run = |ui: &mut Ui| {
                super::super::show_top_bar(&mut app, ui);
                cursor = ui.ctx().output(|o| o.cursor_icon);
            };
            let mut h = Harness::new(SCREEN);
            h.settle(2, &mut run);
            h.pointer(pos2(700.0, edge_y), None, &mut run);
            h.settle(1, &mut run);
            cursor
        };
        assert_eq!(edge_cursor(true, 34.0), egui::CursorIcon::ResizeNorth);
        assert_eq!(edge_cursor(false, 20.0), egui::CursorIcon::ResizeSouth);
    }

    #[test]
    fn the_strips_tooltip_is_centered_wherever_it_is_hovered() {
        let mut app = OctantApp::default();
        app.layout.show_top_bar = false;
        let mut h = Harness::new(SCREEN);
        h.settle(2, &mut run(&mut app));
        h.pointer(pos2(100.0, 10.0), None, &mut run(&mut app));
        h.settle(60, &mut run(&mut app));
        let below_chevron = pos2(SCREEN.center().x, 32.0);
        let layer = h.ctx.layer_id_at(below_chevron).expect("a layer");
        assert_eq!(layer.order, egui::Order::Tooltip, "the tooltip is there");
        let tooltip = h.area(layer.id);
        assert!(
            (tooltip.center().x - SCREEN.center().x).abs() < 1.0,
            "{tooltip:?}"
        );
    }
}
