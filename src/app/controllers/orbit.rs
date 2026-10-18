//! 3D Orbit camera plot controller.

use super::traits::PlotController;
use crate::app::state::NavigationState;
use crate::plots::PlotType;
use crate::plots::camera::{DEFAULT_DISTANCE, MAX_DISTANCE, OrbitCamera, TARGET_LIMIT};

/// Radians turned per screen point dragged, at the default view.
const TURN_SPEED: f32 = 0.008;
/// The sphere's default height above its unit globe, where dragging turns
/// it at `TURN_SPEED`.
const SPHERE_HEIGHT: f32 = DEFAULT_DISTANCE - 1.0;

/// Controller for 3D orbiting camera plots (Surface, Sphere, Volume, PointCloud).
///
/// Dragging orbits the camera about its target, panning moves the target
/// across the view and zooming moves the camera toward what is under the
/// pointer. The sphere keeps looking at its center: panning turns the globe
/// under the pointer and zooming changes the height above it, slowing the
/// turns as the camera comes closer.
pub struct Orbit3DController {
    plot_type: PlotType,
}

impl Orbit3DController {
    /// Constructs an orbit controller for the given 3D plot type.
    pub const fn new(plot_type: PlotType) -> Self {
        Self { plot_type }
    }

    fn is_sphere(&self) -> bool {
        self.plot_type == PlotType::Sphere
    }

    /// The camera as drawn on canvas `rect`.
    fn camera(&self, nav: &NavigationState, rect: egui::Rect) -> OrbitCamera {
        let aspect = rect.width() / rect.height().max(1.0);
        nav.orbit_camera(self.plot_type, aspect)
    }

    /// The camera's distance, within this plot's limits.
    fn distance(&self, nav: &NavigationState) -> f32 {
        nav.sphere_zoom.clamp(self.min_zoom(), MAX_DISTANCE)
    }

    /// Radians turned per screen point: slower as the sphere's camera comes
    /// closer to the globe, so what is under the pointer keeps up with it.
    fn turn_speed(&self, nav: &NavigationState) -> f32 {
        if self.is_sphere() {
            TURN_SPEED * ((self.distance(nav) - 1.0) / SPHERE_HEIGHT).min(1.0)
        } else {
            TURN_SPEED
        }
    }
}

impl PlotController for Orbit3DController {
    fn plot_type(&self) -> PlotType {
        self.plot_type
    }

    fn reset_view(&self, nav: &mut NavigationState) {
        nav.reset_3d_camera();
    }

    fn handle_drag(&self, nav: &mut NavigationState, delta: egui::Vec2) {
        let speed = self.turn_speed(nav);
        nav.sphere_rotation_y += delta.x * speed;
        nav.sphere_rotation_x = (nav.sphere_rotation_x + delta.y * speed).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.05,
            std::f32::consts::FRAC_PI_2 - 0.05,
        );
    }

    fn handle_pan(&self, nav: &mut NavigationState, delta: egui::Vec2, rect: egui::Rect) {
        if self.is_sphere() {
            self.handle_drag(nav, delta);
            return;
        }
        let offset = self
            .camera(nav, rect)
            .screen_offset([delta.x, delta.y], rect.height());
        for (t, o) in nav.orbit_target.iter_mut().zip(offset) {
            *t = (*t - o).clamp(-TARGET_LIMIT, TARGET_LIMIT);
        }
    }

    fn handle_zoom(
        &self,
        nav: &mut NavigationState,
        factor: f32,
        anchor: egui::Pos2,
        rect: egui::Rect,
    ) {
        if !(factor.is_finite() && factor > 0.0) {
            return;
        }
        if self.is_sphere() {
            // The height above the globe, divided by `factor`.
            let height = (self.distance(nav) - 1.0) / factor;
            nav.sphere_zoom = (1.0 + height).clamp(self.min_zoom(), MAX_DISTANCE);
            return;
        }
        let camera = self.camera(nav, rect);
        let distance = (camera.distance / factor).clamp(self.min_zoom(), MAX_DISTANCE);
        // The point under `anchor` at the target's depth stays under it: the
        // target moves toward it as the camera closes in.
        let from_center = anchor - rect.center();
        let under = camera.screen_offset([from_center.x, from_center.y], rect.height());
        let moved = 1.0 - distance / camera.distance;
        for (t, u) in nav.orbit_target.iter_mut().zip(under) {
            *t = (*t + u * moved).clamp(-TARGET_LIMIT, TARGET_LIMIT);
        }
        nav.sphere_zoom = distance;
    }
}
