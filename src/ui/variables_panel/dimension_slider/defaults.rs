//! Default dimension roles, initialization, and limits.

use super::defaults_clamp::clamp_2d_selection_to_gpu_limits;
use super::defaults_dggs::assign_dggs_roles;
use super::defaults_spatial::assign_euclidean_roles;
use crate::app::{AnimationRole, DimConfig, OctantApp, SpatialRole};
use crate::data::VariableInfo;

/// Initializes default dimension roles (spatial X, Y, Z and animation) for a selected variable.
pub fn init_variable_dimension_defaults(app: &mut OctantApp, var_info: &VariableInfo) {
    let rank = var_info.shape.len();
    app.selected.dim_config = (0..rank)
        .map(|d| DimConfig {
            range: chunk_window(var_info, d, 0),
            ..DimConfig::default()
        })
        .collect();

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

    pin_channel(app, var_info);
    apply_omero_defaults(app, var_info);
    finalize_active_configs(app, var_info, rank);
    if var_info.stack_base().is_some() {
        whole_spatial_extent(app, var_info);
    } else {
        clamp_2d_selection_to_gpu_limits(app, var_info);
    }
    app.stage_current_step();
}

/// A multiscale variable shows its whole x and y extent: the plot reads it
/// at a level that fits (`app::multiscale`), so level 0's size never limits it.
fn whole_spatial_extent(app: &mut OctantApp, var_info: &VariableInfo) {
    for d in 0..app.selected.dim_config.len() {
        let config = app.selected.dim_config[d];
        if !matches!(config.spatial, SpatialRole::X | SpatialRole::Y) {
            continue;
        }
        let last = var_info
            .shape
            .get(d)
            .map_or(0, |&n| (n as usize).saturating_sub(1));
        app.selected.set_range(d, (0, last), config.index.min(last));
    }
}

/// The default range of dimension `dim`: the chunk holding index `at`, or the
/// whole dimension for a channel axis or an array without chunks.
pub(super) fn chunk_window(var_info: &VariableInfo, dim: usize, at: usize) -> (usize, usize) {
    let size = var_info.shape.get(dim).copied().unwrap_or(0) as usize;
    let last = size.saturating_sub(1);
    let name = var_info
        .dimension_names
        .get(dim)
        .map(|s| s.as_str())
        .unwrap_or("");
    let chunk = var_info.chunk_shape.get(dim).copied().unwrap_or(0) as usize;
    if chunk == 0 || crate::data::coordinates::naming::is_channel_dim_name(name) {
        return (0, last);
    }
    let start = (at.min(last) / chunk) * chunk;
    (start, (start + chunk).min(size).saturating_sub(1))
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
    app.selected.dim_config[0].spatial = if is_grid {
        SpatialRole::Grid
    } else {
        SpatialRole::X
    };
    let dim_size = var_info.shape[0] as usize;
    let max_selectable = dim_size.min(crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS);
    app.selected
        .set_range(0, (0, max_selectable.saturating_sub(1)), 0);
    app.selected.dim_config[0].active = true;
}

/// The OMERO default plane (`rdefs.defaultZ`) and time point (`defaultT`)
/// of an OME-Zarr image: z collapses to its plane unless animated; an
/// animated z or t keeps its window, moved to hold the default step, and
/// playback starts there.
fn apply_omero_defaults(app: &mut OctantApp, var_info: &VariableInfo) {
    let Some(ome) = &var_info.ome else {
        return;
    };
    let image = &ome.image;
    if let (Some(dz), Some(z)) = (image.default_z, image.z_axis()) {
        if app.selected.animated_dim() == Some(z) {
            hold_and_play(app, var_info, z, dz);
        } else {
            pin_dim(app, var_info, z, dz);
        }
    }
    if let (Some(dt), Some(t)) = (
        image.default_t,
        image.axis_of_kind(crate::data::ome::AxisKind::Time),
    ) {
        hold_and_play(app, var_info, t, dt);
    }
}

/// Moves dimension `dim`'s window to hold step `k`; playback starts there
/// when `dim` is the animated one.
fn hold_and_play(app: &mut OctantApp, var_info: &VariableInfo, dim: usize, k: usize) {
    let Some(&size) = var_info.shape.get(dim) else {
        return;
    };
    let last = (size as usize).saturating_sub(1);
    app.selected.hold_step(dim, k, last);
    if app.selected.animated_dim() == Some(dim) {
        app.playback.current_timestep = k.min(last);
    }
}

/// A channel dimension opens on OMERO's first active channel within its
/// range, else the range's first, unless the variable opens in color
/// (`composite::opens_in_color`): pinned there (one plane) unless it is a
/// plot axis. Defaults never animate a channel.
fn pin_channel(app: &mut OctantApp, var_info: &VariableInfo) {
    let Some(c) = var_info.channel_axis() else {
        return;
    };
    let is_tiff = app.selected.store_kind.is_geotiff();
    let Some(&cfg) = app.selected.dim_config.get(c) else {
        return;
    };
    let plot_axis = matches!(
        cfg.spatial,
        SpatialRole::X | SpatialRole::Y | SpatialRole::Grid
    );
    if plot_axis || super::composite::opens_in_color(var_info, is_tiff) {
        return;
    }
    let (lo, hi) = app
        .selected
        .dim_config
        .get(c)
        .map(|c| c.range)
        .unwrap_or((0, 0));
    let active = |i: &usize| {
        let channels = var_info.ome.as_ref().map(|o| o.image.channels.as_slice());
        channels
            .and_then(|ch| ch.get(*i))
            .is_some_and(|ch| ch.active)
    };
    let k = (lo.min(hi)..=hi.max(lo)).find(active).unwrap_or(lo.min(hi));
    pin_dim(app, var_info, c, k);
}

/// Pins dimension `dim` of the selection to index `k`, clamped into its size.
fn pin_dim(app: &mut OctantApp, var_info: &VariableInfo, dim: usize, k: usize) {
    let Some(&size) = var_info.shape.get(dim) else {
        return;
    };
    app.selected
        .set_index(dim, k.min((size as usize).saturating_sub(1)));
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

        if app.selected.dim_config[i].spatial != SpatialRole::None {
            app.selected.dim_config[i].active = true;
        }
        if app.selected.dim_config[i].animation == AnimationRole::Animated {
            app.selected.dim_config[i].active = true;
        }
        if is_channel && dim_size >= 2 {
            app.selected.dim_config[i].active = true;
        }
    }
}
