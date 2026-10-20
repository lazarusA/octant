//! Capacity calculations, byte counts, and volume rendering validation.

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

/// Calculates requested download bytes and total file size for a variable.
pub fn calculate_download_sizes(
    var_info: &VariableInfo,
    dim_config: &[DimConfig],
    selected_ranges: &[(usize, usize)],
) -> (u64, u64) {
    let dtype_bytes = crate::utils::data_type_bytes(&var_info.data_type);
    let active_dims: Vec<bool> = dim_config.iter().map(|c| c.active).collect();
    crate::utils::math::calculate_download_sizes(
        &var_info.shape,
        var_info.file_size,
        dtype_bytes,
        &active_dims,
        selected_ranges,
    )
}

/// The 3D volume elements the staged selection reads (on its plot's level)
/// over its active dimensions.
pub fn calculate_selected_volume_elements(app: &OctantApp) -> usize {
    selection_volume_elements(app, &app.selected)
}

/// The 3D volume elements `selection` reads, on the level its plot reads,
/// over its active dimensions.
fn selection_volume_elements(app: &OctantApp, selection: &VariableSelection) -> usize {
    let Some((var_info, ranges)) = app.read_window(selection) else {
        return 0;
    };

    let active_dims: Vec<bool> = (0..var_info.shape.len())
        .map(|i| {
            selection
                .dim_config
                .get(i)
                .map(|c| c.active)
                .unwrap_or(false)
                || selection.spatial_dims.contains(&i)
                || selection.animated_dim == Some(i)
        })
        .collect();

    crate::utils::math::calculate_volume_elements(&var_info.shape, &active_dims, &ranges)
}

/// The spatial X and Y plane elements the staged selection reads, on its
/// plot's level.
pub fn calculate_selected_2d_elements(app: &OctantApp) -> usize {
    let Some((var_info, ranges)) = app.read_window(&app.selected) else {
        return 0;
    };

    let rank = var_info.shape.len();
    let (x_dim, y_dim, _) = OctantApp::resolve_spatial_axes(
        rank,
        &var_info.dimension_names,
        &var_info.dimension_names,
        &app.selected.dim_config,
    );

    crate::utils::math::calculate_2d_elements(&var_info.shape, x_dim, y_dim, &ranges)
}

/// Checks if 3D Volume / Point Cloud rendering is allowed under GPU storage limits.
pub fn is_volume_allowed_for_selection(app: &OctantApp) -> bool {
    is_volume_allowed(app, &app.selected, app.layers.base.data.volume.as_ref())
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
