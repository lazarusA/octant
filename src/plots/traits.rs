//! Core traits and parameter types for extensible plot visualization renderers.

use super::common::PlotColorParams;
use crate::data::RenderData;

/// Standard rendering parameter bundle passed to plot renderers during paint passes.
#[derive(Debug, Clone, Default)]
pub struct PlotRenderParams {
    pub color: PlotColorParams,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub aspect_scale: [f32; 2],
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub displacement_strength: f32,
    pub mode: u32,
    pub screen_aspect: f32,
    pub opacity: f32,
    pub step_count: u32,
    pub algorithm: u32,
    pub isovalue: f32,
    pub isorange: f32,
    pub attenuation: f32,
    pub transparency: bool,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
    pub point_size: f32,
}

/// Hover sample hit-test result returned by a plot renderer.
#[derive(Debug, Clone)]
pub struct HoverSample {
    pub cell_x: usize,
    pub cell_y: usize,
    pub value: f32,
    pub coord_lon_lat: Option<(f64, f64)>,
    pub world_pos: Option<[f32; 3]>,
}

/// Core trait representing an extensible visualization plot renderer in Octant.
/// Implementing this trait allows a visualization type to be registered and rendered
/// without altering application state dispatching or canvas paint loops.
pub trait PlotRenderer: Send + Sync {
    /// Updates GPU storage/vertex buffers when data or dimensions change.
    fn update_data(&self, queue: &wgpu::Queue, data: &RenderData);

    /// Submits the plot-specific egui-wgpu paint callback to the egui painter.
    fn paint(&self, ui: &mut egui::Ui, rect: egui::Rect, params: &PlotRenderParams);

    /// Performs CPU-side raycasting or cell index lookup for hover inspection.
    fn inspect_hover(
        &self,
        pointer_pos: egui::Pos2,
        rect: egui::Rect,
        data: &RenderData,
        params: &PlotRenderParams,
    ) -> Option<HoverSample>;
}
