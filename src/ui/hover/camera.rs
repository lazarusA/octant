//! 3D camera geometry, ray casting, and bounding-box intersection utilities.

use crate::app::OctantApp;
use crate::plots::PlotType;
use egui::{Pos2, Rect};

/// A 3D ray with origin and normalized direction.
#[derive(Clone, Copy, Debug)]
pub struct Ray3D {
    pub origin: [f32; 3],
    pub dir: [f32; 3],
}

/// Perspective 3D camera model for ray casting and forward point projection.
#[derive(Clone, Copy, Debug)]
pub struct Camera3D {
    pub rect: Rect,
    pub screen_aspect: f32,
    pub cam_dist: f32,
    pub fov_scale: f32,
    pub cx: f32,
    pub sx: f32,
    pub cy: f32,
    pub sy: f32,
}

impl Camera3D {
    pub fn from_app(app: &OctantApp, rect: Rect) -> Self {
        let screen_aspect = (rect.width() / rect.height().max(1.0)).max(0.01);
        let min_zoom = if app.active_plot_type == PlotType::Sphere {
            1.1
        } else {
            0.1
        };
        let cam_dist = app.sphere_zoom.clamp(min_zoom, 10.0);
        let fov_scale = 1.6_f32;
        let cx = app.sphere_rotation_x.cos();
        let sx = app.sphere_rotation_x.sin();
        let cy = app.sphere_rotation_y.cos();
        let sy = app.sphere_rotation_y.sin();

        Self {
            rect,
            screen_aspect,
            cam_dist,
            fov_scale,
            cx,
            sx,
            cy,
            sy,
        }
    }

    /// Casts a ray from screen coordinates returning `(view_space_ray, world_space_ray)`.
    pub fn cast_ray(&self, screen_pos: Pos2) -> (Ray3D, Ray3D) {
        let clip_x = (screen_pos.x - self.rect.center().x) / (0.5 * self.rect.width().max(1.0));
        let clip_y = -(screen_pos.y - self.rect.center().y) / (0.5 * self.rect.height().max(1.0));

        let dir_x = clip_x * self.screen_aspect / self.fov_scale;
        let dir_y = clip_y / self.fov_scale;
        let dir_z = -1.0_f32;

        let inv_len = 1.0 / (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).sqrt();
        let dx = dir_x * inv_len;
        let dy = dir_y * inv_len;
        let dz = dir_z * inv_len;

        let view_ray = Ray3D {
            origin: [0.0, 0.0, self.cam_dist],
            dir: [dx, dy, dz],
        };

        let o1_y = self.sx * self.cam_dist;
        let o1_z = self.cx * self.cam_dist;

        let d1_x = dx;
        let d1_y = self.cx * dy + self.sx * dz;
        let d1_z = -self.sx * dy + self.cx * dz;

        let o_world_x = -self.sy * o1_z;
        let o_world_y = o1_y;
        let o_world_z = self.cy * o1_z;

        let d_world_x = self.cy * d1_x - self.sy * d1_z;
        let d_world_y = d1_y;
        let d_world_z = self.sy * d1_x + self.cy * d1_z;

        let world_ray = Ray3D {
            origin: [o_world_x, o_world_y, o_world_z],
            dir: [d_world_x, d_world_y, d_world_z],
        };

        (view_ray, world_ray)
    }

    /// Forward projects a 3D model space point to screen coordinates.
    pub fn project_point(&self, pt: [f32; 3]) -> Option<Pos2> {
        let p_y_rot_x = self.cy * pt[0] + self.sy * pt[2];
        let p_y_rot_y = pt[1];
        let p_y_rot_z = -self.sy * pt[0] + self.cy * pt[2];

        let p_rot_x = p_y_rot_x;
        let p_rot_y = self.cx * p_y_rot_y - self.sx * p_y_rot_z;
        let p_rot_z = self.sx * p_y_rot_y + self.cx * p_y_rot_z;

        let dist_c = self.cam_dist - p_rot_z;
        if dist_c > 0.1 && p_rot_z < self.cam_dist {
            let pr_x = (p_rot_x * self.fov_scale) / (self.screen_aspect * dist_c);
            let pr_y = (p_rot_y * self.fov_scale) / dist_c;
            let target_x = self.rect.center().x + pr_x * (0.5 * self.rect.width());
            let target_y = self.rect.center().y - pr_y * (0.5 * self.rect.height());
            Some(Pos2::new(target_x, target_y))
        } else {
            None
        }
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
