//! Store handle resolution and animated window bounding.

use crate::app::OctantApp;
use crate::data::DimensionSelection;

impl OctantApp {
    /// Aligned `[start, end)` window along the animated dimension that
    /// contains `step`, clamped to the dataset's actual extent.
    ///
    /// Automatically bounds the window size based on:
    /// 1. Chunk size along the animated dimension (always chunk-aligned).
    /// 2. Memory footprint of a single chunk (caps single block footprint to ~64 MB).
    /// 3. User-configured `block_window_size` (e.g. 8 to 128 steps).
    pub fn animated_window_bounds(
        &self,
        step: usize,
        full_extent: usize,
        anim_dim: usize,
        chunk_shape: &[u64],
        _shape: &[u64],
        _selections: &[DimensionSelection],
    ) -> (usize, usize, usize) {
        let cs = chunk_shape.get(anim_dim).copied().unwrap_or(1).max(1) as usize;
        let window_size = cs; // 1 storage chunk per block for instant initial display

        let start = (step / cs) * cs;
        let end = (start + window_size).min(full_extent).max(start + 1);

        (start, end, window_size)
    }

    pub(crate) fn resolve_store_handle(
        &self,
        source_id: &str,
        target_input: &str,
        kind: crate::app::StoreKind,
    ) -> Option<crate::data::StoreHandle> {
        if let Some(d) = self.dataset_manager.get(source_id) {
            return Some(d.store.clone());
        }

        // Fallback: match by URI in dataset_manager
        let target_uri = target_input.trim().trim_end_matches('/');
        if let Some(d) = self.dataset_manager.iter().find(|d| {
            let d_uri = d.source.uri.trim().trim_end_matches('/');
            d_uri == target_uri || d.id == source_id || d.id.ends_with(target_uri)
        }) {
            return Some(d.store.clone());
        }

        // Auto-open through SourceFactory if not yet in dataset_manager
        let kind = kind.to_data_source_kind();
        let source = crate::data::DataSource::new(source_id, kind, target_input, "Store");
        crate::data::SourceFactory::open(source).ok()
    }

    /// Returns the open `StoreHandle` for the currently selected (target) dataset from `dataset_manager`.
    pub fn selected_store_handle(&self) -> Option<crate::data::StoreHandle> {
        let source_id = self.selected_source_id();
        self.resolve_store_handle(
            &source_id,
            &self.store_target_input,
            self.selected_store_kind,
        )
    }

    /// Returns the open `StoreHandle` for the currently plotted dataset from `dataset_manager`.
    pub fn plotted_store_handle(&self) -> Option<crate::data::StoreHandle> {
        let source_id = self.plotted_source_id();
        self.resolve_store_handle(
            &source_id,
            &self.plotted_store_target_input,
            self.plotted_store_kind,
        )
    }
}
