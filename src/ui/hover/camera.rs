//! 3D camera geometry, ray casting, and bounding-box intersection utilities.

use crate::app::OctantApp;
use crate::plots::camera::OrbitCamera;
use egui::{Pos2, Rect};

/// A 3D ray with origin and normalized direction.
#[derive(Clone, Copy, Debug)]
pub struct Ray3D {
    pub origin: [f32; 3],
    pub dir: [f32; 3],
}

/// The 3D plots' camera on the canvas: rays from the cursor and points
/// projected back to the screen, with the same `OrbitCamera` the shaders draw.
#[derive(Clone, Copy, Debug)]
pub struct Camera3D {
    pub rect: Rect,
    pub orbit: OrbitCamera,
}

impl Camera3D {
    pub fn from_app(app: &OctantApp, rect: Rect) -> Self {
        let aspect = rect.width() / rect.height().max(1.0);
        Self {
            rect,
            orbit: app
                .nav
                .orbit_camera(app.effective_canvas_plot_type(), aspect),
        }
    }

    /// The world space ray under `screen_pos`.
    pub fn cast_ray(&self, screen_pos: Pos2) -> Ray3D {
        let clip_x = (screen_pos.x - self.rect.center().x) / (0.5 * self.rect.width().max(1.0));
        let clip_y = -(screen_pos.y - self.rect.center().y) / (0.5 * self.rect.height().max(1.0));
        let (origin, dir) = self.orbit.ray([clip_x, clip_y]);
        Ray3D { origin, dir }
    }

    /// Forward projects a 3D model space point to screen coordinates.
    pub fn project_point(&self, pt: [f32; 3]) -> Option<Pos2> {
        let [x, y] = self.orbit.project(pt)?;
        let center = self.rect.center();
        Some(Pos2::new(
            center.x + x * (0.5 * self.rect.width()),
            center.y - y * (0.5 * self.rect.height()),
        ))
    }
}

/// Standard AABB Ray-Box intersection using the slab method.
pub fn intersect_aabb(ray: &Ray3D, min_b: [f32; 3], max_b: [f32; 3]) -> Option<(f32, f32)> {
    let inv_dx = if ray.dir[0].abs() > 1e-6 {
        1.0 / ray.dir[0]
    } else {
        1e6
    };
    let inv_dy = if ray.dir[1].abs() > 1e-6 {
        1.0 / ray.dir[1]
    } else {
        1e6
    };
    let inv_dz = if ray.dir[2].abs() > 1e-6 {
        1.0 / ray.dir[2]
    } else {
        1e6
    };

    let t1_x = (min_b[0] - ray.origin[0]) * inv_dx;
    let t2_x = (max_b[0] - ray.origin[0]) * inv_dx;
    let t_min_x = t1_x.min(t2_x);
    let t_max_x = t1_x.max(t2_x);

    let t1_y = (min_b[1] - ray.origin[1]) * inv_dy;
    let t2_y = (max_b[1] - ray.origin[1]) * inv_dy;
    let t_min_y = t1_y.min(t2_y);
    let t_max_y = t1_y.max(t2_y);

    let t1_z = (min_b[2] - ray.origin[2]) * inv_dz;
    let t2_z = (max_b[2] - ray.origin[2]) * inv_dz;
    let t_min_z = t1_z.min(t2_z);
    let t_max_z = t1_z.max(t2_z);

    let t_enter = t_min_x.max(t_min_y).max(t_min_z);
    let t_exit = t_max_x.min(t_max_y).min(t_max_z);

    if t_enter > t_exit || t_exit <= 0.0 {
        None
    } else {
        Some((t_enter, t_exit))
    }
}
