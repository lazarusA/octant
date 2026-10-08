//! Clamping routines for initial 2D slice selection against GPU storage buffer limits.

use crate::app::{OctantApp, SpatialRole};
use crate::data::VariableInfo;

/// Clamps initial selected 2D spatial dimensions if the total element count exceeds GPU limits.
pub fn clamp_2d_selection_to_gpu_limits(app: &mut OctantApp, var_info: &VariableInfo) {
    let (Some(x_idx), Some(y_idx)) = (
        app.selected
            .dim_config
            .iter()
            .position(|c| c.spatial == SpatialRole::X),
        app.selected
            .dim_config
            .iter()
            .position(|c| c.spatial == SpatialRole::Y),
    ) else {
        return;
    };

    let x_span = (app.selected.dim_ranges[x_idx]
        .1
        .saturating_sub(app.selected.dim_ranges[x_idx].0)
        + 1)
    .max(1);
    let y_span = (app.selected.dim_ranges[y_idx]
        .1
        .saturating_sub(app.selected.dim_ranges[y_idx].0)
        + 1)
    .max(1);
    let total_2d = x_span.saturating_mul(y_span);

    if total_2d > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        let scale = ((crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS as f64)
            / (total_2d as f64))
            .sqrt()
            * 0.95;
        let new_x = ((x_span as f64) * scale).max(1.0) as usize;
        let new_y = ((y_span as f64) * scale).max(1.0) as usize;
        let x_start = app.selected.dim_ranges[x_idx].0;
        let y_start = app.selected.dim_ranges[y_idx].0;
        let max_x = var_info.shape[x_idx].saturating_sub(1) as usize;
        let max_y = var_info.shape[y_idx].saturating_sub(1) as usize;

        app.selected.dim_ranges[x_idx] = (x_start, (x_start + new_x.saturating_sub(1)).min(max_x));
        app.selected.dim_ranges[y_idx] = (y_start, (y_start + new_y.saturating_sub(1)).min(max_y));
        app.selected.dim_config[x_idx].range = app.selected.dim_ranges[x_idx];
        app.selected.dim_config[y_idx].range = app.selected.dim_ranges[y_idx];
    }
}
