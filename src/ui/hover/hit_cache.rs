//! Hover hits resolved once per pointer position and view: surface and
//! sphere hits march the cursor ray, so a still pointer over an unchanged
//! view reuses its hit from egui temp memory.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::plots::PlotType;
use crate::ui::hover::camera::Camera3D;
use crate::ui::hover::hit::{HitResult, resolve_hit_coordinates};
use crate::ui::hover::raycast_volume::VolumeSampler;
use crate::ui::hover::sample_2d::Transform2D;
use crate::ui::temp_cache::{cached, hash_key};
use egui::{Pos2, Rect};

/// Key of everything a surface or sphere hit depends on, so a still pointer
/// over an unchanged view reuses its marched hit (`None` for other plots).
pub fn mesh_hit_key(app: &OctantApp, rect: Rect, hover_pos: Pos2) -> Option<u64> {
    let plot_type = app.effective_canvas_plot_type();
    if !matches!(plot_type, PlotType::Sphere | PlotType::Surface) {
        return None;
    }
    let (mesh, base) = (&app.plot_configs.mesh, &app.layers.base);
    let floats = [
        hover_pos.x,
        hover_pos.y,
        rect.min.x,
        rect.min.y,
        rect.max.x,
        rect.max.y,
        app.nav.sphere_rotation_x,
        app.nav.sphere_rotation_y,
        app.nav.sphere_zoom,
        app.nav.orbit_target[0],
        app.nav.orbit_target[1],
        app.nav.orbit_target[2],
        mesh.surface_displacement,
        mesh.sphere_displacement,
        base.color.range_min,
        base.color.range_max,
    ]
    .map(f32::to_bits);
    let flags = [
        mesh.surface_flip_height,
        mesh.sphere_flip_height,
        base.composite.enabled,
    ];
    let modes = (mesh.surface_mode, mesh.sphere_mode);
    Some(hash_key((
        plot_type,
        floats,
        flags,
        modes,
        base.data.matrix_version,
    )))
}

/// The camera and the valid hit under `hover_pos`, if any.
pub fn resolve_hit(
    app: &OctantApp,
    ctx: &egui::Context,
    matrix: Option<&MatrixData>,
    sampler: Option<&VolumeSampler>,
    rect: Rect,
    hover_pos: Pos2,
) -> Option<(Camera3D, HitResult)> {
    let camera = Camera3D::from_app(app, rect);
    let transform_2d = Transform2D::from_app(app, rect);
    let resolve = || {
        let transform = &transform_2d;
        resolve_hit_coordinates(app, matrix, &camera, sampler, transform, rect, hover_pos)
    };
    let hit = match mesh_hit_key(app, rect, hover_pos) {
        Some(key) => cached(ctx, egui::Id::unique("hover_mesh_hit"), key, resolve),
        None => resolve(),
    };
    hit.is_valid.then_some((camera, hit))
}
