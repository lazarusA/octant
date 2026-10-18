//! The whole window by keyboard, over a plotted, animated variable with
//! every panel open: Tab walks every area and wraps, F6 visits the regions
//! in visual order, Escape in each panel returns focus to its toolbar
//! button, and single keys stay out of text fields.

use super::regions::Region;
use super::{dispatch, focus, regions};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::plots::PlotType;
use crate::ui::panel_layout::Panel;
use crate::ui::panel_width_tests::plotted;
use crate::ui::test_input::Harness;
use egui::{Id, Key, Modifiers, Rect, Ui, pos2};

pub(super) const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1600.0, 1000.0));

/// One frame drawn in the app's order, with a stand-in for the canvas
/// (painting it needs a GPU).
pub(super) fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        let ctx = ui.ctx().clone();
        dispatch(app, &ctx, true);
        crate::ui::top_bar::show_top_bar(app, ui);
        crate::ui::bottom_bar::show_bottom_bar(app, ui);
        let canvas = ui.available_rect_before_wrap();
        crate::ui::dataset::show_dataset_window(app, &ctx, canvas);
        crate::ui::variables_overlay::show_variables_overlay(app, &ctx, canvas);
        crate::ui::settings::show_settings_window(app, &ctx, canvas);
        crate::ui::variables_panel::show_variable_controls(app, &ctx, canvas);
        crate::ui::colorbar::show_colorbar_overlay(app, &ctx, canvas);
        let id = crate::app::canvas::canvas_id();
        let response = ui.interact(canvas, id, egui::Sense::drag());
        regions::register(&ctx, Region::Canvas, id, response.layer_id, canvas);
        focus::end_frame(&ctx);
    }
}

/// A plotted heatmap of `t2m`, animated along its time steps.
pub(super) fn app(panels_open: bool) -> OctantApp {
    let mut app = plotted(PlotType::Heatmap);
    for panel in Panel::ALL {
        *app.layout.panel_shown_mut(panel) = panels_open;
    }
    app
}

fn press(h: &mut Harness, app: &mut OctantApp, key: Key, modifiers: Modifiers) -> focus::LastFocus {
    h.key(key, modifiers, &mut run(app));
    h.settle(2, &mut run(app));
    focus::last(&h.ctx).expect("a widget holds focus")
}

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
    let mut app = app(true);
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    let first = press(&mut h, &mut app, Key::Tab, Modifiers::NONE);
    let mut regions = vec![region_of(&h, first)];
    let mut top_bar_stops = 1;
    let mut wrapped = false;
    for _ in 0..600 {
        let at = press(&mut h, &mut app, Key::Tab, Modifiers::NONE);
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
    for region in [
        Region::TopBar,
        Region::Dataset,
        Region::Variables,
        Region::Settings,
        Region::Dimensions,
        Region::Canvas,
        Region::Colorbars,
        Region::BottomBar,
    ] {
        assert!(
            regions.contains(&region),
            "Tab reaches {region:?}: {regions:?}"
        );
    }
    assert!(top_bar_stops >= 10, "every top-bar item: {top_bar_stops}");
}

#[test]
fn f6_visits_the_regions_in_visual_order() {
    let mut app = app(true);
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    // Visual order, top to bottom; F6 starts after the region holding
    // focus (the Variables panel focuses its search when it opens).
    let ring = [
        Region::TopBar,
        Region::Dataset,
        Region::Variables,
        Region::Settings,
        Region::Dimensions,
        Region::Canvas,
        Region::Colorbars,
        Region::BottomBar,
    ];
    let visited: Vec<Region> = (0..ring.len() + 1)
        .map(|_| {
            let at = press(&mut h, &mut app, Key::F6, Modifiers::NONE);
            region_of(&h, at)
        })
        .collect();
    let start = ring
        .iter()
        .position(|&r| r == visited[0])
        .expect("a region");
    let expected: Vec<Region> = (0..visited.len())
        .map(|i| ring[(start + i) % ring.len()])
        .collect();
    assert_eq!(visited, expected);
    // Back from the last region visited, one before it in the ring.
    let before = expected[expected.len() - 2];
    let back = press(&mut h, &mut app, Key::F6, Modifiers::SHIFT);
    assert_eq!(region_of(&h, back), before, "Shift+F6 goes back");
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
        let mut app = app(false);
        let mut h = Harness::new(SCREEN);
        h.settle(4, &mut run(&mut app));
        press(&mut h, &mut app, Key::F10, Modifiers::NONE);
        let mut button = press(&mut h, &mut app, Key::Home, Modifiers::NONE);
        for _ in 0..place {
            button = press(&mut h, &mut app, Key::ArrowRight, Modifiers::NONE);
        }
        let inside = press(&mut h, &mut app, Key::Enter, Modifiers::NONE);
        assert!(app.layout.panel_shown(panel), "{panel:?} opens");
        assert_eq!(inside.layer.id, panel.area_id(), "focus enters {panel:?}");
        let back = press(&mut h, &mut app, Key::Escape, Modifiers::NONE);
        assert!(!app.layout.panel_shown(panel), "Escape closes {panel:?}");
        assert_eq!(back.id, button.id, "focus returns from {panel:?}");
    }
}

#[test]
fn single_keys_stay_out_of_text_fields() {
    let mut app = app(false);
    let mut h = Harness::new(SCREEN);
    h.settle(4, &mut run(&mut app));
    // Cmd+K opens the variable search and focuses its field.
    press(&mut h, &mut app, Key::K, Modifiers::COMMAND);
    for (key, text) in [(Key::C, "c"), (Key::Space, " "), (Key::L, "l")] {
        let typed = vec![
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            },
            egui::Event::Text(text.into()),
        ];
        h.frame(typed, &mut run(&mut app));
    }
    assert_eq!(app.layout.variable_search, "c l");
    assert!(!app.show_crop_overlay);
    assert!(!app.playback.is_playing);
}
