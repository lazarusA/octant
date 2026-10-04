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

impl OctantApp {
    /// Runs `f` with the plotted selection in place of the staged one: loads in
    /// `f` show (and re-sync) the plotted view, then the staged selection and
    /// its pending request come back untouched.
    pub(crate) fn with_plotted_selection(&mut self, f: impl FnOnce(&mut Self)) {
        let staged = Staged {
            store_kind: replace(&mut self.selected_store_kind, self.plotted_store_kind),
            target: replace(
                &mut self.store_target_input,
                self.plotted_store_target_input.clone(),
            ),
            metadata: replace(
                &mut self.active_dataset_metadata,
                self.plotted_dataset_metadata.clone(),
            ),
            variable: replace(&mut self.selected_variable_idx, self.plotted_variable_idx),
            dim_config: replace(&mut self.dim_config, self.plotted_dim_config.clone()),
            indices: replace(
                &mut self.selected_dim_indices,
                self.plotted_selected_dim_indices.clone(),
            ),
            ranges: replace(
                &mut self.selected_dim_ranges,
                self.plotted_selected_dim_ranges.clone(),
            ),
            spatial_dims: replace(&mut self.spatial_dims, self.plotted_spatial_dims.clone()),
            animated_dim: replace(&mut self.animated_dim, self.plotted_animated_dim),
            plot_type: replace(&mut self.active_plot_type, self.plotted_plot_type),
            block_key: self.active_block_key.clone(),
            slice_request: self.active_slice_request.clone(),
            target_step: self.pending_target_step,
        };
        f(self);
        self.selected_store_kind = staged.store_kind;
        self.store_target_input = staged.target;
        self.active_dataset_metadata = staged.metadata;
        self.selected_variable_idx = staged.variable;
        self.dim_config = staged.dim_config;
        self.selected_dim_indices = staged.indices;
        self.selected_dim_ranges = staged.ranges;
        self.spatial_dims = staged.spatial_dims;
        self.animated_dim = staged.animated_dim;
        self.active_plot_type = staged.plot_type;
        self.active_block_key = staged.block_key;
        self.active_slice_request = staged.slice_request;
        self.pending_target_step = staged.target_step;
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
