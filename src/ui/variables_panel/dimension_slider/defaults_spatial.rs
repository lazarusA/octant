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
            app.selected.dim_config[i].spatial = SpatialRole::X;
            x_assigned = true;
        } else if !y_assigned && crate::data::coordinates::naming::is_spatial_y_name(dim_name) {
            app.selected.dim_config[i].spatial = SpatialRole::Y;
            y_assigned = true;
        } else if !z_assigned && crate::data::coordinates::naming::is_spatial_z_name(dim_name) {
            app.selected.dim_config[i].spatial = SpatialRole::Z;
            z_assigned = true;
        }
        if rank >= 3
            && !anim_assigned
            && crate::data::coordinates::naming::is_animated_time_name(dim_name)
        {
            app.selected.dim_config[i].animation = AnimationRole::Animated;
            anim_assigned = true;
        }
    }

    // 2. Fallback spatial assignment for unassigned non-channel dimensions. MATLAB arrays
    // are stored transposed (`netcdf/pure/mat.rs`): their rows, stored last, go on Y.
    let matlab = var_info.attributes.contains_key("MATLAB_class");
    for step in 0..rank {
        let i = if matlab { rank - 1 - step } else { step };
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");
        if crate::data::coordinates::naming::is_channel_dim_name(dim_name) {
            continue;
        }
        if app.selected.dim_config[i].spatial == SpatialRole::None
            && app.selected.dim_config[i].animation == AnimationRole::None
        {
            if !y_assigned {
                app.selected.dim_config[i].spatial = SpatialRole::Y;
                y_assigned = true;
            } else if !x_assigned {
                app.selected.dim_config[i].spatial = SpatialRole::X;
                x_assigned = true;
            } else if !z_assigned && rank >= 3 {
                app.selected.dim_config[i].spatial = SpatialRole::Z;
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
            if app.selected.dim_config[i].spatial == SpatialRole::None {
                app.selected.dim_config[i].spatial = SpatialRole::Z;
                break;
            }
        }
    }

    // 4. Without an explicit time axis, an OME-Zarr image animates its Z unless
    // its pyramid downsamples Z; other 4D+ datasets animate an unassigned
    // dimension. Never the channel.
    if anim_assigned {
        return;
    }
    let pick = if var_info.ome.is_some() {
        (0..rank).find(|&i| {
            app.selected.dim_config[i].spatial == SpatialRole::Z && !downsampled(var_info, i)
        })
    } else if rank >= 4 {
        (0..rank).find(|&i| {
            let name = var_info.dimension_names.get(i).map_or("", |s| s.as_str());
            app.selected.dim_config[i].spatial == SpatialRole::None
                && !crate::data::coordinates::naming::is_channel_dim_name(name)
        })
    } else {
        None
    };
    if let Some(i) = pick {
        app.selected.dim_config[i].animation = AnimationRole::Animated;
    }
}

/// Whether a level of `var_info`'s stack downsamples dimension `dim`: an
/// animated dimension reads every level-0 index, so such levels are never read.
fn downsampled(var_info: &VariableInfo, dim: usize) -> bool {
    var_info.multiscale.as_ref().is_some_and(|r| {
        (0..r.set.len())
            .filter_map(|l| r.set.level(l))
            .any(|l| l.factor.get(dim).is_some_and(|&f| f > 1.0))
    })
}
