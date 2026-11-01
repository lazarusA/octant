//! One layer's GPU paint callback for the canvas plot type.

use crate::app::OctantApp;
use crate::app::layers::Layer;
use crate::plots::PlotType;

/// Where the canvas draws this frame.
#[derive(Clone, Copy)]
pub struct CanvasView {
    pub canvas_rect: egui::Rect,
    pub plot_rect: egui::Rect,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
}

impl OctantApp {
    /// Adds `layer`'s paint callback for `plot_type`, when its renderer exists.
    pub(super) fn paint_layer(
        &self,
        ui: &egui::Ui,
        layer: &Layer,
        plot_type: PlotType,
        view: &CanvasView,
    ) {
        match plot_type {
            PlotType::Line => self.paint_line(ui, layer, view),
            PlotType::Sphere | PlotType::Surface => self.paint_mesh(ui, layer, plot_type, view),
            PlotType::Volume => self.paint_volume(ui, layer, view),
            PlotType::PointCloud => self.paint_point_cloud(ui, layer, view),
            PlotType::Heatmap => self.paint_heatmap(ui, layer, view),
        }
    }

    fn paint_line(&self, ui: &egui::Ui, layer: &Layer, view: &CanvasView) {
        let Some(line_renderer) = &layer.renderers.line else {
            return;
        };
        let color_params = self.get_color_params(layer);
        let shape = self.upload_line_payload(layer);
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.canvas_rect,
            crate::plots::LineCallback {
                renderer: line_renderer.clone(),
                color_params,
                line_color: self.plot_configs.line.line_color,
                use_custom_color: self.plot_configs.line.use_custom_color,
                show_lines: self.plot_configs.line.show_lines,
                show_points: self.plot_configs.line.show_points,
                point_size: self.plot_configs.line.point_size,
                rect: view.canvas_rect,
                shape,
                line_mode: if self.plot_configs.line.all_series {
                    1
                } else {
                    0
                },
                pan: view.pan,
                zoom: view.zoom,
            },
        );
        ui.painter().add(callback);
    }

    fn paint_mesh(&self, ui: &egui::Ui, layer: &Layer, plot_type: PlotType, view: &CanvasView) {
        let sphere = plot_type == PlotType::Sphere;
        let (renderer, cube_mode_idx) = if sphere {
            (&layer.renderers.sphere, 3)
        } else {
            (&layer.renderers.surface, 2)
        };
        let Some(renderer) = renderer else {
            return;
        };
        let aspect_ratio = crate::plots::common::compute_aspect_ratio(&view.plot_rect);
        let params = self.get_mesh_3d_uniform_params(layer, sphere, aspect_ratio);
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.plot_rect,
            crate::plots::Mesh3DCallback {
                renderer: renderer.clone(),
                params,
                cube_mode_idx,
                rect: view.plot_rect,
                transparency: self.transparency_mode(layer),
                channels: channel_draw(
                    layer,
                    &layer.renderers.channels,
                    probe_start(layer.data.composite_probe.as_ref()),
                    None,
                ),
                thinning: layer.mesh_thinning(),
            },
        );
        ui.painter().add(callback);
    }

    fn paint_volume(&self, ui: &egui::Ui, layer: &Layer, view: &CanvasView) {
        let Some(volume_renderer) = &layer.renderers.volume else {
            return;
        };
        let screen_aspect = crate::plots::common::compute_aspect_ratio(&view.plot_rect);
        let params = self.get_volume_uniform_params(layer, screen_aspect);
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.plot_rect,
            crate::plots::VolumeCallback {
                renderer: volume_renderer.clone(),
                params,
                rect: view.plot_rect,
                // Half resolution while rotating or zooming; the
                // frame after the input stops renders in full.
                scale: if self.nav.view_interacting { 0.5 } else { 1.0 },
                channels: channel_draw(
                    layer,
                    &layer.renderers.channels,
                    layer.data.channel_volume.as_ref().map(|cv| cv.c_start),
                    None,
                ),
            },
        );
        ui.painter().add(callback);
    }

    fn paint_point_cloud(&self, ui: &egui::Ui, layer: &Layer, view: &CanvasView) {
        let Some(point_cloud_renderer) = &layer.renderers.point_cloud else {
            return;
        };
        let screen_aspect = crate::plots::common::compute_aspect_ratio(&view.plot_rect);
        let params = self.get_point_cloud_uniform_params(layer, screen_aspect);
        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
            view.plot_rect,
            crate::plots::PointCloudCallback {
                renderer: point_cloud_renderer.clone(),
                params,
                rect: view.plot_rect,
                transparency: self.transparency_mode(layer),
                channels: channel_draw(
                    layer,
                    &layer.renderers.channels,
                    layer.data.channel_volume.as_ref().map(|cv| cv.c_start),
                    None,
                ),
            },
        );
        ui.painter().add(callback);
    }

    fn paint_heatmap(&self, ui: &egui::Ui, layer: &Layer, view: &CanvasView) {
        let Some(renderer) = &layer.renderers.heatmap else {
            return;
        };
        let coord_mode = layer
            .data
            .matrix
            .as_ref()
            .map_or(0, |m| m.grid.render_coord_mode());
        let labels = || match (layer.labels(), &layer.renderers.labels) {
            (None, _) => Some(None),
            (Some(style), Some(labels)) => Some(Some(crate::plots::LabelDraw {
                renderer: labels.clone(),
                outline: style.outline,
                opacity: layer.color.opacity,
            })),
            // Label ids are never drawn through a colormap.
            (Some(_), None) => None,
        };
        let Some(label_draw) = labels() else {
            return;
        };
        let color_params = self.get_color_params(layer);
        let nan = (color_params.use_nan_color == 1).then_some(color_params.nan_color);
        let channels =
            |renderer: &Option<_>, probe| channel_draw(layer, renderer, probe_start(probe), nan);
        let callback = |renderer: &std::sync::Arc<crate::plots::HeatmapRenderer>,
                        coord_mode,
                        labels,
                        channels| {
            eframe::egui_wgpu::Callback::new_paint_callback(
                view.canvas_rect,
                crate::plots::HeatmapCallback {
                    renderer: renderer.clone(),
                    color_params,
                    rect: view.canvas_rect,
                    pan: view.pan,
                    zoom: view.zoom,
                    aspect_scale: view.aspect_scale,
                    coord_mode,
                    labels,
                    channels,
                },
            )
        };
        let probe = layer.data.composite_probe.as_ref();
        let base_channels = channels(&layer.renderers.channels, probe);
        ui.painter()
            .add(callback(renderer, coord_mode, label_draw, base_channels));
        // A multiscale heatmap's finer tile, over its coarse plot.
        if let (Some(tile), Some(detail)) = (&layer.data.detail.tile, &layer.renderers.detail) {
            let tile_channels = channels(&layer.renderers.detail_channels, tile.probe.as_ref());
            ui.painter()
                .add(callback(detail, 0, labels().flatten(), tile_channels));
        }
    }
}

/// The first channel of `probe`'s window.
pub(super) fn probe_start(probe: Option<&crate::data::slicing::CompositeProbe>) -> Option<usize> {
    probe.map(|p| p.channel_span().0)
}

/// `layer`'s composite blended by `renderer` from its planes, channel
/// `c_start` first, in its style this frame; `None` when the CPU blended it.
pub(super) fn channel_draw(
    layer: &Layer,
    renderer: &Option<std::sync::Arc<crate::plots::ChannelRenderer>>,
    c_start: Option<usize>,
    nan_color: Option<[f32; 4]>,
) -> Option<crate::plots::ChannelDraw> {
    let renderer = renderer.as_ref().filter(|_| layer.composite.enabled)?;
    let c_start = c_start?;
    let params = crate::plots::ChannelParams::of(
        &layer.composite.channel_configs,
        layer.data.composite_stretch.as_ref()?,
        (c_start, renderer.planes),
        u32::try_from(renderer.plane_len).ok()?,
        nan_color,
    );
    Some(crate::plots::ChannelDraw {
        renderer: renderer.clone(),
        params,
    })
}
