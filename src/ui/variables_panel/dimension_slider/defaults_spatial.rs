//! Standard Euclidean spatial and temporal dimension role assignments.

use crate::app::{AnimationRole, OctantApp, SpatialRole};
use crate::data::VariableInfo;

/// Assigns standard Euclidean spatial (X, Y, Z) and animation roles for a variable.
pub fn assign_euclidean_roles(app: &mut OctantApp, var_info: &VariableInfo) {
    let rank = var_info.shape.len();
    let mut x_assigned = false;
    let mut y_assigned = false;
    let mut z_assigned = false;
    let mut anim_assigned = false;

    // 1. Explicit naming recognition
    for i in 0..rank {
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");

        if !x_assigned && crate::data::coordinates::naming::is_spatial_x_name(dim_name) {
            app.dim_config[i].spatial = SpatialRole::X;
            x_assigned = true;
        } else if !y_assigned && crate::data::coordinates::naming::is_spatial_y_name(dim_name) {
            app.dim_config[i].spatial = SpatialRole::Y;
            y_assigned = true;
        } else if !z_assigned && crate::data::coordinates::naming::is_spatial_z_name(dim_name) {
            app.dim_config[i].spatial = SpatialRole::Z;
            z_assigned = true;
        }
        if rank >= 3
            && !anim_assigned
            && crate::data::coordinates::naming::is_animated_time_name(dim_name)
        {
            app.dim_config[i].animation = AnimationRole::Animated;
            app.animated_dim = Some(i);
            anim_assigned = true;
        }
    }

    // 2. Fallback spatial assignment for unassigned non-channel dimensions
    for i in 0..rank {
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");
        if crate::data::coordinates::naming::is_channel_dim_name(dim_name) {
            continue;
        }
        if app.dim_config[i].spatial == SpatialRole::None
            && app.dim_config[i].animation == AnimationRole::None
        {
            if !y_assigned {
                app.dim_config[i].spatial = SpatialRole::Y;
                y_assigned = true;
            } else if !x_assigned {
                app.dim_config[i].spatial = SpatialRole::X;
                x_assigned = true;
            } else if !z_assigned && rank >= 3 {
                app.dim_config[i].spatial = SpatialRole::Z;
                z_assigned = true;
            }
        }
    }

    // 3. For 3D datasets, assign Z if still unassigned (skipping channel dimensions)
    if rank >= 3 && !z_assigned {
        for i in 0..rank {
            let dim_name = var_info
                .dimension_names
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("");
            if crate::data::coordinates::naming::is_channel_dim_name(dim_name) {
                continue;
            }
            if app.dim_config[i].spatial == SpatialRole::None {
                app.dim_config[i].spatial = SpatialRole::Z;
                break;
            }
        }
    }

    // 4. For 4D+ datasets without an explicit time axis, default animation to an unassigned dimension
    if rank >= 4
        && !anim_assigned
        && let Some(unassigned) = (0..rank).find(|&i| {
            app.dim_config[i].spatial == SpatialRole::None
                && app.dim_config[i].animation == AnimationRole::None
        })
    {
        app.dim_config[unassigned].animation = AnimationRole::Animated;
        app.animated_dim = Some(unassigned);
    }
}
