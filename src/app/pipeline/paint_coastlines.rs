//! The 3D coastlines drawn over the base layer's surface or sphere, on its
//! grid, heights and composite.

use std::sync::Arc;

use crate::app::OctantApp;
use crate::plots::PlotType;

impl OctantApp {
    /// The 3D coastlines over the base layer's surface or sphere
    /// (`canvas_plot_type`), on its grid and composite.
    pub(super) fn paint_coastlines_3d(
        &self,
        ui: &egui::Ui,
        plot_rect: egui::Rect,
        canvas_plot_type: PlotType,
    ) {
        if canvas_plot_type == PlotType::Heatmap {
            return;
        }
        let Some(renderer) = self.coastline_3d_renderer.as_ref().map(Arc::clone) else {
            return;
        };
        let sphere = canvas_plot_type == PlotType::Sphere;
        let params = self.coastline_3d_params(ui, sphere, plot_rect);
        let cb = eframe::egui_wgpu::Callback::new_paint_callback(
            plot_rect,
            crate::plots::Coastline3DCallback {
                renderer,
                params,
                rect: plot_rect,
                channels: super::paint_layer::channel_draw(
                    &self.layers.base,
                    &self.layers.base.renderers.channels,
                    super::paint_layer::probe_start(self.layers.base.data.composite_probe.as_ref()),
                    None,
                ),
                thinning: self.layers.base.mesh_thinning(),
            },
        );
        ui.painter().add(cb);
    }

    /// The 3D coastlines' parameters over the base layer's surface or
    /// `sphere` in `plot_rect`: its camera, heights and grid.
    fn coastline_3d_params(
        &self,
        ui: &egui::Ui,
        sphere: bool,
        plot_rect: egui::Rect,
    ) -> crate::plots::Coastline3DParams {
        // Coastlines follow the base layer's grid.
        let mesh_params = self.get_mesh_3d_uniform_params(
            &self.layers.base,
            sphere,
            crate::plots::common::compute_aspect_ratio(&plot_rect),
        );
        let (lon_bounds, lat_bounds) = self.coastline_bounds_3d(sphere, &mesh_params);
        let line_color = self.coastline_color.unwrap_or_else(|| {
            if ui.visuals().dark_mode {
                [1.0, 1.0, 1.0, 0.75]
            } else {
                [0.15, 0.15, 0.15, 0.85]
            }
        });
        crate::plots::Coastline3DParams {
            camera: mesh_params.camera,
            displacement_strength: mesh_params.displacement_strength,
            plot_kind: u32::from(sphere),
            plot_mode: mesh_params.mode,
            line_width: self.coastline_line_width,
            coord_mode: mesh_params.coord_mode,
            crop_to_domain: if self.coastline_crop_to_data_domain {
                1
            } else {
                0
            },
            lon_bounds,
            lat_bounds,
            color: line_color,
            color_range: [mesh_params.color.cmin, mesh_params.color.cmax],
            flip_height: mesh_params.flip_height,
            composite: mesh_params.color.colormap == crate::utils::colormap::COLORMAP_RGB_COMPOSITE,
        }
    }
}
