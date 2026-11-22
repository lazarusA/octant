//! What the frame cache shows (`render.rs`): whether it is the frame of
//! given parameters, dropping it for another plot, and test probes.

use super::super::{VolumeRenderer, VolumeUniformParams};

impl VolumeRenderer {
    /// Whether the frame shown is the one `params` draws in `bands` bands
    /// (at any size): else a banded frame is still being built.
    pub fn shows(&self, params: &VolumeUniformParams, bands: u32) -> bool {
        let cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        let Some(key) = cache.key else {
            return false;
        };
        // As `render_frame_banded` clamps them: one band per row at most.
        let banded = bands.clamp(1, key.size[1]) > 1;
        key == self.frame_key(params, key.size, banded)
    }

    /// Drops the shown frame and any band build: the renderer now draws
    /// another plot (`app::volume_lod` reuses it for the next step).
    pub fn forget_frames(&self) {
        let mut cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        (cache.frame, cache.key, cache.next, cache.next_key) = (None, None, None, None);
        cache.band = 0;
        // Nor may the last plot's ray feedback reach the next one's selection.
        if let Some(bricks) = &self.bricks {
            bricks.feedback.forget();
        }
    }

    /// Whether a band build is under way (tests).
    #[cfg(test)]
    pub(crate) fn building(&self) -> bool {
        let cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        cache.next_key.is_some()
    }

    /// The cached frame's texture (tests read it back).
    #[cfg(test)]
    pub(crate) fn frame_texture(&self) -> Option<wgpu::Texture> {
        let cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        cache.frame.as_ref().map(|f| f.texture.clone())
    }
}
