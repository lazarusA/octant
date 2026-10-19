//! Viewport pan, zoom, and 3D camera navigation state.

use crate::plots::PlotType;
use crate::plots::camera::{self, OrbitCamera};

/// How far 2D plots zoom in: 50 screen pixels per plotted cell at most.
pub const MAX_ZOOM: f32 = 50.0;

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationState {
    pub heatmap_zoom: f32,
    pub heatmap_pan: egui::Vec2,
    /// How far the heatmap may zoom in: farther over a coarse level of a
    /// multiscale variable, so its finest level can fill the screen
    /// (`OctantApp::heatmap_max_zoom`, set every frame by `render_canvas`).
    pub heatmap_max_zoom: f32,
    pub line_zoom: f32,
    pub line_pan: egui::Vec2,
    pub sphere_rotation_y: f32,
    pub sphere_rotation_x: f32,
    pub sphere_auto_rotate: bool,
    /// The 3D camera's distance from `orbit_target`.
    pub sphere_zoom: f32,
    /// The point the 3D camera looks at and orbits, moved by panning (the
    /// sphere always looks at its center).
    pub orbit_target: [f32; 3],
    /// Whether the user is rotating or zooming this frame: volumes render
    /// at reduced resolution until it settles.
    pub view_interacting: bool,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            heatmap_zoom: 1.0,
            heatmap_pan: egui::Vec2::ZERO,
            heatmap_max_zoom: MAX_ZOOM,
            line_zoom: 1.0,
            line_pan: egui::Vec2::ZERO,
            sphere_rotation_y: 0.0,
            sphere_rotation_x: 0.25,
            sphere_auto_rotate: false,
            sphere_zoom: camera::DEFAULT_DISTANCE,
            orbit_target: [0.0; 3],
            view_interacting: false,
        }
    }
}

impl NavigationState {
    pub fn reset_heatmap(&mut self) {
        self.heatmap_zoom = 1.0;
        self.heatmap_pan = egui::Vec2::ZERO;
    }

    pub fn reset_line(&mut self) {
        self.line_zoom = 1.0;
        self.line_pan = egui::Vec2::ZERO;
    }

    pub fn reset_3d_camera(&mut self) {
        self.sphere_rotation_x = 0.25;
        self.sphere_rotation_y = 0.0;
        self.sphere_zoom = camera::DEFAULT_DISTANCE;
        self.orbit_target = [0.0; 3];
    }

    /// The camera `plot_type` draws with on a canvas `aspect` wide per unit
    /// of height.
    pub fn orbit_camera(&self, plot_type: PlotType, aspect: f32) -> OrbitCamera {
        let sphere = plot_type == PlotType::Sphere;
        OrbitCamera {
            rotation_x: self.sphere_rotation_x,
            rotation_y: self.sphere_rotation_y,
            distance: self
                .sphere_zoom
                .clamp(camera::min_distance(plot_type), camera::MAX_DISTANCE),
            target: if sphere { [0.0; 3] } else { self.orbit_target },
            aspect: aspect.max(0.01),
        }
    }
}
