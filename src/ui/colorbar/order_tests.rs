//! Colorbars sit in the middle order: the floating panels and the catalog's
//! backdrop cover them, even after a colorbar was clicked (which raises it
//! within its order).

use super::show_colorbar_overlay;
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::ui::settings::show_settings_window;
use crate::ui::test_input::Harness;
use egui::{Id, Rect, Ui, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1240.0, 930.0));

fn colorbar_id() -> Id {
    Id::new(("octant_colorbar_overlay", LayerId::BASE))
}

/// One frame of the colorbars under the Settings panel.
fn with_settings(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        show_colorbar_overlay(app, ui.ctx(), SCREEN);
        show_settings_window(app, ui.ctx(), SCREEN);
    }
}

/// One frame of the colorbars and the catalog.
fn with_catalog(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        show_colorbar_overlay(app, ui.ctx(), SCREEN);
        crate::ui::catalog::show_catalog_window(app, ui.ctx());
    }
}

#[test]
fn panels_draw_over_colorbars_even_after_a_colorbar_click() {
    let mut app = OctantApp::default();
    app.layout.show_settings_panel = true;
    app.layout.show_colorbar = true;
    let at = crate::ui::drag_grip::to_fraction(pos2(300.0, 200.0), SCREEN);
    app.layers.base.colorbar.pos = Some(at);
    let mut h = Harness::new(SCREEN);
    h.settle(3, &mut with_settings(&mut app));
    let colorbar = h.area(colorbar_id());
    let settings = h.area(Id::new("octant_settings_area"));
    let overlap = colorbar.intersect(settings);
    assert!(overlap.is_positive(), "{colorbar:?} vs {settings:?}");

    let only_bar = pos2(colorbar.right() - 30.0, colorbar.center().y);
    assert!(!settings.contains(only_bar));
    h.click(only_bar, &mut with_settings(&mut app));
    assert_eq!(
        h.top_at(overlap.center()),
        Some(Id::new("octant_settings_area"))
    );
}

#[test]
fn the_catalog_backdrop_covers_colorbars() {
    let mut app = OctantApp::default();
    app.layout.show_colorbar = true;
    let mut h = Harness::new(SCREEN);
    h.settle(3, &mut with_catalog(&mut app));
    let at = h.area(colorbar_id()).center();
    // Open and close the catalog, then click the colorbar, raising it.
    app.layout.show_catalog_window = true;
    h.settle(1, &mut with_catalog(&mut app));
    app.layout.show_catalog_window = false;
    h.settle(2, &mut with_catalog(&mut app));
    h.click(at, &mut with_catalog(&mut app));
    app.layout.show_catalog_window = true;
    h.settle(3, &mut with_catalog(&mut app));
    assert_eq!(h.top_at(at), Some(Id::new("catalog_modal_backdrop")));
}
