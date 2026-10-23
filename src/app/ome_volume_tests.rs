//! OME-Zarr images in 3D and their channel axis: a volume keeps its z
//! range and reads the level a volume fits on; a channel axis animates.

use serde_json::json;

use crate::app::OctantApp;
use crate::data::ome::test_support::Fixture;

fn defaults_for(var: &crate::data::VariableInfo) -> OctantApp {
    let mut app = OctantApp::default();
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, var);
    app
}

/// A `(c, z, y, x)` image of 3 levels (z scaled by `z_scales`) with channel 1
/// active and `defaultZ` 10.
fn czyx(model: &str, z_scales: [f64; 3]) -> crate::data::DatasetMetadata {
    use crate::data::ome::test_support::{czyx_axes, multiscale_v04};
    let scales: Vec<Vec<f64>> = (0..3)
        .map(|l| {
            let xy = 0.25 * f64::from(1 << l);
            vec![1.0, z_scales[l], xy, xy]
        })
        .collect();
    let attrs = json!({
        "multiscales": [multiscale_v04(czyx_axes(), &scales)],
        "omero": {
            "rdefs": { "defaultZ": 10, "model": model },
            "channels": [
                { "label": "off", "color": "FF0000", "active": false },
                { "label": "on", "color": "00FF00", "active": true }
            ]
        }
    });
    let z = |l: usize| (64.0 / z_scales[l]) as u64;
    let found = Fixture::default()
        .v2_group("", attrs)
        .v2_array("0", &[2, z(0), 2048, 2048])
        .v2_array("1", &[2, z(1), 1024, 1024])
        .v2_array("2", &[2, z(2), 512, 512])
        .discover()
        .expect("ome");
    crate::data::DatasetMetadata {
        variables: found.variables,
        ..Default::default()
    }
}

#[test]
fn an_animated_channel_axis_starts_on_its_active_channel() {
    let meta = czyx("greyscale", [1.0; 3]);
    let app = defaults_for(&meta.variables[0]);
    assert_eq!(app.selected.animated_dim, Some(0), "no t: c animates");
    assert_eq!(app.selected.dim_indices[0], 1);
    assert_eq!(app.selected.dim_ranges[0], (0, 1), "the range stays");
}

#[test]
fn a_volume_keeps_its_z_range_and_reads_a_level_that_fits() {
    use crate::plots::PlotType;
    let meta = czyx("color", [1.0, 2.0, 4.0]);
    let var = meta.variables[0].clone();
    let mut app = OctantApp::default();
    app.selected.metadata = Some(meta);
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var);
    crate::ui::variables_panel::dimension_slider::init_composite_defaults(&mut app, &var);
    app.layout.canvas_px = [4096, 4096];
    *app.layers.base.selection_mut() = app.selected.clone();

    app.switch_plot_type(PlotType::Volume);
    assert_eq!(app.selected.dim_ranges[1], (10, 10), "z is not widened");
    assert_eq!(app.selection_level(&app.selected), Some(2), "the coarsest");

    // The whole z stack: too big at the heatmap's level 0, fits at level 2.
    app.switch_plot_type(PlotType::Heatmap);
    app.selected.dim_ranges[1] = (0, 63);
    app.selected.dim_config[1].range = (0, 63);
    app.selected.dim_config[1].active = true;
    assert_eq!(app.selection_level(&app.selected), Some(0));
    let blocked = |app: &mut OctantApp, plot_type: PlotType| {
        crate::ui::plot_type::options::plot_options(app)
            .into_iter()
            .find(|o| o.plot_type == plot_type)
            .and_then(|o| o.blocked)
    };
    assert_eq!(
        blocked(&mut app, PlotType::Volume),
        None,
        "measured on its level"
    );

    // Surfaces are measured on their (coarsest) level, not the heatmap's.
    use crate::ui::variables_panel::{
        calculate_selected_2d_elements, calculate_selected_surface_elements,
    };
    assert_eq!(calculate_selected_2d_elements(&app), 2048 * 2048);
    assert_eq!(calculate_selected_surface_elements(&app), 512 * 512);

    // A resampled heatmap blocks no 3D type of a multiscale variable: those
    // read their own level.
    app.enable_pyramid_resampling = true;
    assert_eq!(blocked(&mut app, PlotType::Volume), None, "not aggregated");
    assert_eq!(blocked(&mut app, PlotType::Surface), None, "not aggregated");
}
