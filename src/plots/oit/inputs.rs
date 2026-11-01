//! What a mesh or point cloud's OIT frame depends on, so it is rendered
//! again only when one of them changed (`OitState::render`).

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

/// A renderer's last written uniforms (hashed) and a data version bumped by
/// every upload into its buffers, including channel bakes.
#[derive(Default)]
pub struct FrameInputs {
    uniforms: AtomicU64,
    data: AtomicU64,
}

impl FrameInputs {
    /// Records the uniform block just written.
    pub fn uniforms_written(&self, bytes: &[u8]) {
        let mut h = DefaultHasher::new();
        bytes.hash(&mut h);
        self.uniforms.store(h.finish(), Ordering::Relaxed);
    }

    /// Records that the renderer's data or coordinates changed.
    pub fn data_changed(&self) {
        self.data.fetch_add(1, Ordering::Relaxed);
    }

    /// The key of a frame drawn now with `pipeline` (what else picks its
    /// pixels, such as cube mode), the colormaps included.
    pub fn key(&self, pipeline: impl Hash) -> u64 {
        let mut h = DefaultHasher::new();
        (
            self.uniforms.load(Ordering::Relaxed),
            self.data.load(Ordering::Relaxed),
            crate::utils::colormap::registry::generation(),
            pipeline,
        )
            .hash(&mut h);
        h.finish()
    }
}
