//! 3D volumetric raymarching for Volume and PointCloud plots.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::plots::PlotType;
use crate::ui::hover::camera::{Camera3D, Ray3D, intersect_aabb};
use egui::Pos2;

/// Helper for volumetric sampling, circular dataset shifts, and 3D ray marching.
pub struct VolumeSampler<'a> {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub values: &'a [f32],
    pub shift_x: usize,
    pub shift_y: usize,
    pub shift_z: usize,
}

impl<'a> VolumeSampler<'a> {
    pub fn from_app(app: &'a OctantApp, matrix: &'a MatrixData) -> Self {
        let (shift_x, shift_y, shift_z) = app.get_volume_shifts();
        if let Some(v) = &app.volume_data {
            Self {
                width: v.width.max(1),
                height: v.height.max(1),
                depth: v.depth.max(1),
                values: &v.values,
                shift_x: shift_x as usize,
                shift_y: shift_y as usize,
                shift_z: shift_z as usize,
            }
        } else {
            Self {
                width: matrix.width.max(1),
                height: matrix.height.max(1),
                depth: 1,
                values: &matrix.values,
                shift_x: shift_x as usize,
                shift_y: shift_y as usize,
                shift_z: shift_z as usize,
            }
        }
    }

    pub fn sample_cell(&self, cx: usize, cy: usize, cz: usize) -> f32 {
        let shifted_x = (cx + self.shift_x) % self.width;
        let shifted_y = (cy + self.shift_y) % self.height;
        let shifted_z = (cz + self.shift_z) % self.depth;
        let idx = shifted_z * (self.width * self.height) + shifted_y * self.width + shifted_x;
        self.values.get(idx).copied().unwrap_or(f32::NAN)
    }

    pub fn is_visible(&self, app: &OctantApp, val: f32) -> bool {
        let is_nan = val.is_nan() || val.abs() > 1e30;
        if is_nan {
            app.use_nan_color
        } else {
            let in_low = app.use_lowclip || val >= app.color_range_min;
            let in_high = app.use_highclip || val <= app.color_range_max;
            in_low && in_high
        }
    }

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
                    ((u * (self.width - 1) as f32).round() as usize).min(self.width - 1)
                } else {
                    ((u * self.width as f32).floor() as usize).min(self.width - 1)
                };
                let cy = if is_half_scale {
                    ((v * (self.height - 1) as f32).round() as usize).min(self.height - 1)
                } else {
                    ((v * self.height as f32).floor() as usize).min(self.height - 1)
                };
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

                if is_half_scale
                    && app.active_plot_type == PlotType::Volume
                    && app.volume_algorithm == 1
                {
                    // Isosurface mode
                    if !is_nan && (raw_val - app.volume_isovalue).abs() <= app.volume_isorange {
                        hit_point = Some((cx, cy, cz, raw_val));
                        break;
                    }
                } else if is_half_scale
                    && app.active_plot_type == PlotType::Volume
                    && app.volume_algorithm == 2
                {
                    // MIP mode
                    if !is_nan && raw_val > max_val {
                        let is_visible = app.use_highclip || raw_val <= app.color_range_max;
                        if is_visible {
                            max_val = raw_val;
                            max_intensity_hit = Some((cx, cy, cz, raw_val));
                        }
                    }
                } else if is_half_scale
                    && app.active_plot_type == PlotType::Volume
                    && app.volume_algorithm == 3
                {
                    // MinIP mode
                    if !is_nan && raw_val < min_val {
                        let is_visible = app.use_lowclip || raw_val >= app.color_range_min;
                        if is_visible {
                            min_val = raw_val;
                            min_intensity_hit = Some((cx, cy, cz, raw_val));
                        }
                    }
                } else if is_half_scale
                    && app.active_plot_type == PlotType::Volume
                    && app.volume_algorithm == 5
                {
                    // Categorical Label Isosurface mode
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

        if is_half_scale && app.active_plot_type == PlotType::Volume && app.volume_algorithm == 2 {
            max_intensity_hit.or(hit_point)
        } else if is_half_scale
            && app.active_plot_type == PlotType::Volume
            && app.volume_algorithm == 3
        {
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
