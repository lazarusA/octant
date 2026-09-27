//! Default dimension roles, initialization, and limits.

use super::defaults_clamp::clamp_2d_selection_to_gpu_limits;
use super::defaults_dggs::assign_dggs_roles;
use super::defaults_spatial::assign_euclidean_roles;
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

    init_initial_ranges(app, var_info, rank);

    let dggs_opt =
        crate::data::coordinates::dggs::DggsMetadata::from_attributes(&var_info.attributes);

    if rank == 1 {
        init_rank1_defaults(app, var_info, &dggs_opt);
        return;
    }

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
        assign_dggs_roles(app, var_info, grid_i);
    } else {
        assign_euclidean_roles(app, var_info);
    }

    apply_omero_default_z(app, var_info, rank);
    finalize_active_configs(app, var_info, rank);
    clamp_2d_selection_to_gpu_limits(app, var_info);
    sort_spatial_dims(app);
}

fn init_initial_ranges(app: &mut OctantApp, var_info: &VariableInfo, rank: usize) {
    for i in 0..rank {
        let dim_size = var_info.shape[i] as usize;
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");
        let is_channel = crate::data::coordinates::naming::is_channel_dim_name(dim_name);

        let chunk_size = var_info.chunk_shape.get(i).copied().unwrap_or(0) as usize;
        let range_end = if is_channel {
            dim_size.saturating_sub(1)
        } else if chunk_size > 0 {
            chunk_size.min(dim_size).saturating_sub(1)
        } else {
            dim_size.saturating_sub(1)
        };
        app.selected_dim_ranges.push((0, range_end));
    }
}

fn init_rank1_defaults(
    app: &mut OctantApp,
    var_info: &VariableInfo,
    dggs_opt: &Option<crate::data::coordinates::dggs::DggsMetadata>,
) {
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
}

fn apply_omero_default_z(app: &mut OctantApp, var_info: &VariableInfo, rank: usize) {
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
}

fn finalize_active_configs(app: &mut OctantApp, var_info: &VariableInfo, rank: usize) {
    for i in 0..rank {
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");
        let is_channel = crate::data::coordinates::naming::is_channel_dim_name(dim_name);
        let dim_size = var_info.shape[i] as usize;

        if app.dim_config[i].spatial != SpatialRole::None {
            app.spatial_dims.push(i);
            app.dim_config[i].active = true;
        }
        if app.dim_config[i].animation == AnimationRole::Animated {
            app.dim_config[i].active = true;
        }
        if is_channel && dim_size >= 2 {
            app.dim_config[i].active = true;
        }
        if let Some(&r) = app.selected_dim_ranges.get(i) {
            app.dim_config[i].range = r;
        }
    }
}

fn sort_spatial_dims(app: &mut OctantApp) {
    app.spatial_dims
        .sort_by_key(|&d| match app.dim_config[d].spatial {
            SpatialRole::Grid => 0,
            SpatialRole::X => 1,
            SpatialRole::Y => 2,
            SpatialRole::Z => 3,
            SpatialRole::None => 99,
        });
}
