//! Composites open off, on one band, except GeoTIFFs in color; a composite
//! reads the band range like any dimension.

use crate::app::layers::LayerId;
use crate::app::{OctantApp, StoreKind};
use crate::data::{DatasetMetadata, DimensionSelection, VariableInfo};
use crate::ui::variables_panel::dimension_slider::init_composite_defaults;
use crate::ui::variables_panel::init_variable_dimension_defaults;

fn variable(dims: [&str; 3], shape: [u64; 3], attrs: &[(&str, &str)]) -> VariableInfo {
    VariableInfo {
        name: "v".into(),
        dimension_names: dims.iter().map(|d| d.to_string()).collect(),
        shape: shape.to_vec(),
        chunk_shape: shape.to_vec(),
        attributes: attrs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..Default::default()
    }
}

/// An app staging `var` from a store of `kind`, with its defaults set.
fn staged(kind: StoreKind, var: &VariableInfo) -> OctantApp {
    let mut app = OctantApp::default();
    app.selected.store_kind = kind;
    app.selected.store_target = "memory://v".into();
    app.selected.metadata = Some(DatasetMetadata {
        variables: vec![var.clone()],
        ..Default::default()
    });
    init_variable_dimension_defaults(&mut app, var);
    init_composite_defaults(&mut app, var);
    app
}

fn tiff(bands: u64, attrs: &[(&str, &str)]) -> OctantApp {
    let var = variable(["band", "y", "x"], [bands, 64, 64], attrs);
    staged(StoreKind::RemoteGeoTiff, &var)
}

#[test]
fn a_zarr_band_dimension_opens_on_one_band_with_the_composite_off() {
    let var = variable(["y", "x", "band"], [256, 256, 128], &[]);
    let app = staged(StoreKind::RemoteZarr, &var);
    assert_eq!(app.selected.dim_ranges[2], (0, 0));
    let composite = &app.layers.base.composite;
    assert!(!composite.enabled);
    assert_eq!(composite.channel_configs.len(), 128);
    let ticked: Vec<usize> = composite
        .channel_configs
        .iter()
        .filter(|c| c.visible)
        .map(|c| c.index)
        .collect();
    assert_eq!(ticked, [0]);
}

#[test]
fn a_composite_reads_only_the_selected_bands() {
    let var = variable(["y", "x", "band"], [256, 256, 128], &[]);
    let mut app = staged(StoreKind::RemoteZarr, &var);
    app.selected.dim_ranges[2] = (0, 8);
    app.selected.dim_config[2].range = (0, 8);
    let off = app.staged_layer_request(LayerId::BASE).expect("request");
    app.layers.base.composite.enabled = true;
    let on = app.staged_layer_request(LayerId::BASE).expect("request");
    assert_eq!(
        on.request.selections[2],
        DimensionSelection::Range { start: 0, end: 9 }
    );
    // Switching the composite on reads the block already cached.
    assert_eq!(on.request.selections, off.request.selections);
}

#[test]
fn only_rgb_rgba_and_cmyk_tiffs_open_in_color() {
    let rgb = tiff(3, &[]);
    assert!(rgb.layers.base.composite.enabled);
    assert_eq!(rgb.selected.dim_ranges[0], (0, 2));

    let rgba = tiff(4, &[("alpha_band", "last")]);
    assert!(rgba.layers.base.composite.enabled);
    assert_eq!(rgba.selected.dim_ranges[0], (0, 3));

    let cmyk = tiff(4, &[("photometric", "cmyk")]);
    assert!(cmyk.layers.base.composite.enabled);

    for bands in [4, 12] {
        let app = tiff(bands, &[]);
        assert!(!app.layers.base.composite.enabled, "{bands} bands");
        assert_eq!(app.selected.dim_ranges[0], (0, 0), "{bands} bands");
    }
}

#[test]
fn a_three_band_zarr_is_not_rgb() {
    let var = variable(["band", "y", "x"], [3, 64, 64], &[]);
    let app = staged(StoreKind::RemoteZarr, &var);
    assert!(!app.layers.base.composite.enabled);
    assert_eq!(app.selected.dim_ranges[0], (0, 0));
}
