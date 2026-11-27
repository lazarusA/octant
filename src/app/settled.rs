//! Whether the view shows all it will: no zoom window waiting for its
//! heatmap, no detail tile and no volume brick being read (`--snapshot`,
//! tests).

use super::OctantApp;

impl OctantApp {
    /// Whether the plotted view has finished refining. Reads start only once
    /// the view rests (`detail::SETTLE`), so a view that just stopped may
    /// still count as settled for a frame or two.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(crate) fn view_settled(&self) -> bool {
        self.nav.pending_view.is_none()
            && !self.layers.iter().any(|l| l.data.detail.busy())
            && !self.layers.base.data.volume_lod.busy()
    }
}
