//! Capacity calculations, byte counts, and volume rendering validation.

use crate::app::multiscale_window::ReadWindow;
use crate::app::{DimConfig, OctantApp, VariableSelection};
use crate::data::{VariableInfo, VolumeData};

/// Computes the maximum steps along the animated dimension that fit within GPU limits.
pub fn calculate_max_animated_steps(
    var_info: &VariableInfo,
    dim_config: &[DimConfig],
    selected_ranges: &[(usize, usize)],
    anim_dim: usize,
) -> (usize, usize, usize) {
    let active_dims: Vec<bool> = dim_config.iter().map(|c| c.active).collect();
    crate::utils::math::calculate_max_animated_steps(
        &var_info.shape,
        &active_dims,
        selected_ranges,
        anim_dim,
        crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS,
    )
}

/// The cells and bytes the selection over `selected_ranges` downloads from
/// `var_info` (its active dimensions), and the variable's total bytes.
pub fn calculate_download_sizes(
    var_info: &VariableInfo,
    dim_config: &[DimConfig],
    selected_ranges: &[(usize, usize)],
) -> (u64, u64, u64) {
    crate::utils::math::calculate_download_sizes(
        &var_info.shape,
        var_info.file_size,
        crate::utils::data_type_bytes(&var_info.data_type),
        |i| dim_config.get(i).is_some_and(|c| c.active),
        selected_ranges,
    )
}

/// The 3D volume elements the staged selection reads (on its plot's level)
/// over its active dimensions.
pub fn calculate_selected_volume_elements(app: &OctantApp) -> usize {
    selection_volume_elements(app, &app.selected)
}

/// The 3D volume elements `selection` reads, on the level a volume or point
/// cloud reads (whatever its plot is now), over its active dimensions.
fn selection_volume_elements(app: &OctantApp, selection: &VariableSelection) -> usize {
    app.read_window_as(selection, crate::plots::PlotType::Volume)
        .map_or(0, |w| window_volume_elements(selection, &w))
}

/// The 3D volume elements `selection` reads over `window`
/// (`OctantApp::read_window`), over its active dimensions.
fn window_volume_elements(selection: &VariableSelection, window: &ReadWindow) -> usize {
    let (var_info, ranges) = window;
    let is_active = |i: usize| {
        selection.dim_config.get(i).is_some_and(|c| c.active)
            || selection.spatial_dims.contains(&i)
            || selection.animated_dim == Some(i)
    };
    crate::utils::math::calculate_volume_elements(&var_info.shape, is_active, ranges)
}

/// The spatial X and Y plane elements the staged selection reads, on its
/// plot's level.
pub fn calculate_selected_2d_elements(app: &OctantApp) -> usize {
    app.read_window(&app.selected)
        .map_or(0, |w| window_2d_elements(&app.selected, &w))
}

/// The spatial X and Y plane elements the staged selection reads as a
/// surface or sphere (on the level those read, whatever its plot is now).
pub fn calculate_selected_surface_elements(app: &OctantApp) -> usize {
    app.read_window_as(&app.selected, crate::plots::PlotType::Surface)
        .map_or(0, |w| window_2d_elements(&app.selected, &w))
}

/// The spatial X and Y plane elements `selection` reads over `window`.
pub fn window_2d_elements(selection: &VariableSelection, window: &ReadWindow) -> usize {
    let (var_info, ranges) = window;
    let (x_dim, y_dim, _) = OctantApp::resolve_spatial_axes(
        var_info.shape.len(),
        &var_info.dimension_names,
        &var_info.dimension_names,
        &selection.dim_config,
    );
    crate::utils::math::calculate_2d_elements(&var_info.shape, x_dim, y_dim, ranges)
}

/// `var`'s size in bytes: its stored size when known, else its shape times
/// its data type.
pub fn variable_bytes(var: &VariableInfo) -> u64 {
    match var.file_size {
        0 => crate::utils::calculate_variable_size_bytes(&var.shape, &var.data_type),
        size => size,
    }
}

/// Whether `selection` can draw as a volume or point cloud next to a layer's
/// current `volume`, within GPU storage limits, on the level its plot reads.
pub fn is_volume_allowed(
    app: &OctantApp,
    selection: &VariableSelection,
    volume: Option<&VolumeData>,
) -> bool {
    // Discrete global grids (HEALPix) do not support 3D Volume or PointCloud raycasting
    if selection
        .dim_config
        .iter()
        .any(|c| c.spatial == crate::app::SpatialRole::Grid)
    {
        return false;
    }
    let elements = selection_volume_elements(app, selection);
    if elements == 0 && selection.metadata.is_some() {
        return false;
    }
    if elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        return false;
    }
    if let Some(vdata) = volume
        && vdata.values.len() > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS
    {
        return false;
    }
    true
}
