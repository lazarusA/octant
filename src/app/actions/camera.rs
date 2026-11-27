//! The 3D camera by angle and distance (share links' `camera`): set at once,
//! since nothing resets it when a plot arrives, and clamped as dragging,
//! zooming and panning clamp it. A plot that is not 3D keeps it for the
//! next one that is.

use super::{Done, Failure, Outcome};
use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::plots::camera::{MAX_DISTANCE, PITCH_LIMIT, TARGET_LIMIT, min_distance};

impl OctantApp {
    pub(super) fn set_camera(
        &mut self,
        pitch: f32,
        yaw: f32,
        distance: f32,
        target: Option<[f32; 3]>,
    ) -> Outcome {
        let target = target.unwrap_or_default();
        if ![pitch, yaw, distance]
            .iter()
            .chain(&target)
            .all(|v| v.is_finite())
        {
            return Err(Failure::Refused);
        }
        let nav = &mut self.nav;
        nav.sphere_rotation_x = pitch.to_radians().clamp(-PITCH_LIMIT, PITCH_LIMIT);
        // Wrapped: a huge yaw would leave too few f32 steps for dragging.
        nav.sphere_rotation_y = yaw.rem_euclid(360.0).to_radians();
        // The closest any 3D plot comes; the sphere's own limit applies when
        // it draws (`NavigationState::orbit_camera`).
        nav.sphere_zoom = distance.clamp(min_distance(PlotType::Volume), MAX_DISTANCE);
        nav.orbit_target = target.map(|t| t.clamp(-TARGET_LIMIT, TARGET_LIMIT));
        nav.sphere_auto_rotate = false;
        Ok(Done::CameraSet)
    }
}
