//! Default dimension roles, initialization, and limits.

use crate::app::{AnimationRole, DimConfig, OctantApp, SpatialRole};
use crate::data::VariableInfo;

/// Initializes default dimension roles (spatial X, Y, Z and animation) for a selected variable.
pub fn init_variable_dimension_defaults(app: &mut OctantApp, var_info: &VariableInfo) {
    let rank = var_info.shape.len();

    app.dim_config = vec![DimConfig::default(); rank];
    app.selected_dim_indices = vec![0; rank];
    app.selected_dim_ranges.clear();
    app.spatial_dims.clear();
    app.animated_dim = None;
    app.rgb_composite_channels = [0, 1, 2];

    if rank < 3 || var_info.shape.first().copied().unwrap_or(0) < 3 {
        app.rgb_composite_mode = false;
        if app.active_colormap == 1000 {
            app.active_colormap = 0;
        }
    }

    for i in 0..rank {
        let dim_size = var_info.shape[i] as usize;
        let chunk_size = var_info.chunk_shape.get(i).copied().unwrap_or(0) as usize;
        let range_end = if chunk_size > 0 {
            chunk_size.min(dim_size).saturating_sub(1)
        } else {
            dim_size.saturating_sub(1)
        };
        app.selected_dim_ranges.push((0, range_end));
    }

    let dggs_opt =
        crate::data::coordinates::dggs::DggsMetadata::from_attributes(&var_info.attributes);

    if rank == 1 {
        let dim_name = var_info
            .dimension_names
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let is_grid = dggs_opt.as_ref().is_some_and(|d| d.matches_dim(dim_name))
            || crate::data::coordinates::naming::is_healpix_dim_name(dim_name);
        app.dim_config[0].spatial = if is_grid {
            SpatialRole::Grid
        } else {
            SpatialRole::X
        };
        let dim_size = var_info.shape[0] as usize;
        let max_selectable = dim_size.min(crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS);
        app.selected_dim_ranges[0] = (0, max_selectable.saturating_sub(1));
        app.dim_config[0].active = true;
        app.dim_config[0].range = app.selected_dim_ranges[0];
        app.spatial_dims.push(0);
        return;
    }

    // Check if this variable has a discrete global grid / HEALPix dimension via DGGS or naming heuristics
    let healpix_dim_idx = (0..rank).find(|&i| {
        let name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");
        dggs_opt.as_ref().is_some_and(|d| d.matches_dim(name))
            || crate::data::coordinates::naming::is_healpix_dim_name(name)
    });

    if let Some(grid_i) = healpix_dim_idx {
        app.dim_config[grid_i].spatial = SpatialRole::Grid;
        let grid_size = var_info.shape[grid_i] as usize;
        let max_selectable = grid_size.min(crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS);
        app.selected_dim_ranges[grid_i] = (0, max_selectable.saturating_sub(1));
        app.dim_config[grid_i].range = app.selected_dim_ranges[grid_i];

        let mut z_assigned = false;
        let mut anim_assigned = false;

        // Check for Z (layer/level/depth/elevation) and Anim (time)
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

        // Fallback animation assignment if time name was not standard
        if !anim_assigned {
            for i in 0..rank {
                if i != grid_i && app.dim_config[i].spatial == SpatialRole::None {
                    app.dim_config[i].animation = AnimationRole::Animated;
                    app.animated_dim = Some(i);
                    break;
                }
            }
        }
    } else {
        let mut x_assigned = false;
        let mut y_assigned = false;
        let mut z_assigned = false;
        let mut anim_assigned = false;

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

        // 2. Fallback spatial assignment for unassigned dimensions (skipping channel dimensions)
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

        // 4. For 3D+ datasets, default animation dimension
        if rank >= 3 && !anim_assigned {
            let default_anim = (0..rank)
                .find(|&i| app.dim_config[i].spatial == SpatialRole::Z)
                .unwrap_or(0);
            app.dim_config[default_anim].animation = AnimationRole::Animated;
            app.animated_dim = Some(default_anim);
        }
    }

    if let Some(dz_str) = var_info.attributes.get("default_z")
        && let Ok(dz) = dz_str.parse::<usize>()
        && let Some(z_idx) = (0..rank).find(|&i| {
            let dim_name = var_info
                .dimension_names
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("");
            crate::data::coordinates::naming::is_spatial_z_name(dim_name)
        })
    {
        let dim_max = var_info.shape[z_idx].saturating_sub(1) as usize;
        let clamped_dz = dz.min(dim_max);
        app.selected_dim_indices[z_idx] = clamped_dz;
        app.selected_dim_ranges[z_idx] = (clamped_dz, clamped_dz);
        app.dim_config[z_idx].range = (clamped_dz, clamped_dz);
    }

    for i in 0..rank {
        if app.dim_config[i].spatial != SpatialRole::None {
            app.spatial_dims.push(i);
            app.dim_config[i].active = true;
        }
        if app.dim_config[i].animation == AnimationRole::Animated {
            app.dim_config[i].active = true;
        }
        if let Some(&r) = app.selected_dim_ranges.get(i) {
            app.dim_config[i].range = r;
        }
    }

    // Clamp initial 2D selection to GPU limits if needed
    if let (Some(x_idx), Some(y_idx)) = (
        app.dim_config
            .iter()
            .position(|c| c.spatial == SpatialRole::X),
        app.dim_config
            .iter()
            .position(|c| c.spatial == SpatialRole::Y),
    ) {
        let x_span = (app.selected_dim_ranges[x_idx]
            .1
            .saturating_sub(app.selected_dim_ranges[x_idx].0)
            + 1)
        .max(1);
        let y_span = (app.selected_dim_ranges[y_idx]
            .1
            .saturating_sub(app.selected_dim_ranges[y_idx].0)
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
            let x_start = app.selected_dim_ranges[x_idx].0;
            let y_start = app.selected_dim_ranges[y_idx].0;
            let max_x = var_info.shape[x_idx].saturating_sub(1) as usize;
            let max_y = var_info.shape[y_idx].saturating_sub(1) as usize;
            app.selected_dim_ranges[x_idx] =
                (x_start, (x_start + new_x.saturating_sub(1)).min(max_x));
            app.selected_dim_ranges[y_idx] =
                (y_start, (y_start + new_y.saturating_sub(1)).min(max_y));
            app.dim_config[x_idx].range = app.selected_dim_ranges[x_idx];
            app.dim_config[y_idx].range = app.selected_dim_ranges[y_idx];
        }
    }

    app.spatial_dims
        .sort_by_key(|&d| match app.dim_config[d].spatial {
            SpatialRole::Grid => 0,
            SpatialRole::X => 1,
            SpatialRole::Y => 2,
            SpatialRole::Z => 3,
            SpatialRole::None => 99,
        });
}
