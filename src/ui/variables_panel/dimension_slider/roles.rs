//! Dimension spatial and animation role transitions.

use crate::app::{AnimationRole, OctantApp, SpatialRole};

pub fn apply_role_change(
    dim: usize,
    spatial: SpatialRole,
    anim: AnimationRole,
    app: &mut OctantApp,
) {
    let old_spatial = app.selected.dim_config[dim].spatial;
    let old_anim = app.selected.dim_config[dim].animation;

    if spatial != old_spatial && spatial != SpatialRole::None {
        for j in 0..app.selected.dim_config.len() {
            if j != dim {
                let should_clear = (spatial == SpatialRole::Grid
                    && (app.selected.dim_config[j].spatial == SpatialRole::Grid
                        || app.selected.dim_config[j].spatial == SpatialRole::X
                        || app.selected.dim_config[j].spatial == SpatialRole::Y))
                    || app.selected.dim_config[j].spatial == spatial
                    || (app.selected.dim_config[j].spatial == SpatialRole::Grid
                        && (spatial == SpatialRole::X || spatial == SpatialRole::Y));

                if should_clear {
                    app.selected.dim_config[j].spatial = SpatialRole::None;
                    if app.selected.dim_config[j].animation == AnimationRole::None {
                        app.selected.dim_config[j].active = false;
                    }
                }
            }
        }
    }

    if anim != old_anim && anim == AnimationRole::Animated {
        for j in 0..app.selected.dim_config.len() {
            if j != dim && app.selected.dim_config[j].animation == AnimationRole::Animated {
                app.selected.dim_config[j].animation = AnimationRole::None;
                if app.selected.dim_config[j].spatial == SpatialRole::None {
                    app.selected.dim_config[j].active = false;
                }
            }
        }
    }

    app.selected.dim_config[dim].spatial = spatial;
    app.selected.dim_config[dim].animation = anim;

    if spatial != SpatialRole::None || anim == AnimationRole::Animated {
        app.selected.dim_config[dim].active = true;
    }

    // A dimension that becomes spatial spans the chunk holding its index,
    // like every default range; one that stays spatial (this runs every
    // frame) keeps the range it has.
    if spatial != SpatialRole::None
        && spatial != old_spatial
        && let Some(&(st, en)) = app.selected.dim_ranges.get(dim)
        && st == en
        && let Some(var_info) = app
            .selected
            .metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(app.selected.variable_idx))
    {
        app.selected.dim_ranges[dim] = super::defaults::chunk_window(var_info, dim, st);
    }

    app.selected.spatial_dims.clear();
    for j in 0..app.selected.dim_config.len() {
        if app.selected.dim_config[j].spatial != SpatialRole::None {
            app.selected.spatial_dims.push(j);
        }
    }
    app.selected
        .spatial_dims
        .sort_by_key(|&d| match app.selected.dim_config[d].spatial {
            SpatialRole::Grid => 0,
            SpatialRole::X => 1,
            SpatialRole::Y => 2,
            SpatialRole::Z => 3,
            SpatialRole::None => 99,
        });

    app.selected.animated_dim = app
        .selected
        .dim_config
        .iter()
        .position(|c| c.animation == AnimationRole::Animated);
}
