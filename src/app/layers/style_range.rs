//! A layer's color range and scale: following the data extent, set by the
//! user, and a log scale kept off a range at or below zero.

use super::ColorStyle;
use crate::utils::colormap::ColorScale;

impl ColorStyle {
    /// Forgets the data extent seen so far, unless the range is locked.
    pub fn reset_bounds(&mut self) {
        if !self.lock_bounds {
            self.global_min = f32::MAX;
            self.global_max = f32::MIN;
        }
    }

    /// Starts the data extent and range over at a new variable's `min`..`max`
    /// (non-finite ends, as in an empty volume, keep the old ones) and unlocks
    /// the range.
    pub fn reset_to_extent(&mut self, min: f32, max: f32) {
        if min.is_finite() {
            self.global_min = min;
            self.range_min = min;
        }
        if max.is_finite() {
            self.global_max = max;
            self.range_max = max;
        }
        self.lock_bounds = false;
    }

    /// Sets the range, `locked` as a range the user typed is. A locked range
    /// a log scale cannot draw turns it linear; one that follows the data
    /// leaves the scale (the shaders draw a log scale through zero linearly).
    pub fn set_range(&mut self, min: f32, max: f32, locked: bool) {
        (self.range_min, self.range_max, self.lock_bounds) = (min, max, locked);
        if locked && !self.allows_log() && self.scale == ColorScale::Log {
            self.scale = ColorScale::Linear;
        }
    }

    /// Whether the range can draw a log scale: nothing below zero.
    pub fn allows_log(&self) -> bool {
        self.range_min >= -1e-15 && self.range_max > 0.0
    }

    /// Sets `scale`, unless it is a log scale the range cannot draw.
    pub fn set_scale(&mut self, scale: ColorScale) {
        if scale != ColorScale::Log || self.allows_log() {
            self.scale = scale;
        }
    }

    /// Widens the data extent seen so far by another step's `min`..`max`, and
    /// moves the range there unless it is locked. Non-finite ends are skipped.
    pub fn follow_extent(&mut self, min: f32, max: f32) {
        if min.is_finite() {
            self.global_min = self.global_min.min(min);
            if !self.lock_bounds {
                self.range_min = min;
            }
        }
        if max.is_finite() {
            self.global_max = self.global_max.max(max);
            if !self.lock_bounds {
                self.range_max = max;
            }
        }
    }
}
