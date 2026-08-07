use crate::data::matrix_data::MatrixData;
use crate::plots::{
    LineRenderer, MatrixRenderer, PlotType, PointCloudRenderer, SphereRenderer, SurfaceRenderer,
    VolumeRenderer,
};
use std::sync::Arc;

use super::OctantApp;

impl OctantApp {
    pub fn rebuild_pipeline_with_matrix_data(&mut self, data: MatrixData) {
        // If the data is a line plot, set the line plot all series to false and the line profile dim index and slice index to 0
        if data.height == 1 {
            self.line_plot_all_series = false;
            self.line_profile_dim_idx = 0;
            self.line_profile_slice_idx = 0;
        }
        if let Some(wgpu_render_state) = &self.wgpu_render_state {
            let same_dimensions = self
                .matrix_data
                .as_ref()
                .is_some_and(|m| m.width == data.width && m.height == data.height);

            if same_dimensions
                && self.renderer.is_some()
                && self.line_renderer.is_some()
                && self.sphere_renderer.is_some()
                && self.surface_renderer.is_some()
                && self.volume_renderer.is_some()
                && self.point_cloud_renderer.is_some()
            {
                if let Some(renderer) = &self.renderer {
                    renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(line_renderer) = &self.line_renderer {
                    line_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(sphere_renderer) = &self.sphere_renderer {
                    sphere_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(surface_renderer) = &self.surface_renderer {
                    surface_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(volume_renderer) = &self.volume_renderer {
                    volume_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(point_cloud_renderer) = &self.point_cloud_renderer {
                    point_cloud_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
            } else {
                let renderer = MatrixRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width,
                    data.height,
                );
                let line_renderer = LineRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width,
                    data.height,
                );
                let sphere_renderer = SphereRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width,
                    data.height,
                );
                let surface_renderer = SurfaceRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width,
                    data.height,
                );
                let volume_renderer = VolumeRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width as u32,
                    data.height as u32,
                );
                let point_cloud_renderer = PointCloudRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width as u32,
                    data.height as u32,
                );
                self.renderer = Some(Arc::new(renderer));
                self.line_renderer = Some(Arc::new(line_renderer));
                self.sphere_renderer = Some(Arc::new(sphere_renderer));
                self.surface_renderer = Some(Arc::new(surface_renderer));
                self.volume_renderer = Some(Arc::new(volume_renderer));
                self.point_cloud_renderer = Some(Arc::new(point_cloud_renderer));

                if data.height == 1 {
                    self.active_plot_type = PlotType::Line;
                } else if self.active_plot_type == PlotType::Line {
                    self.active_plot_type = PlotType::Heatmap;
                }
            }
        }

        self.global_data_min = self.global_data_min.min(data.min_val);
        self.global_data_max = self.global_data_max.max(data.max_val);

        if !self.lock_color_bounds {
            self.color_range_min = data.min_val;
            self.color_range_max = data.max_val;
            self.volume_cmin = data.min_val;
            self.volume_cmax = data.max_val;
        }

        self.matrix_data = Some(data);
    }

    pub fn get_color_params(&self) -> crate::plots::common::PlotColorParams {
        let effective_colormap = self.preview_colormap.unwrap_or(self.active_colormap);

        let (is_cat, num_cats) = if self.is_categorical {
            if let Some(mdata) = &self.matrix_data {
                if let Some(unique) = mdata.detect_unique_values() {
                    (1, unique.len() as u32)
                } else {
                    (1, 10)
                }
            } else {
                (1, 10)
            }
        } else {
            (0, 10)
        };

        crate::plots::common::PlotColorParams {
            colormap: effective_colormap,
            cmin: self.color_range_min,
            cmax: self.color_range_max,
            use_nan_color: if self.use_nan_color { 1 } else { 0 },
            use_lowclip: if self.use_lowclip { 1 } else { 0 },
            use_highclip: if self.use_highclip { 1 } else { 0 },
            scale_type: self.active_scale_type,
            scale_param: self.scale_param,
            is_categorical: is_cat,
            num_categories: num_cats,
            _pad0: 0,
            _pad1: 0,
            nan_color: self.nan_color,
            lowclip_color: self.lowclip_color,
            highclip_color: self.highclip_color,
        }
    }

    pub fn get_3d_aspect_ratio(&self) -> (f32, f32, f32) {
        let (w, h, max_t) = self.matrix_data.as_ref().map_or((64, 64, 64), |m| {
            (m.width as u32, m.height as u32, m.max_timesteps as u32)
        });

        let (shape_d, shape_h, shape_w) = if let Some(meta) = &self.active_dataset_metadata {
            if let Some(v) = meta.variables.get(self.selected_variable_idx) {
                if v.shape.len() >= 3 {
                    (v.shape[0] as u32, v.shape[1] as u32, v.shape[2] as u32)
                } else {
                    (max_t, h, w)
                }
            } else {
                (max_t, h, w)
            }
        } else {
            (max_t, h, w)
        };

        let width = shape_w.max(w);
        let height = shape_h.max(h);
        let depth = shape_d.max(max_t);

        let max_spatial = (width.max(height)) as f32;
        let aspect_x = width as f32 / max_spatial;
        let aspect_y = height as f32 / max_spatial;
        let aspect_z = ((depth as f32 / max_spatial) * 0.12).clamp(0.4, 1.0);

        (aspect_x, aspect_y, aspect_z)
    }
}
