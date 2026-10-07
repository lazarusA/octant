//! Loads against the plotted selection while the UI stages another one (a
//! different variable or a plot still loading): playback keeps animating what
//! is shown until the staged plot's data arrives.

use crate::app::{DimConfig, OctantApp, StoreKind};
use crate::data::{BlockCacheKey, DatasetMetadata, SliceRequest};
use crate::plots::PlotType;
use std::mem::replace;

/// The staged selection and the bookkeeping of its pending request.
struct Staged {
    store_kind: StoreKind,
    target: String,
    metadata: Option<DatasetMetadata>,
    generation: u64,
    variable: usize,
    dim_config: Vec<DimConfig>,
    indices: Vec<usize>,
    ranges: Vec<(usize, usize)>,
    spatial_dims: Vec<usize>,
    animated_dim: Option<usize>,
    plot_type: PlotType,
    block_key: Option<BlockCacheKey>,
    slice_request: Option<SliceRequest>,
    target_step: Option<usize>,
}

impl Staged {
    fn swap_with_plotted(app: &mut OctantApp) -> Self {
        Self {
            store_kind: replace(&mut app.selected_store_kind, app.plotted_store_kind),
            target: replace(
                &mut app.store_target_input,
                app.plotted_store_target_input.clone(),
            ),
            generation: replace(
                &mut app.metadata_generation,
                app.plotted_metadata_generation,
            ),
            metadata: replace(
                &mut app.active_dataset_metadata,
                app.plotted_dataset_metadata.clone(),
            ),
            variable: replace(&mut app.selected_variable_idx, app.plotted_variable_idx),
            dim_config: replace(&mut app.dim_config, app.plotted_dim_config.clone()),
            indices: replace(
                &mut app.selected_dim_indices,
                app.plotted_selected_dim_indices.clone(),
            ),
            ranges: replace(
                &mut app.selected_dim_ranges,
                app.plotted_selected_dim_ranges.clone(),
            ),
            spatial_dims: replace(&mut app.spatial_dims, app.plotted_spatial_dims.clone()),
            animated_dim: replace(&mut app.animated_dim, app.plotted_animated_dim),
            plot_type: replace(&mut app.active_plot_type, app.plotted_plot_type),
            block_key: app.active_block_key.clone(),
            slice_request: app.active_slice_request.clone(),
            target_step: app.pending_target_step,
        }
    }

    fn restore(self, app: &mut OctantApp) {
        app.selected_store_kind = self.store_kind;
        app.store_target_input = self.target;
        app.active_dataset_metadata = self.metadata;
        app.metadata_generation = self.generation;
        app.selected_variable_idx = self.variable;
        app.dim_config = self.dim_config;
        app.selected_dim_indices = self.indices;
        app.selected_dim_ranges = self.ranges;
        app.spatial_dims = self.spatial_dims;
        app.animated_dim = self.animated_dim;
        app.active_plot_type = self.plot_type;
        app.active_block_key = self.block_key;
        app.active_slice_request = self.slice_request;
        app.pending_target_step = self.target_step;
    }
}

impl OctantApp {
    /// Runs `f` with the plotted selection in place of the staged one: loads in
    /// `f` show (and re-sync) the plotted view, then the staged selection and
    /// its pending request come back untouched.
    pub(crate) fn with_plotted_selection(&mut self, f: impl FnOnce(&mut Self)) {
        let staged = Staged::swap_with_plotted(self);
        f(self);
        staged.restore(self);
    }

    /// Whether the UI stages something other than the plotted view: another
    /// variable, or a plot whose requested block is still loading.
    pub(crate) fn staging_differs(&self) -> bool {
        let request_pending = self
            .active_block_key
            .as_ref()
            .is_some_and(|key| self.block_prefetcher.is_pending(key));
        request_pending || self.is_exploring_unplotted_variable()
    }

    /// Whether a block with cache key `key` belongs to the latest requested view
    /// (any step): blocks of an older selection, variable or plot layout must
    /// not be projected into it.
    pub(crate) fn key_matches_view(&self, key: &BlockCacheKey, anim_dim: Option<usize>) -> bool {
        self.active_slice_request.as_ref().is_some_and(|req| {
            key.variable_name == req.variable
                && crate::data::blocks::key::selections_match_except_anim(
                    &key.selections,
                    &req.selections,
                    anim_dim,
                )
        })
    }
}
