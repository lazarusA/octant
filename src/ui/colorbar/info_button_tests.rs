//! The colorbar's info button, right of the cog: it opens the plotted
//! variable's attributes in a popup, keeps the controls shown while the
//! popup is open, and never overlaps the grip in either orientation.

use super::controls;
use super::controls_tests::{harness, run};
use crate::app::OctantApp;
use crate::app::layers::{BarOrientation, LayerId};
use crate::data::VariableInfo;
use crate::ui::test_input::Harness;
use egui::{Rect, pos2};
use std::collections::HashMap;

/// The base layer's colorbar panel.
fn panel(h: &Harness) -> Rect {
    h.area(egui::Id::unique(("octant_colorbar_overlay", LayerId::BASE)))
}

/// The default app, its base layer's variable given a `units` attribute.
fn app_with_attributes() -> OctantApp {
    let mut app = OctantApp::default();
    let selection = app.layers.base.selection_mut();
    let mut meta = selection.metadata.clone().unwrap_or_default();
    let attributes = HashMap::from([("units".to_string(), "kelvin".to_string())]);
    match meta.variables.get_mut(selection.variable_idx) {
        Some(var) => var.attributes = attributes,
        None => meta.variables.push(VariableInfo {
            name: "t2m".into(),
            attributes,
            ..Default::default()
        }),
    }
    selection.metadata = Some(meta);
    app
}

#[test]
fn the_info_button_opens_the_attributes_and_stays_while_open() {
    let mut app = app_with_attributes();
    let mut h = harness(&mut app);
    h.pointer(panel(&h).center(), None, &mut run(&mut app));
    let info = controls::info_rect(panel(&h));
    h.click(info.center(), &mut run(&mut app));
    let shapes = h.settle(3, &mut run(&mut app));
    assert!(egui::Popup::is_any_open(&h.ctx), "the popup opens");
    let shown = |text: &str| {
        shapes
            .iter()
            .any(|c| matches!(&c.shape, egui::Shape::Text(t) if t.galley.text() == text))
    };
    assert!(shown("kelvin"), "the attribute's value is shown");

    // Away from the panel the button stays, so a click closes the popup.
    h.pointer(pos2(5.0, 5.0), None, &mut run(&mut app));
    h.click(info.center(), &mut run(&mut app));
    assert!(
        !egui::Popup::is_any_open(&h.ctx),
        "a second click closes it"
    );
}

#[test]
fn the_info_button_never_overlaps_the_grip() {
    for orientation in [BarOrientation::Horizontal, BarOrientation::Vertical] {
        let mut app = app_with_attributes();
        app.layers.base.colorbar.orientation = orientation;
        let h = harness(&mut app);
        let panel = panel(&h);
        let [grip, _] = controls::rects(panel);
        let info = controls::info_rect(panel);
        assert!(
            info.max.x <= grip.min.x,
            "{orientation:?}: {info:?} vs {grip:?}"
        );
    }
}
