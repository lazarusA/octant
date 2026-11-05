//! 3D volumetric raymarching for Volume and PointCloud plots.

use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::hover::camera::{Camera3D, Ray3D, intersect_aabb};
pub use crate::ui::hover::volume_sampler::VolumeSampler;
use egui::Pos2;

impl VolumeSampler<'_> {
    /// Marches a ray through the 3D dataset.
    pub fn march_ray(
        &self,
        app: &OctantApp,
        ray: &Ray3D,
        aspects: (f32, f32, f32),
        is_half_scale: bool,
    ) -> Option<(usize, usize, usize, f32)> {
        let (aspect_x, aspect_y, aspect_z) = aspects;
        let scale = if is_half_scale { 0.5 } else { 1.0 };
        let min_b = [-scale * aspect_x, -scale * aspect_y, -scale * aspect_z];
        let max_b = [scale * aspect_x, scale * aspect_y, scale * aspect_z];

        let (t_enter, t_exit) = intersect_aabb(ray, min_b, max_b)?;
        let t_start = t_enter.max(0.0);
        let t_end = t_exit;
        let total_len = t_end - t_start;
        if total_len <= 0.0 {
            return None;
        }

        let max_dim = self.width.max(self.height).max(self.depth);
        let num_steps = (max_dim * 3).clamp(64, 512);
        let dt = total_len / num_steps as f32;

        let mut hit_point = None;
        let mut last_cell = None;
        let mut max_intensity_hit = None;
        let mut min_intensity_hit = None;
        let mut max_val = -1e30_f32;
        let mut min_val = 1e30_f32;

        let canvas_plot_type = app.effective_canvas_plot_type();
        let is_mip = is_half_scale
            && canvas_plot_type == PlotType::Volume
            && app.plot_configs.volume.algorithm == 1;
        let is_minip = is_half_scale
            && canvas_plot_type == PlotType::Volume
            && app.plot_configs.volume.algorithm == 2;
        let is_label = is_half_scale
            && canvas_plot_type == PlotType::Volume
            && app.plot_configs.volume.algorithm == 4;

        // MIP walks every voxel the ray crosses, as its shader does.
        if is_mip && !app.layers.base.composite.enabled {
            let texel = |t: f32| {
                let p = [0, 1, 2].map(|i| ray.origin[i] + t * ray.dir[i]);
                let u = p[0] / aspect_x.max(1e-4) + 0.5;
                let v = p[1] / aspect_y.max(1e-4) + 0.5;
                let w = p[2] / aspect_z.max(1e-4) + 0.5;
                [
                    u * self.width as f32,
                    (1.0 - v) * self.height as f32,
                    (1.0 - w) * self.depth as f32,
                ]
            };
            return self.mip_hit(app, texel(t_start), texel(t_end));
        }

        // Rays pick the cells of the grid drawn (a thinned cloud's).
        let (_, (gw, gh)) = self.drawn_cell(0, 0);
        for i in 0..num_steps {
            let t = t_start + (i as f32 + 0.5) * dt;
            let px_world = ray.origin[0] + t * ray.dir[0];
            let py_world = ray.origin[1] + t * ray.dir[1];
            let pz_world = ray.origin[2] + t * ray.dir[2];

            let (u, v, w) = if is_half_scale {
                let u = ((px_world / aspect_x.max(1e-4)) + 0.5).clamp(0.0, 1.0);
                let v = ((py_world / aspect_y.max(1e-4)) + 0.5).clamp(0.0, 1.0);
                let w = ((pz_world / aspect_z.max(1e-4)) + 0.5).clamp(0.0, 1.0);
                (u, 1.0 - v, 1.0 - w)
            } else {
                let u = ((px_world / aspect_x.max(1e-4)) + 1.0) * 0.5;
                let v = (1.0 - (py_world / aspect_y.max(1e-4))) * 0.5;
                let w = (1.0 - (pz_world / aspect_z.max(1e-4))) * 0.5;
                (u, v, w)
            };

            if (0.0..1.0).contains(&u) && (0.0..1.0).contains(&v) && (0.0..1.0).contains(&w) {
                let cx = if is_half_scale {
                    ((u * (gw - 1) as f32).round() as usize).min(gw - 1)
                } else {
                    ((u * gw as f32).floor() as usize).min(gw - 1)
                };
                let cy = if is_half_scale {
                    ((v * (gh - 1) as f32).round() as usize).min(gh - 1)
                } else {
                    ((v * gh as f32).floor() as usize).min(gh - 1)
                };
                let (cx, cy) = self.drawn_voxel(cx, cy);
                let cz = if is_half_scale {
                    ((w * (self.depth - 1) as f32).round() as usize).min(self.depth - 1)
                } else {
                    ((w * self.depth as f32).floor() as usize).min(self.depth - 1)
                };

                if last_cell == Some((cx, cy, cz)) {
                    continue;
                }
                last_cell = Some((cx, cy, cz));

                let raw_val = self.sample_cell(cx, cy, cz);
                let is_nan = raw_val.is_nan() || raw_val.abs() > 1e30;

                if is_mip {
                    // MIP mode
                    if !is_nan && raw_val > max_val {
                        let is_visible = app.layers.base.color.use_highclip
                            || raw_val <= app.layers.base.color.range_max;
                        if is_visible {
                            max_val = raw_val;
                            max_intensity_hit = Some((cx, cy, cz, raw_val));
                        }
                    }
                } else if is_minip {
                    // MinIP mode
                    if !is_nan && raw_val < min_val {
                        let is_visible = app.layers.base.color.use_lowclip
                            || raw_val >= app.layers.base.color.range_min;
                        if is_visible {
                            min_val = raw_val;
                            min_intensity_hit = Some((cx, cy, cz, raw_val));
                        }
                    }
                } else if is_label {
                    // Categorical Label Surface mode
                    if !is_nan && raw_val >= 0.5 {
                        hit_point = Some((cx, cy, cz, raw_val));
                        break;
                    }
                } else if self.is_visible(app, raw_val) {
                    hit_point = Some((cx, cy, cz, raw_val));
                    break;
                }
            }
        }

        if is_mip {
            max_intensity_hit.or(hit_point)
        } else if is_minip {
            min_intensity_hit.or(hit_point)
        } else {
            hit_point
        }
    }
}

/// Forward projects the 3D position of the hovered volume voxel to screen space.
pub fn volume_target_pos(
    camera: &Camera3D,
    hit: (usize, usize, usize),
    dimensions: (usize, usize, usize),
    aspects: (f32, f32, f32),
) -> Option<Pos2> {
    let (hit_x, hit_y, hit_z) = hit;
    let (dim_w, dim_h, dim_d) = dimensions;
    let (aspect_x, aspect_y, aspect_z) = aspects;

    let u_c = (hit_x as f32 + 0.5) / dim_w as f32;
    let v_c = (hit_y as f32 + 0.5) / dim_h as f32;
    let w_c = (hit_z as f32 + 0.5) / dim_d as f32;

    let pos_3d_x = (u_c - 0.5) * aspect_x;
    let pos_3d_y = (0.5 - v_c) * aspect_y;
    let pos_3d_z = (0.5 - w_c) * aspect_z;

    camera.project_point([pos_3d_x, pos_3d_y, pos_3d_z])
}
