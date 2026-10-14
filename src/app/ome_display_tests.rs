//! OME-Zarr rendering defaults: default plane and time point, greyscale
//! images and channel display settings.

use serde_json::json;

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::VariableInfo;
use crate::data::ome::ChannelFamily;
use crate::data::ome::test_support::Fixture;

/// Level 0 of a `(t, c, z, y, x)` image with two channels.
fn image(model: &str) -> VariableInfo {
    let axes = json!([
        { "name": "t", "type": "time", "unit": "second" },
        { "name": "c", "type": "channel" },
        { "name": "z", "type": "space", "unit": "micrometer" },
        { "name": "y", "type": "space", "unit": "micrometer" },
        { "name": "x", "type": "space", "unit": "micrometer" }
    ]);
    let window =
        |start: f32, end: f32| json!({ "min": 0, "max": 4095, "start": start, "end": end });
    let attrs = json!({
        "multiscales": [{ "version": "0.4", "axes": axes, "datasets": [{ "path": "0",
            "coordinateTransformations": [{ "type": "scale", "scale": [2.0, 1.0, 0.5, 0.25, 0.25] }] }] }],
        "omero": {
            "rdefs": { "defaultT": 7, "defaultZ": 3, "model": model },
            "channels": [
                { "label": "off", "color": "FF0000", "active": false, "window": window(0.0, 100.0) },
                { "label": "on", "color": "00FF00", "active": true, "inverted": true,
                  "family": "exponential", "coefficient": 2.5, "window": window(10.0, 900.0) }
            ]
        }
    });
    let found = Fixture::default()
        .v2_group("", attrs)
        .v2_array("0", &[10, 2, 6, 8, 8])
        .discover()
        .expect("ome");
    found.variables[0].clone()
}

fn defaults_for(var: &VariableInfo) -> OctantApp {
    let mut app = OctantApp::default();
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, var);
    app
}

#[test]
fn default_time_point_and_plane_are_selected() {
    let var = image("color");
    let app = defaults_for(&var);
    assert_eq!(app.selected.animated_dim, Some(0));
    assert_eq!(app.selected.dim_indices[0], 7);
    let (start, end) = app.selected.dim_ranges[0];
    assert!(start <= 7 && 7 <= end);
    assert_eq!(app.playback.current_timestep, 7);
    assert_eq!(app.selected.dim_indices[2], 3);
    assert_eq!(app.selected.dim_ranges[2], (3, 3));
    // A colour image keeps every channel.
    assert_eq!(app.selected.dim_ranges[1], (0, 1));
}

#[test]
fn colour_images_overlay_their_channels_with_omero_settings() {
    let var = image("color");
    let mut app = defaults_for(&var);
    crate::ui::variables_panel::dimension_slider::init_layer_composite_defaults(
        &mut app,
        LayerId::BASE,
        &var,
    );
    let composite = &app.layers.base.composite;
    assert!(composite.enabled);
    let on = &composite.channel_configs[1];
    assert_eq!(
        (on.name.as_str(), on.color_rgb, on.visible),
        ("on", [0, 255, 0], true)
    );
    assert_eq!(on.window, Some((10.0, 900.0)));
    assert_eq!(on.display.range, Some((0.0, 4095.0)));
    assert!(on.display.inverted);
    assert_eq!(on.display.family, ChannelFamily::Exponential);
    assert_eq!(on.display.coefficient, 2.5);
    assert!(!composite.channel_configs[0].visible);
}

#[test]
fn greyscale_images_show_their_first_active_channel() {
    let var = image("greyscale");
    let mut app = defaults_for(&var);
    assert_eq!(app.selected.dim_ranges[1], (1, 1));
    assert_eq!(app.selected.dim_indices[1], 1);
    crate::ui::variables_panel::dimension_slider::init_layer_composite_defaults(
        &mut app,
        LayerId::BASE,
        &var,
    );
    let composite = &app.layers.base.composite;
    assert!(!composite.enabled);
    assert_eq!(composite.channel_configs.len(), 2);
    assert!(crate::utils::colormap::registry::find("classic:gray").is_some());
}

#[test]
fn physical_units_reach_the_dimensions() {
    let var = image("color");
    assert_eq!(var.dimension_unit("x"), Some("µm"));
    assert_eq!(var.dimension_unit("t"), Some("s"));
    assert_eq!(var.dimension_unit("c"), None);
}

#[test]
fn greyscale_window_follows_the_shown_channel() {
    let var = image("greyscale");
    let mut app = defaults_for(&var);
    let indices = app.selected.dim_indices.clone();
    let sel = app.layers.base.selection_mut();
    sel.metadata = Some(crate::data::DatasetMetadata {
        variables: vec![var],
        ..Default::default()
    });
    sel.dim_indices = indices;
    let apply = |app: &mut OctantApp| {
        crate::app::pipeline::greyscale::apply_greyscale_display(&mut app.layers.base, (0.0, 1.0));
        let c = &app.layers.base.color;
        (c.range_min, c.range_max, c.reversed, c.lock_bounds)
    };
    assert_eq!(apply(&mut app), (10.0, 900.0, true, true));
    app.layers.base.selection_mut().dim_indices[1] = 0;
    assert_eq!(apply(&mut app), (0.0, 100.0, false, true));
}
