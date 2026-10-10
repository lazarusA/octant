//! Viewport pan, zoom, and 3D camera navigation state.

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationState {
    pub heatmap_zoom: f32,
    pub heatmap_pan: egui::Vec2,
    pub line_zoom: f32,
    pub line_pan: egui::Vec2,
    pub sphere_rotation_y: f32,
    pub sphere_rotation_x: f32,
    pub sphere_auto_rotate: bool,
    pub sphere_zoom: f32,
    /// Whether the user is rotating or zooming this frame: volumes render
    /// at reduced resolution until it settles.
    pub view_interacting: bool,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            heatmap_zoom: 1.0,
            heatmap_pan: egui::Vec2::ZERO,
            line_zoom: 1.0,
            line_pan: egui::Vec2::ZERO,
            sphere_rotation_y: 0.0,
            sphere_rotation_x: 0.25,
            sphere_auto_rotate: false,
            sphere_zoom: 2.5,
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
        self.sphere_zoom = 2.5;
    }
}
