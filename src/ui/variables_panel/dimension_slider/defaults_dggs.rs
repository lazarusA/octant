//! DGGS and HEALPix discrete global grid dimension default role assignments.

use crate::app::{AnimationRole, OctantApp, SpatialRole};
use crate::data::VariableInfo;

/// Configures dimension roles for discrete global grid / HEALPix variables.
pub fn assign_dggs_roles(app: &mut OctantApp, var_info: &VariableInfo, grid_i: usize) {
    let rank = var_info.shape.len();
    app.dim_config[grid_i].spatial = SpatialRole::Grid;
    let grid_size = var_info.shape[grid_i] as usize;
    let max_selectable = grid_size.min(crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS);
    app.selected_dim_ranges[grid_i] = (0, max_selectable.saturating_sub(1));
    app.dim_config[grid_i].range = app.selected_dim_ranges[grid_i];

    let mut z_assigned = false;
    let mut anim_assigned = false;

    for i in 0..rank {
        if i == grid_i {
            continue;
        }
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");
        if !z_assigned && crate::data::coordinates::naming::is_spatial_z_name(dim_name) {
            app.dim_config[i].spatial = SpatialRole::Z;
            z_assigned = true;
        }
        if !anim_assigned && crate::data::coordinates::naming::is_animated_time_name(dim_name) {
            app.dim_config[i].animation = AnimationRole::Animated;
            app.animated_dim = Some(i);
            anim_assigned = true;
        }
    }

    if !anim_assigned {
        for i in 0..rank {
            if i != grid_i && app.dim_config[i].spatial == SpatialRole::None {
                app.dim_config[i].animation = AnimationRole::Animated;
                app.animated_dim = Some(i);
                break;
            }
        }
    }
}
