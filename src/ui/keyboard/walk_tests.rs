//! The whole window by keyboard, over a plotted, animated variable with
//! every panel open: Tab walks every area and wraps, F6 visits the regions
//! in visual order, Escape in each panel returns focus to its toolbar
//! button, and single keys stay out of text fields.

use super::focus;
use super::regions::Region;
use super::test_support::{press, run, window};
use crate::app::layers::LayerId;
use crate::ui::panel_layout::Panel;
use crate::ui::test_input::Harness;
use egui::{Id, Key, Modifiers};

const NONE: Modifiers = Modifiers::NONE;

/// Every region, in visual order.
const RING: [Region; 8] = [
    Region::TopBar,
    Region::Dataset,
    Region::Variables,
    Region::Settings,
    Region::Dimensions,
    Region::Canvas,
    Region::Colorbars,
    Region::BottomBar,
];

/// Which region the focused widget sits in.
fn region_of(h: &Harness, f: focus::LastFocus) -> Region {
    let panel = Panel::ALL.into_iter().find(|p| p.area_id() == f.layer.id);
    let colorbar = Id::unique(("octant_colorbar_overlay", LayerId::BASE));
    let canvas = h.ctx.read_response(crate::app::canvas::canvas_id());
    match panel {
        Some(panel) => Region::of(panel),
        None if f.layer.id == colorbar => Region::Colorbars,
        None if f.id == crate::app::canvas::canvas_id() => Region::Canvas,
        None if canvas.is_some_and(|c| f.rect.top() >= c.rect.bottom()) => Region::BottomBar,
        None => Region::TopBar,
    }
}

#[test]
fn tab_walks_every_area_and_wraps() {
    let (mut app, mut h) = window(true);
    let first = press(&mut h, &mut app, Key::Tab, NONE).expect("a first stop");
    let mut regions = vec![region_of(&h, first)];
    let (mut top_bar_stops, mut wrapped) = (1, false);
    for _ in 0..200 {
        let at = press(&mut h, &mut app, Key::Tab, NONE).expect("a stop");
        if at.id == first.id {
            wrapped = true;
            break;
        }
        let region = region_of(&h, at);
        top_bar_stops += usize::from(region == Region::TopBar);
        if !regions.contains(&region) {
            regions.push(region);
        }
    }
    assert!(wrapped, "Tab comes back around");
    assert!(
        RING.iter().all(|r| regions.contains(r)),
        "Tab reaches every region: {regions:?}"
    );
    assert!(top_bar_stops >= 10, "every top-bar item: {top_bar_stops}");
}

#[test]
fn f6_visits_the_regions_in_visual_order() {
    let (mut app, mut h) = window(true);
    // F6 starts after the region holding focus (the Variables panel focuses
    // its search when it opens), then follows the ring.
    let mut f6 = |mods| {
        let at = press(&mut h, &mut app, Key::F6, mods).expect("a region");
        region_of(&h, at)
    };
    let visited: Vec<Region> = (0..=RING.len()).map(|_| f6(NONE)).collect();
    let start = RING
        .iter()
        .position(|&r| r == visited[0])
        .expect("a region");
    let expected: Vec<Region> = (0..visited.len())
        .map(|i| RING[(start + i) % RING.len()])
        .collect();
    assert_eq!(visited, expected);
    assert_eq!(
        f6(Modifiers::SHIFT),
        expected[expected.len() - 2],
        "Shift+F6 goes back"
    );
}

#[test]
fn escape_in_each_panel_returns_to_its_button() {
    // Places of the panel buttons after the logo and the brand label.
    let buttons = [
        (Panel::Dataset, 2),
        (Panel::Variables, 3),
        (Panel::Dimensions, 4),
        (Panel::Settings, 6),
    ];
    for (panel, place) in buttons {
        let (mut app, mut h) = window(false);
        press(&mut h, &mut app, Key::F10, NONE);
        let mut button = press(&mut h, &mut app, Key::Home, NONE);
        for _ in 0..place {
            button = press(&mut h, &mut app, Key::ArrowRight, NONE);
        }
        press(&mut h, &mut app, Key::Enter, NONE);
        assert!(
            h.focus_in(panel.area_id()),
            "Enter opens {panel:?} with focus inside"
        );
        let back = press(&mut h, &mut app, Key::Escape, NONE);
        assert!(!app.layout.panel_shown(panel), "Escape closes {panel:?}");
        assert_eq!(
            back.map(|f| f.id),
            button.map(|f| f.id),
            "focus returns from {panel:?}"
        );
    }
}

#[test]
fn single_keys_stay_out_of_text_fields() {
    let (mut app, mut h) = window(false);
    // Cmd+K opens the variable search and focuses its field.
    press(&mut h, &mut app, Key::K, Modifiers::COMMAND);
    for (key, text) in [
        (Key::C, "c"),
        (Key::Space, " "),
        (Key::D, "d"),
        (Key::S, "s"),
    ] {
        let typed = egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: NONE,
        };
        h.frame(
            vec![typed, egui::Event::Text(text.into())],
            &mut run(&mut app),
        );
    }
    assert_eq!(app.layout.variable_search, "c ds");
    assert!(!app.show_crop_overlay && !app.playback.is_playing);
    assert!(!app.layout.show_dataset_panel && !app.layout.show_settings_panel);
}
