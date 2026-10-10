//! 2D and 1D plot axis lines, tick marks, and coordinate labels.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::plots::PlotType;
use crate::ui::axes::{PlotAxisOptions, draw_plot_axes};

type AxisTuple = (
    (f64, f64),
    (f64, f64),
    String,
    String,
    Option<String>,
    Option<String>,
);

/// Draws dynamic axis lines, tick marks, and titles on 2D and 1D plots.
pub fn draw_canvas_axes(
    app: &OctantApp,
    ui: &mut egui::Ui,
    canvas_rect: egui::Rect,
    plot_rect: egui::Rect,
    canvas_plot_type: PlotType,
) {
    if canvas_plot_type.is_3d() {
        return;
    }

    let Some(matrix) = &app.layers.base.data.matrix else {
        return;
    };

    let (x_dom, y_dom, x_label, y_label, x_units, y_units) = if canvas_plot_type == PlotType::Line {
        resolve_line_plot_axis_info(app, matrix)
    } else {
        resolve_2d_plot_axis_info(app, matrix)
    };

    let options = PlotAxisOptions {
        x_domain: x_dom,
        y_domain: y_dom,
        x_title: &x_label,
        y_title: &y_label,
        x_units: x_units.as_deref(),
        y_units: y_units.as_deref(),
    };

    draw_plot_axes(ui, canvas_rect, plot_rect, &options);
}

fn resolve_line_plot_axis_info(app: &OctantApp, matrix: &MatrixData) -> AxisTuple {
    let y_min = app.layers.base.color.range_min as f64;
    let y_max = app.layers.base.color.range_max as f64;
    let profile_len = match app.plot_configs.line.profile_dim_idx {
        2 => app
            .layers
            .base
            .data
            .volume
            .as_ref()
            .map_or(matrix.width, |v| v.depth),
        1 => matrix.height,
        _ => matrix.width,
    };

    let y_name = app
        .plotted_variable_info()
        .map(|var| {
            if let Some(u) = &var.units {
                format!("{} [{u}]", var.name)
            } else if let Some(u) = var.attributes.get("units") {
                format!("{} [{u}]", var.name)
            } else {
                var.name.clone()
            }
        })
        .unwrap_or_else(|| "Data Value".to_string());

    let target_dim_idx = app.get_spatial_dim_index(app.plot_configs.line.profile_dim_idx);
    let fallback_dim_name = match app.plot_configs.line.profile_dim_idx {
        2 => "z",
        1 => "y",
        _ => "x",
    };

    let (x_bounds, x_title, x_units) =
        app.resolve_axis_bounds_and_title(target_dim_idx, fallback_dim_name, profile_len);

    (x_bounds, (y_min, y_max), x_title, y_name, x_units, None)
}

fn resolve_2d_plot_axis_info(app: &OctantApp, matrix: &MatrixData) -> AxisTuple {
    if matrix.grid.is_healpix() {
        let x_bounds = (-180.0, 180.0);
        let y_bounds = (-90.0, 90.0);
        let x_title = "Longitude (°)".to_string();
        let y_title = "Latitude (°)".to_string();
        (x_bounds, y_bounds, x_title, y_title, None, None)
    } else {
        let (orig_w, orig_h) = app.active_data_dimensions_2d();
        let x_dim = app.get_spatial_dim_index(0);
        let y_dim = app.get_spatial_dim_index(1);

        let (x_bounds, x_title, x_units) = app.resolve_axis_bounds_and_title(x_dim, "X", orig_w);
        let (y_bounds, y_title, y_units) = app.resolve_axis_bounds_and_title(y_dim, "Y", orig_h);

        (x_bounds, y_bounds, x_title, y_title, x_units, y_units)
    }
}
