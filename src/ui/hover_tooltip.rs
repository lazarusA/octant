use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::plots::PlotType;
use egui::{Pos2, Rect, Stroke};

/// A 3D ray with origin and normalized direction
#[derive(Clone, Copy, Debug)]
pub struct Ray3D {
    pub origin: [f32; 3],
    pub dir: [f32; 3],
}

/// Perspective 3D Camera encapsulating view parameters, ray casting, and forward projection
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
        let cam_dist = app.sphere_zoom.clamp(1.1, 10.0);
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

    /// Casts a ray from screen coordinates returning `(view_space_ray, world_space_ray)`
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

        // Transform ray into Model World Space
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

    /// Forward projects a 3D model point to screen coordinates
    pub fn project_point(&self, pt: [f32; 3]) -> Option<Pos2> {
        // Rotate around Y
        let p_y_rot_x = self.cy * pt[0] + self.sy * pt[2];
        let p_y_rot_y = pt[1];
        let p_y_rot_z = -self.sy * pt[0] + self.cy * pt[2];

        // Rotate around X
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

/// Standard AABB Ray-Box intersection using the slab method
pub fn intersect_aabb(ray: &Ray3D, min_b: [f32; 3], max_b: [f32; 3]) -> Option<(f32, f32)> {
    let inv_dx = if ray.dir[0].abs() > 1e-6 { 1.0 / ray.dir[0] } else { 1e6 };
    let inv_dy = if ray.dir[1].abs() > 1e-6 { 1.0 / ray.dir[1] } else { 1e6 };
    let inv_dz = if ray.dir[2].abs() > 1e-6 { 1.0 / ray.dir[2] } else { 1e6 };

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

/// Helper for volumetric sampling, circular dataset shifts, and 3D ray marching with hole penetration
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
    /// `is_half_scale` is true for Volume bounds $[-0.5\times aspect, +0.5\times aspect]$
    /// and false for PointCloud bounds $[-aspect, +aspect]$.
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
        let mut max_val = -1e30_f32;

        for i in 0..num_steps {
            let t = t_start + (i as f32 + 0.5) * dt;
            let px_world = ray.origin[0] + t * ray.dir[0];
            let py_world = ray.origin[1] + t * ray.dir[1];
            let pz_world = ray.origin[2] + t * ray.dir[2];

            let (u, v, w) = if is_half_scale {
                let u = ((px_world / aspect_x.max(1e-4)) + 0.5).clamp(0.0, 1.0);
                let v = ((py_world / aspect_y.max(1e-4)) + 0.5).clamp(0.0, 1.0);
                let w = ((pz_world / aspect_z.max(1e-4)) + 0.5).clamp(0.0, 1.0);
                (u, 1.0 - v, w)
            } else {
                let u = ((px_world / aspect_x.max(1e-4)) + 1.0) * 0.5;
                let v = (1.0 - (py_world / aspect_y.max(1e-4))) * 0.5;
                let w = ((pz_world / aspect_z.max(1e-4)) + 1.0) * 0.5;
                (u, v, w)
            };

            if u >= 0.0 && u < 1.0 && v >= 0.0 && v < 1.0 && w >= 0.0 && w < 1.0 {
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

                if is_half_scale && app.active_plot_type == PlotType::Volume && app.volume_algorithm == 1 {
                    // Isosurface mode
                    if !is_nan && (raw_val - app.volume_isovalue).abs() <= app.volume_isorange {
                        hit_point = Some((cx, cy, cz, raw_val));
                        break;
                    }
                } else if is_half_scale && app.active_plot_type == PlotType::Volume && app.volume_algorithm == 2 {
                    // MIP mode
                    if !is_nan && raw_val > max_val {
                        let is_visible = app.use_highclip || raw_val <= app.color_range_max;
                        if is_visible {
                            max_val = raw_val;
                            max_intensity_hit = Some((cx, cy, cz, raw_val));
                        }
                    }
                } else if self.is_visible(app, raw_val) {
                    hit_point = Some((cx, cy, cz, raw_val));
                    break;
                }
            }
        }

        if is_half_scale && app.active_plot_type == PlotType::Volume && app.volume_algorithm == 2 {
            max_intensity_hit.or(hit_point)
        } else {
            hit_point
        }
    }
}

/// 2D Viewport Transformation Helper (Aspect-scaling, Zoom, Pan)
#[derive(Clone, Copy, Debug)]
pub struct Transform2D {
    pub rect: Rect,
    pub aspect_scale_x: f32,
    pub aspect_scale_y: f32,
    pub zoom: f32,
    pub gpu_pan_x: f32,
    pub gpu_pan_y: f32,
}

impl Transform2D {
    pub fn from_app(app: &OctantApp, rect: Rect, matrix: &MatrixData) -> Self {
        let (aspect_scale_x, aspect_scale_y) = if app.enforce_data_aspect_ratio {
            let data_aspect = (matrix.width as f32 / matrix.height.max(1) as f32).max(0.001);
            let canvas_aspect = rect.width() / rect.height().max(1.0);
            if canvas_aspect > data_aspect {
                (data_aspect / canvas_aspect, 1.0)
            } else {
                (1.0, canvas_aspect / data_aspect)
            }
        } else {
            (1.0, 1.0)
        };

        let zoom = app.heatmap_zoom;
        let pan = app.heatmap_pan;
        let gpu_pan_x = pan.x / (0.5 * rect.width().max(1.0));
        let gpu_pan_y = -pan.y / (0.5 * rect.height().max(1.0));

        Self {
            rect,
            aspect_scale_x,
            aspect_scale_y,
            zoom,
            gpu_pan_x,
            gpu_pan_y,
        }
    }

    pub fn screen_to_norm(&self, screen_pos: Pos2) -> (f32, f32) {
        let ndc_x = ((screen_pos.x - self.rect.min.x) / self.rect.width().max(1.0)) * 2.0 - 1.0;
        let unpanned_x = (ndc_x - self.gpu_pan_x) / self.zoom.max(0.01);
        let unscaled_x = unpanned_x / self.aspect_scale_x.max(0.001);
        let nx = ((unscaled_x + 1.0) / 2.0).clamp(0.0, 1.0);

        let ndc_y = 1.0 - ((screen_pos.y - self.rect.min.y) / self.rect.height().max(1.0)) * 2.0;
        let unpanned_y = (ndc_y - self.gpu_pan_y) / self.zoom.max(0.01);
        let unscaled_y = unpanned_y / self.aspect_scale_y.max(0.001);
        let ny = ((1.0 - unscaled_y) / 2.0).clamp(0.0, 1.0);

        (nx, ny)
    }

    pub fn norm_to_screen(&self, u: f32, v: f32) -> Pos2 {
        let unscaled_x = u * 2.0 - 1.0;
        let unpanned_x = unscaled_x * self.aspect_scale_x.max(0.001);
        let ndc_x = unpanned_x * self.zoom.max(0.01) + self.gpu_pan_x;
        let target_x = self.rect.min.x + ((ndc_x + 1.0) * 0.5) * self.rect.width();

        let unscaled_y = 1.0 - v * 2.0;
        let unpanned_y = unscaled_y * self.aspect_scale_y.max(0.001);
        let ndc_y = unpanned_y * self.zoom.max(0.01) + self.gpu_pan_y;
        let target_y = self.rect.min.y + ((1.0 - ndc_y) * 0.5) * self.rect.height();

        Pos2::new(target_x, target_y)
    }
}

/// Renders the reticle marker dot, calculates tooltip anchor, and draws the leader elbow line
pub fn draw_leader_callout(
    painter: &egui::Painter,
    ctx: &egui::Context,
    target_pos: Pos2,
    tooltip_rect: Rect,
) {
    let visuals = &ctx.style_of(ctx.theme()).visuals;
    let strong_color = visuals.strong_text_color();
    let text_color = visuals.text_color();
    let line_color = visuals.widgets.noninteractive.fg_stroke.color;

    // 1. Target reticle marker dot
    painter.circle_filled(target_pos, 7.0, text_color.linear_multiply(0.12));
    painter.circle_filled(target_pos, 4.5, text_color.linear_multiply(0.25));
    painter.circle_stroke(target_pos, 3.5, Stroke::new(1.2, strong_color));
    painter.circle_filled(target_pos, 1.8, strong_color);

    // 2. Compute anchor on tooltip box
    let box_anchor = if tooltip_rect.min.x >= target_pos.x {
        Pos2::new(tooltip_rect.min.x, (tooltip_rect.min.y + 16.0).min(tooltip_rect.max.y - 6.0))
    } else if tooltip_rect.max.x <= target_pos.x {
        Pos2::new(tooltip_rect.max.x, (tooltip_rect.min.y + 16.0).min(tooltip_rect.max.y - 6.0))
    } else if tooltip_rect.min.y >= target_pos.y {
        Pos2::new(target_pos.x, tooltip_rect.min.y)
    } else {
        Pos2::new(target_pos.x, tooltip_rect.max.y)
    };

    // 3. Elbow connector line
    let elbow = Pos2::new(target_pos.x, box_anchor.y);
    let leader_stroke = Stroke::new(1.2, line_color.linear_multiply(0.85));

    painter.line_segment([target_pos, elbow], leader_stroke);
    painter.line_segment([elbow, box_anchor], leader_stroke);

    // 4. Subtle junction dot at the elbow vertex
    painter.circle_filled(elbow, 1.5, strong_color.linear_multiply(0.8));
}

pub fn show_hover_tooltip(
    app: &OctantApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    response: &egui::Response,
    rect: Rect,
) {
    let hover_pos = match response.hover_pos() {
        Some(pos) => pos,
        None => return,
    };

    let matrix = match &app.matrix_data {
        Some(m) if m.width > 0 && m.height > 0 && !m.values.is_empty() => m,
        _ => return,
    };

    let camera = Camera3D::from_app(app, rect);
    let sampler = VolumeSampler::from_app(app, matrix);
    let transform_2d = Transform2D::from_app(app, rect, matrix);

    // 1. Calculate Normalized Coordinates (norm_x, norm_y) considering 2D vs 3D Plot Mode
    let (norm_x, norm_y, is_valid_hit, geo_coords, point_3d_hit) = match app.active_plot_type {
        PlotType::Sphere => {
            // 3D Globe Projection Perspective Raycast supporting all Sphere Modes (Smooth, Steps, Lego)
            let (view_ray, _) = camera.cast_ray(hover_pos);
            let dx = view_ray.dir[0];
            let dy = view_ray.dir[1];
            let dz = view_ray.dir[2];

            let max_r = if app.sphere_mode > 0 {
                1.0 + 0.4 * app.sphere_displacement_strength
            } else {
                1.0
            };

            let b = camera.cam_dist * dz;
            let c_max = camera.cam_dist * camera.cam_dist - max_r * max_r;
            let discr_max = b * b - c_max;

            if discr_max < 0.0 {
                return;
            }

            let mut r = 1.0_f32;
            let c = camera.cam_dist * camera.cam_dist - r * r;
            let discr = b * b - c;
            let t = if discr >= 0.0 {
                -b - discr.sqrt()
            } else {
                -b - discr_max.sqrt()
            };

            let pos_rot_x = t * dx;
            let pos_rot_y = t * dy;
            let pos_rot_z = camera.cam_dist + t * dz;

            // Inverse rotate around X by -rx
            let pos_y_rot_x = pos_rot_x;
            let pos_y_rot_y = camera.cx * pos_rot_y + camera.sx * pos_rot_z;
            let pos_y_rot_z = -camera.sx * pos_rot_y + camera.cx * pos_rot_z;

            // Inverse rotate around Y by -ry
            let pos_3d_x = camera.cy * pos_y_rot_x - camera.sy * pos_y_rot_z;
            let pos_3d_y = pos_y_rot_y;
            let pos_3d_z = camera.sy * pos_y_rot_x + camera.cy * pos_y_rot_z;

            let lat_rad = (pos_3d_y / r).clamp(-1.0, 1.0).asin();
            let lon_rad = pos_3d_x.atan2(pos_3d_z);

            let u = (lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
            let v = 0.5 - (lat_rad / std::f32::consts::PI);

            let mut nx = u.clamp(0.0, 1.0);
            let mut ny = v.clamp(0.0, 1.0);

            if app.sphere_mode > 0 {
                let px = ((nx * matrix.width as f32).floor() as usize).min(matrix.width.saturating_sub(1));
                let py = ((ny * matrix.height as f32).floor() as usize).min(matrix.height.saturating_sub(1));
                let cell_val = matrix.values.get(py * matrix.width + px).copied().unwrap_or(f32::NAN);
                let dr = get_normalized_radial_dr(app, cell_val);
                r = 1.0 + dr;

                let c_ref = camera.cam_dist * camera.cam_dist - r * r;
                let discr_ref = b * b - c_ref;
                if discr_ref >= 0.0 {
                    let t_ref = -b - discr_ref.sqrt();
                    let pr_x = t_ref * dx;
                    let pr_y = t_ref * dy;
                    let pr_z = camera.cam_dist + t_ref * dz;

                    let py_x = pr_x;
                    let py_y = camera.cx * pr_y + camera.sx * pr_z;
                    let py_z = -camera.sx * pr_y + camera.cx * pr_z;

                    let p3_x = camera.cy * py_x - camera.sy * py_z;
                    let p3_y = py_y;
                    let p3_z = camera.sy * py_x + camera.cy * py_z;

                    let l_rad = (p3_y / r).clamp(-1.0, 1.0).asin();
                    let o_rad = p3_x.atan2(p3_z);
                    nx = ((o_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)).clamp(0.0, 1.0);
                    ny = (0.5 - (l_rad / std::f32::consts::PI)).clamp(0.0, 1.0);
                }
            }

            let lat_deg = (0.5 - ny) * 180.0;
            let lon_deg = (nx - 0.5) * 360.0;

            (nx, ny, true, Some((lat_deg, lon_deg)), None)
        }
        PlotType::Surface => {
            // 3D Surface Projection Perspective Raycast supporting all Surface Modes (Terrain, Steps, Lego)
            let (_, world_ray) = camera.cast_ray(hover_pos);
            let data_aspect = (matrix.width as f32 / matrix.height.max(1) as f32).max(0.1);

            if world_ray.dir[1].abs() < 1e-5 {
                return;
            }

            let t0 = -world_ray.origin[1] / world_ray.dir[1];
            if t0 <= 0.0 {
                return;
            }

            let hit_x = world_ray.origin[0] + t0 * world_ray.dir[0];
            let hit_z = world_ray.origin[2] + t0 * world_ray.dir[2];

            let mut u = ((hit_x / data_aspect) + 1.0) * 0.5;
            let mut v = (hit_z + 1.0) * 0.5;

            if u < -0.1 || u > 1.1 || v < -0.1 || v > 1.1 {
                return;
            }

            let px = ((u.clamp(0.0, 1.0) * matrix.width as f32).floor() as usize).min(matrix.width.saturating_sub(1));
            let py = ((v.clamp(0.0, 1.0) * matrix.height as f32).floor() as usize).min(matrix.height.saturating_sub(1));
            let cell_val = matrix.values.get(py * matrix.width + px).copied().unwrap_or(f32::NAN);
            let h = get_normalized_surface_height(app, cell_val);
            let target_h = if app.surface_mode == 2 { h.max(0.0) } else { h };

            let t_ref = (target_h - world_ray.origin[1]) / world_ray.dir[1];
            if t_ref > 0.0 {
                let ref_x = world_ray.origin[0] + t_ref * world_ray.dir[0];
                let ref_z = world_ray.origin[2] + t_ref * world_ray.dir[2];
                let u_ref = ((ref_x / data_aspect) + 1.0) * 0.5;
                let v_ref = (ref_z + 1.0) * 0.5;
                if u_ref >= -0.05 && u_ref <= 1.05 && v_ref >= -0.05 && v_ref <= 1.05 {
                    u = u_ref;
                    v = v_ref;
                }
            }

            let nx = u.clamp(0.0, 1.0);
            let ny = v.clamp(0.0, 1.0);

            (nx, ny, true, None, None)
        }
        PlotType::PointCloud => {
            // 3D Point Cloud Volumetric Ray Marching
            let (_, world_ray) = camera.cast_ray(hover_pos);
            let aspects = app.get_3d_aspect_ratio();

            let (hit_x, hit_y, hit_z, hit_val) = match sampler.march_ray(app, &world_ray, aspects, false) {
                Some(hit) => hit,
                None => return,
            };

            let nx = (hit_x as f32 + 0.5) / sampler.width as f32;
            let ny = (hit_y as f32 + 0.5) / sampler.height as f32;

            (nx, ny, true, None, Some((hit_x, hit_y, hit_z, hit_val)))
        }
        PlotType::Volume => {
            // 3D Volume Volumetric Ray Marching
            let (_, world_ray) = camera.cast_ray(hover_pos);
            let aspects = app.get_3d_aspect_ratio();

            let (hit_x, hit_y, hit_z, hit_val) = match sampler.march_ray(app, &world_ray, aspects, true) {
                Some(hit) => hit,
                None => return,
            };

            let nx = (hit_x as f32 + 0.5) / sampler.width as f32;
            let ny = (hit_y as f32 + 0.5) / sampler.height as f32;

            (nx, ny, true, None, Some((hit_x, hit_y, hit_z, hit_val)))
        }
        PlotType::Line => {
            // 1D Line Plot Direct Inverse Mapping matching shader vertex transformation (zero gap)
            let is_inside = rect.contains(hover_pos);
            let zoom = app.line_zoom;
            let gpu_pan_x = app.line_pan.x / (0.5 * rect.width().max(1.0));
            let gpu_pan_y = -app.line_pan.y / (0.5 * rect.height().max(1.0));

            let ndc_x = ((hover_pos.x - rect.min.x) / rect.width().max(1.0)) * 2.0 - 1.0;
            let unpanned_x = (ndc_x - gpu_pan_x) / zoom.max(0.01);
            let nx = ((unpanned_x + 1.0) / 2.0).clamp(0.0, 1.0);

            let ndc_y = 1.0 - ((hover_pos.y - rect.min.y) / rect.height().max(1.0)) * 2.0;
            let unpanned_y = (ndc_y - gpu_pan_y) / zoom.max(0.01);
            let ny = ((1.0 - unpanned_y) / 2.0).clamp(0.0, 1.0);

            (nx, ny, is_inside, None, None)
        }
        _ => {
            // 2D Heatmap Direct Mapping within canvas_rect
            let (nx, ny) = transform_2d.screen_to_norm(hover_pos);
            let is_inside = rect.contains(hover_pos);
            (nx, ny, is_inside, None, None)
        }
    };

    if !is_valid_hit {
        return;
    }

    // 2. Metadata lookup (variable, units, dimension names)
    let meta = app
        .plotted_dataset_metadata
        .as_ref()
        .or(app.active_dataset_metadata.as_ref());

    let var = meta.and_then(|m| {
        m.variables
            .get(app.plotted_variable_idx)
            .or_else(|| m.variables.get(app.selected_variable_idx))
            .or_else(|| m.variables.first())
    });

    let var_name = var
        .map(|v| v.name.clone())
        .unwrap_or_else(|| "variable".to_string());

    let units_str = var
        .and_then(|v| {
            v.units
                .as_deref()
                .or(v.attributes.get("units").map(|s| s.as_str()))
        })
        .map(|u| {
            let clean = u.trim();
            if clean.is_empty() || clean == "1" || clean == "none" || clean == "dimensionless" {
                String::new()
            } else {
                format!("\u{00A0}{}", clean)
            }
        })
        .unwrap_or_default();

    // 3. Coordinate String Formatting & Series Detection
    let (raw_val, dim_entries, px, py) = if app.active_plot_type == PlotType::Line {
        let (profile_values, profile_length, line_count) = app.get_line_profile_payload();
        let prof_len = profile_length as usize;
        let l_count = line_count as usize;

        let sample_idx = if prof_len > 1 {
            ((norm_x * (prof_len - 1) as f32) + 0.5) as usize
        } else {
            0
        }
        .min(prof_len.saturating_sub(1));

        let cmin = app.color_range_min;
        let cmax = app.color_range_max;
        let range = (cmax - cmin).max(1e-6);

        // Find the closest line series to the cursor's Y position
        let mut best_line_idx = 0usize;
        let mut best_dist = f32::INFINITY;
        let mut best_val = f32::NAN;

        if l_count > 0 {
            for line_idx in 0..l_count {
                let idx = line_idx * prof_len + sample_idx;
                if let Some(&v) = profile_values.get(idx) && !v.is_nan() && v.is_finite() {
                    let norm_y_val = (((v - cmin) / range) * 2.0 - 1.0).clamp(-1.0, 1.0);
                    let dist = (norm_y_val - (norm_y * 2.0 - 1.0)).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_line_idx = line_idx;
                        best_val = v;
                    }
                }
            }
        }

        let val = if !best_val.is_nan() {
            best_val
        } else {
            profile_values.get(sample_idx).copied().unwrap_or(f32::NAN)
        };

        let dim_name = app
            .get_spatial_dim_name(app.line_profile_dim_idx)
            .unwrap_or_else(|| match app.line_profile_dim_idx {
                2 => "z".to_string(),
                1 => "y".to_string(),
                _ => "x".to_string(),
            });

        let loc_str = format_dimension_coord(meta, &dim_name, sample_idx, prof_len, None);
        let mut entries = vec![loc_str];

        if l_count > 1 {
            // Include orthogonal dimension / series coordinate
            if let Some(v) = var {
                let (explicit_x, explicit_y, _) =
                    v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                        &app.plotted_dim_config
                    } else {
                        &app.dim_config
                    });

                let ortho_dim_name = match app.line_profile_dim_idx {
                    0 => explicit_y.and_then(|i| v.dimension_names.get(i)),
                    1 => explicit_x.and_then(|i| v.dimension_names.get(i)),
                    _ => None,
                };

                if let Some(ortho_name) = ortho_dim_name {
                    let ortho_str =
                        format_dimension_coord(meta, ortho_name, best_line_idx, l_count, None);
                    entries.insert(0, ortho_str);
                } else {
                    entries.insert(
                        0,
                        format!("series:\u{00A0}{}/{}", best_line_idx + 1, l_count),
                    );
                }
            } else {
                entries.insert(
                    0,
                    format!("series:\u{00A0}{}/{}", best_line_idx + 1, l_count),
                );
            }
        }

        (val, entries, sample_idx, best_line_idx)
    } else if let Some((hit_x, hit_y, hit_z, hit_val)) = point_3d_hit {
        let px = hit_x;
        let py = hit_y;
        let pz = hit_z;
        let val = hit_val;

        let entries = if let Some(v) = var {
            let (explicit_x, explicit_y, explicit_z) =
                v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                    &app.plotted_dim_config
                } else {
                    &app.dim_config
                });

            let dim_y_name = explicit_y
                .and_then(|i| v.dimension_names.get(i))
                .cloned()
                .unwrap_or_else(|| "y".to_string());

            let dim_x_name = explicit_x
                .and_then(|i| v.dimension_names.get(i))
                .cloned()
                .unwrap_or_else(|| "x".to_string());

            let loc_y = format_dimension_coord(meta, &dim_y_name, py, sampler.height, None);
            let loc_x = format_dimension_coord(meta, &dim_x_name, px, sampler.width, None);

            let mut list = vec![loc_y, loc_x];

            if sampler.depth > 1 {
                let dim_z_name = explicit_z
                    .and_then(|i| v.dimension_names.get(i))
                    .cloned()
                    .unwrap_or_else(|| "z".to_string());
                let loc_z = format_dimension_coord(meta, &dim_z_name, pz, sampler.depth, None);
                list.insert(0, loc_z);
            }

            list
        } else if sampler.depth > 1 {
            vec![
                format!("z:\u{00A0}{}/{}", pz + 1, sampler.depth),
                format!("y:\u{00A0}{}/{}", py + 1, sampler.height),
                format!("x:\u{00A0}{}/{}", px + 1, sampler.width),
            ]
        } else {
            vec![
                format!("y:\u{00A0}{}/{}", py + 1, sampler.height),
                format!("x:\u{00A0}{}/{}", px + 1, sampler.width),
            ]
        };

        (val, entries, px, py)
    } else {
        let px = if app.active_plot_type == PlotType::Sphere
            || app.active_plot_type == PlotType::Surface
            || app.active_plot_type == PlotType::PointCloud
            || app.active_plot_type == PlotType::Volume
        {
            ((norm_x * matrix.width as f32).floor() as usize).min(matrix.width.saturating_sub(1))
        } else {
            (((norm_x * (matrix.width as f32 - 1.0)) + 0.5) as usize).min(matrix.width.saturating_sub(1))
        };
        let py = if app.active_plot_type == PlotType::Sphere
            || app.active_plot_type == PlotType::Surface
            || app.active_plot_type == PlotType::PointCloud
            || app.active_plot_type == PlotType::Volume
        {
            ((norm_y * matrix.height as f32).floor() as usize).min(matrix.height.saturating_sub(1))
        } else {
            (((norm_y * (matrix.height as f32 - 1.0)) + 0.5) as usize).min(matrix.height.saturating_sub(1))
        };

        let idx = py * matrix.width + px;
        let val = matrix.values.get(idx).copied().unwrap_or(f32::NAN);

        let entries = if let Some(v) = var {
            let (explicit_x, explicit_y, _) =
                v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                    &app.plotted_dim_config
                } else {
                    &app.dim_config
                });

            let dim_y_name = explicit_y
                .and_then(|i| v.dimension_names.get(i))
                .cloned()
                .unwrap_or_else(|| "y".to_string());

            let dim_x_name = explicit_x
                .and_then(|i| v.dimension_names.get(i))
                .cloned()
                .unwrap_or_else(|| "x".to_string());

            let geo_y = geo_coords.map(|(lat, _)| lat);
            let geo_x = geo_coords.map(|(_, lon)| lon);

            let loc_y = format_dimension_coord(meta, &dim_y_name, py, matrix.height, geo_y);
            let loc_x = format_dimension_coord(meta, &dim_x_name, px, matrix.width, geo_x);

            let mut list = vec![loc_y, loc_x];

            if v.shape.len() >= 3 {
                let total_steps = app
                    .animated_dim_extent()
                    .max(v.shape.first().copied().unwrap_or(1) as usize);

                let step_dim_name = app
                    .animated_dim
                    .and_then(|i| v.dimension_names.get(i))
                    .cloned()
                    .or_else(|| v.dimension_names.first().cloned())
                    .unwrap_or_else(|| "time".to_string());

                let time_coord = meta.and_then(|m| {
                    m.dimension_coordinates
                        .get(&step_dim_name.to_lowercase())
                        .or_else(|| m.dimension_coordinates.get(&step_dim_name))
                        .and_then(|coords| {
                            if coords.len() == total_steps {
                                coords.get(app.current_timestep).cloned()
                            } else if coords.len() >= 2
                                && let (Some(first), Some(last)) = (coords.first(), coords.last())
                                && let (Ok(f_v), Ok(l_v)) =
                                    (first.parse::<f64>(), last.parse::<f64>())
                            {
                                let t = if total_steps > 1 {
                                    app.current_timestep as f64 / (total_steps - 1) as f64
                                } else {
                                    0.0
                                };
                                let val = f_v + t * (l_v - f_v);
                                Some(format!("{:.2}", val))
                            } else {
                                coords.get(app.current_timestep).cloned()
                            }
                        })
                });

                let formatted_val = if let Some(tc) = time_coord {
                    let is_raw_numeric = tc.parse::<f64>().is_ok()
                        && !tc.contains('-')
                        && !tc.contains(':')
                        && !tc.contains('/')
                        && !tc.contains('T');

                    if !is_raw_numeric && !tc.trim().is_empty() {
                        tc
                    } else {
                        crate::utils::units::format_axis_value(
                            app.current_timestep,
                            total_steps,
                            Some(&step_dim_name),
                            v.units
                                .as_deref()
                                .or(v.attributes.get("units").map(|s| s.as_str())),
                            v.time_coverage_start
                                .as_deref()
                                .or(v.attributes.get("time_coverage_start").map(|s| s.as_str())),
                            v.temporal_resolution
                                .as_deref()
                                .or(v.attributes.get("temporal_resolution").map(|s| s.as_str())),
                            Some(&app.plotted_store_target_input),
                        )
                    }
                } else {
                    crate::utils::units::format_axis_value(
                        app.current_timestep,
                        total_steps,
                        Some(&step_dim_name),
                        v.units
                            .as_deref()
                            .or(v.attributes.get("units").map(|s| s.as_str())),
                        v.time_coverage_start
                            .as_deref()
                            .or(v.attributes.get("time_coverage_start").map(|s| s.as_str())),
                        v.temporal_resolution
                            .as_deref()
                            .or(v.attributes.get("temporal_resolution").map(|s| s.as_str())),
                        Some(&app.plotted_store_target_input),
                    )
                };

                let loc_t = format!("{}:\u{00A0}{}", step_dim_name, formatted_val);
                list.insert(0, loc_t);
            }

            list
        } else {
            vec![
                format!("y:\u{00A0}{}/{}", py + 1, matrix.height),
                format!("x:\u{00A0}{}/{}", px + 1, matrix.width),
            ]
        };

        (val, entries, px, py)
    };

    // 4. Draw Glowing Reticle Marker Dot & Guide Lines on 1D Line Plot
    if app.active_plot_type == PlotType::Line {
        let (_, profile_length, _) = app.get_line_profile_payload();
        let prof_len = profile_length as usize;
        let norm_x_step = if prof_len > 1 {
            px as f32 / (prof_len - 1) as f32
        } else {
            0.5
        };

        let zoom = app.line_zoom;
        let gpu_pan_x = app.line_pan.x / (0.5 * rect.width().max(1.0));
        let gpu_pan_y = -app.line_pan.y / (0.5 * rect.height().max(1.0));

        let ndc_x = (norm_x_step * 2.0 - 1.0) * zoom + gpu_pan_x;
        let screen_dot_x = rect.min.x + (ndc_x + 1.0) * 0.5 * rect.width();

        let cmin = app.color_range_min;
        let cmax = app.color_range_max;
        let range = (cmax - cmin).max(1e-6);

        let norm_y_val = if !raw_val.is_nan() && raw_val.is_finite() {
            ((raw_val - cmin) / range).clamp(0.0, 1.0)
        } else {
            0.5
        };

        let ndc_y = (norm_y_val * 2.0 - 1.0) * zoom + gpu_pan_y;
        let screen_dot_y = rect.min.y + (1.0 - ndc_y) * 0.5 * rect.height();

        let marker_pos = Pos2::new(screen_dot_x, screen_dot_y);

        let visuals = &ctx.style_of(ctx.theme()).visuals;
        let strong_color = visuals.strong_text_color();
        let guideline_color = visuals.widgets.noninteractive.bg_stroke.color;

        let painter = ui.painter();

        // Crosshair guidelines from axis to axis
        painter.line_segment(
            [Pos2::new(rect.min.x, marker_pos.y), Pos2::new(rect.max.x, marker_pos.y)],
            Stroke::new(1.0, guideline_color),
        );
        painter.line_segment(
            [Pos2::new(marker_pos.x, rect.min.y), Pos2::new(marker_pos.x, rect.max.y)],
            Stroke::new(1.0, guideline_color),
        );

        // Reticle dot
        painter.circle_filled(marker_pos, 6.0, strong_color.linear_multiply(0.15));
        painter.circle_filled(marker_pos, 4.0, strong_color.linear_multiply(0.35));
        painter.circle_stroke(marker_pos, 3.0, Stroke::new(1.2, strong_color));
        painter.circle_filled(marker_pos, 1.5, strong_color);
    }

    // 5. Forward-Project Exact Center of Hovered Point / Face / Particle to Screen Space
    let target_pos = match app.active_plot_type {
        PlotType::Sphere => {
            let u_c = (px as f32 + 0.5) / matrix.width.max(1) as f32;
            let v_c = (py as f32 + 0.5) / matrix.height.max(1) as f32;

            let dr = get_normalized_radial_dr(app, raw_val);
            let r = 1.0 + dr;

            let lon_c = (u_c * 2.0 - 1.0) * std::f32::consts::PI;
            let lat_c = (0.5 - v_c) * std::f32::consts::PI;

            let world_x = r * lat_c.cos() * lon_c.sin();
            let world_y = r * lat_c.sin();
            let world_z = r * lat_c.cos() * lon_c.cos();

            camera.project_point([world_x, world_y, world_z])
        }
        PlotType::Surface => {
            let data_aspect = (matrix.width as f32 / matrix.height.max(1) as f32).max(0.1);
            let height = get_normalized_surface_height(app, raw_val);
            let world_y = if app.surface_mode == 2 {
                height.max(0.0) // Lego cube top face
            } else {
                height
            };

            let u_c = (px as f32 + 0.5) / matrix.width.max(1) as f32;
            let v_c = (py as f32 + 0.5) / matrix.height.max(1) as f32;

            let world_x = (2.0 * u_c - 1.0) * data_aspect;
            let world_z = 2.0 * v_c - 1.0;

            camera.project_point([world_x, world_y, world_z])
        }
        PlotType::PointCloud => {
            if let Some((hit_x, hit_y, hit_z, _)) = point_3d_hit {
                let (aspect_x, aspect_y, aspect_z) = app.get_3d_aspect_ratio();
                let u_c = (hit_x as f32 + 0.5) / sampler.width as f32;
                let v_c = (hit_y as f32 + 0.5) / sampler.height as f32;
                let w_c = (hit_z as f32 + 0.5) / sampler.depth as f32;

                let norm_x = (-1.0 + u_c * 2.0) * aspect_x;
                let norm_y = (1.0 - v_c * 2.0) * aspect_y;
                let norm_z = (-1.0 + w_c * 2.0) * aspect_z;

                camera.project_point([norm_x, norm_y, norm_z])
            } else {
                None
            }
        }
        PlotType::Volume => {
            if let Some((hit_x, hit_y, hit_z, _)) = point_3d_hit {
                let (aspect_x, aspect_y, aspect_z) = app.get_3d_aspect_ratio();
                let u_c = (hit_x as f32 + 0.5) / sampler.width as f32;
                let v_c = (hit_y as f32 + 0.5) / sampler.height as f32;
                let w_c = (hit_z as f32 + 0.5) / sampler.depth as f32;

                let pos_3d_x = (u_c - 0.5) * aspect_x;
                let pos_3d_y = (0.5 - v_c) * aspect_y;
                let pos_3d_z = (w_c - 0.5) * aspect_z;

                camera.project_point([pos_3d_x, pos_3d_y, pos_3d_z])
            } else {
                None
            }
        }
        PlotType::Heatmap | PlotType::Block => {
            let u_c = (px as f32 + 0.5) / matrix.width.max(1) as f32;
            let v_c = (py as f32 + 0.5) / matrix.height.max(1) as f32;
            Some(transform_2d.norm_to_screen(u_c, v_c))
        }
        _ => None,
    };

    // 6. Format Value String
    let val_formatted = if raw_val.is_nan() {
        "NaN".to_string()
    } else if raw_val.abs() >= 1e4 || (raw_val.abs() <= 1e-3 && raw_val != 0.0) {
        format!("{:.4e}", raw_val)
    } else {
        format!("{:.4}", raw_val)
    };

    // 7. Render Floating Glassmorphic Tooltip Window near Cursor
    let style = ctx.style_of(ctx.theme());
    let strong_text = style.visuals.strong_text_color();
    let text_color = style.visuals.text_color();

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let tooltip_w = 210.0;
    let tooltip_est_h = if dim_entries.len() > 2 { 84.0 } else { 68.0 };

    let is_connected_mode = app.active_plot_type != PlotType::Line;
    let mut tooltip_pos = if is_connected_mode {
        // Offset outward to create a clear leader line
        let offset_x = if hover_pos.x >= rect.center().x { 36.0 } else { -tooltip_w - 36.0 };
        let offset_y = if hover_pos.y >= rect.center().y { -tooltip_est_h - 18.0 } else { 18.0 };
        Pos2::new(hover_pos.x + offset_x, hover_pos.y + offset_y)
    } else {
        Pos2::new(hover_pos.x + 14.0, hover_pos.y + 14.0)
    };

    if tooltip_pos.x + tooltip_w > screen_rect.max.x - 10.0 {
        tooltip_pos.x = screen_rect.max.x - tooltip_w - 10.0;
    }
    if tooltip_pos.x < screen_rect.min.x + 10.0 {
        tooltip_pos.x = screen_rect.min.x + 10.0;
    }
    if tooltip_pos.y + tooltip_est_h > screen_rect.max.y - 10.0 {
        tooltip_pos.y = screen_rect.max.y - tooltip_est_h - 10.0;
    }
    if tooltip_pos.y < screen_rect.min.y + 10.0 {
        tooltip_pos.y = screen_rect.min.y + 10.0;
    }

    let tooltip_rect = Rect::from_min_size(tooltip_pos, egui::vec2(tooltip_w, tooltip_est_h));

    // 8. Draw Leader Elbow Connector from Pixel to Tooltip Box
    if let Some(target) = target_pos {
        draw_leader_callout(ui.painter(), ctx, target, tooltip_rect);
    }

    egui::Area::new(egui::Id::new("octant_hover_pixel_tooltip"))
        .order(egui::Order::Tooltip)
        .fixed_pos(tooltip_pos)
        .show(ctx, |ui| {
            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.set_max_width(tooltip_w - 16.0);
                    ui.vertical(|ui| {
                        // Title / Variable Name
                        ui.label(
                            egui::RichText::new(&var_name)
                                .small()
                                .strong()
                                .color(strong_text),
                        );

                        ui.add_space(2.0);

                        // Value [Units] (Prominent 15.0pt bold font)
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Val:").small().color(text_color));
                            ui.label(
                                egui::RichText::new(format!("{}{}", val_formatted, units_str))
                                    .size(15.0)
                                    .strong()
                                    .color(strong_text),
                            );
                        });

                        ui.add_space(1.0);

                        // Real Dimension Names & Coordinates with atomic break-wrap
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.spacing_mut().item_spacing.y = 1.0;
                            for (idx, entry) in dim_entries.iter().enumerate() {
                                if idx > 0 {
                                    ui.label(
                                        egui::RichText::new("•")
                                            .size(8.0)
                                            .color(text_color.linear_multiply(0.4)),
                                    );
                                }
                                ui.label(egui::RichText::new(entry).small().color(text_color));
                            }
                        });
                    });
                });
        });
}

/// Helper to format dimension coordinate values with physical units and proper cardinal degrees.
fn format_dimension_coord(
    meta: Option<&crate::data::DatasetMetadata>,
    dim_name: &str,
    idx: usize,
    total_len: usize,
    geo_fallback: Option<f32>,
) -> String {
    let clean = dim_name.trim().to_lowercase();

    if let Some(geo) = geo_fallback {
        return if clean.contains("lon") {
            let cardinal = if geo >= 0.0 { "°E" } else { "°W" };
            format!("{}:\u{00A0}{:.2}{}", dim_name, geo.abs(), cardinal)
        } else if clean.contains("lat") {
            let cardinal = if geo >= 0.0 { "°N" } else { "°S" };
            format!("{}:\u{00A0}{:.2}{}", dim_name, geo.abs(), cardinal)
        } else {
            format!("{}:\u{00A0}{:.2}°", dim_name, geo)
        };
    }

    if let Some(m) = meta {
        if let Some(coords) = m
            .dimension_coordinates
            .get(&clean)
            .or_else(|| m.dimension_coordinates.get(dim_name))
        {
            if coords.len() == total_len
                && let Some(c) = coords.get(idx)
                && !c.trim().is_empty()
            {
                return format!("{}:\u{00A0}{}", dim_name, c.replace(' ', "\u{00A0}"));
            }
            if coords.len() >= 2
                && let (Some(first), Some(last)) = (coords.first(), coords.last())
                && let (Ok(f_v), Ok(l_v)) = (first.parse::<f64>(), last.parse::<f64>())
            {
                let t = if total_len > 1 {
                    idx as f64 / (total_len - 1) as f64
                } else {
                    0.0
                };
                let val = f_v + t * (l_v - f_v);
                return if clean.contains("lon") {
                    let cardinal = if val >= 0.0 { "°E" } else { "°W" };
                    format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
                } else if clean.contains("lat") {
                    let cardinal = if val >= 0.0 { "°N" } else { "°S" };
                    format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
                } else if clean.contains("depth")
                    || clean.contains("height")
                    || clean.contains("alt")
                {
                    format!("{}:\u{00A0}{:.2}\u{00A0}m", dim_name, val)
                } else {
                    format!("{}:\u{00A0}{:.2}", dim_name, val)
                };
            }
            if let Some(first) = coords.first()
                && !first.trim().is_empty()
            {
                return format!("{}:\u{00A0}{}", dim_name, first.replace(' ', "\u{00A0}"));
            }
        }

        if let Some((min_b, max_b)) = m.get_coord_bounds(dim_name) {
            let t = if total_len > 1 {
                idx as f64 / (total_len - 1) as f64
            } else {
                0.0
            };
            let val = min_b + t * (max_b - min_b);
            return if clean.contains("lon") {
                let cardinal = if val >= 0.0 { "°E" } else { "°W" };
                format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
            } else if clean.contains("lat") {
                let cardinal = if val >= 0.0 { "°N" } else { "°S" };
                format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
            } else if clean.contains("depth") || clean.contains("height") || clean.contains("alt") {
                format!("{}:\u{00A0}{:.2}\u{00A0}m", dim_name, val)
            } else {
                format!("{}:\u{00A0}{:.2}", dim_name, val)
            };
        }
    }

    format!("{}:\u{00A0}{}", dim_name, idx)
}

/// Computes normalized radial displacement on the 3D sphere matching sphere.wgsl
fn get_normalized_radial_dr(app: &OctantApp, val: f32) -> f32 {
    if val.is_nan() || !val.is_finite() || app.sphere_mode == 0 {
        return 0.0;
    }
    let cmin = app.color_range_min;
    let cmax = app.color_range_max;
    let range = (cmax - cmin).max(1e-6);
    let disp = app.sphere_displacement_strength;

    if cmin < 0.0 && cmax > 0.0 {
        let max_abs = cmin.abs().max(cmax.abs());
        (val / max_abs).clamp(-1.0, 1.0) * 0.4 * disp
    } else {
        let norm_val = ((val - cmin) / range).clamp(0.0, 1.0);
        norm_val * 0.4 * disp
    }
}

/// Computes normalized surface height on the 3D surface mesh matching surface.wgsl
fn get_normalized_surface_height(app: &OctantApp, val: f32) -> f32 {
    if val.is_nan() || !val.is_finite() {
        return 0.0;
    }
    let cmin = app.color_range_min;
    let cmax = app.color_range_max;
    let range = (cmax - cmin).max(1e-6);
    let disp = app.surface_displacement_strength;

    let mult = match app.surface_mode {
        1 => 0.6, // Flat Steps
        _ => 0.8, // Smooth Terrain (0) and 3D Lego Cubes (2)
    };

    if cmin < 0.0 && cmax > 0.0 {
        let max_abs = cmin.abs().max(cmax.abs());
        (val / max_abs).clamp(-1.0, 1.0) * mult * disp
    } else {
        let norm_val = ((val - cmin) / range).clamp(0.0, 1.0);
        norm_val * mult * disp
    }
}
