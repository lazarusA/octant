//! What one layer requests: its store, variable and slice selections, built
//! from the layer's own selection so every layer loads alike.

use crate::app::layers::{LayerId, VariableSelection};
use crate::app::{OctantApp, StoreKind};
use crate::data::{DimensionSelection, SliceRequest, StoreHandle, VariableInfo};

/// A layer's request over its selected ranges, before the animated window.
pub(crate) struct LayerRequest {
    pub source_id: String,
    store_kind: StoreKind,
    store_target: String,
    pub var: VariableInfo,
    pub anim_dim: Option<usize>,
    pub request: SliceRequest,
}

/// The source id of `selection`'s store, or `fallback` (the staged store's)
/// while its target is empty.
pub(crate) fn source_id_or(selection: &VariableSelection, fallback: &str) -> String {
    if selection.store_target.is_empty() {
        fallback.to_string()
    } else {
        StoreKind::make_source_id(selection.store_kind, &selection.store_target)
    }
}

impl OctantApp {
    /// The source id of `selection`'s store; an empty target falls back to
    /// the staged store.
    pub(crate) fn selection_source_id(&self, selection: &VariableSelection) -> String {
        source_id_or(selection, &self.selected_source_id())
    }

    /// The request of layer `id`'s staged selection (the next plot); `None`
    /// while its variable index is invalid.
    pub(crate) fn staged_layer_request(&self, id: LayerId) -> Option<LayerRequest> {
        let (_, staged) = self.layer_selections(id)?;
        self.selection_request(id, staged, staged.variable_info()?)
    }

    /// The request of layer `id`'s shown selection (what it plots now); its
    /// variable falls back to the staged one until the layer's first plot.
    pub(crate) fn shown_layer_request(&self, id: LayerId) -> Option<LayerRequest> {
        let (shown, _) = self.layer_selections(id)?;
        self.selection_request(id, shown, self.layer_variable_info(id)?)
    }

    /// `selection`'s request of `var` for layer `id`, reading the channel
    /// dimension whole while the layer's composite is on.
    fn selection_request(
        &self,
        id: LayerId,
        selection: &VariableSelection,
        var: &VariableInfo,
    ) -> Option<LayerRequest> {
        let var = var.clone();
        let composite = self.layers.get(id)?.composite.enabled;
        let composite_dim =
            crate::app::state::dataset_activation::channel_dim(&var).filter(|_| composite);
        let request = selection.slice_request(&var.name, &var.shape, composite_dim);
        Some(LayerRequest {
            source_id: self.selection_source_id(selection),
            store_kind: selection.store_kind,
            store_target: selection.store_target.clone(),
            anim_dim: selection.animated_dim,
            var,
            request,
        })
    }

    /// The open store `request` reads from (opening it when needed); `None`
    /// while the dataset can't be opened.
    pub(crate) fn request_store(&self, request: &LayerRequest) -> Option<StoreHandle> {
        self.resolve_store_handle(
            &request.source_id,
            &request.store_target,
            request.store_kind,
        )
    }

    /// Whether the block cache holds step `step` of layer `id`'s shown selection.
    pub(crate) fn layer_step_resident(&self, id: LayerId, step: usize) -> bool {
        self.shown_layer_request(id)
            .is_some_and(|r| self.request_resident(&r, step))
    }

    /// Whether the block cache holds step `step` of `request`.
    pub(crate) fn request_resident(&self, request: &LayerRequest, step: usize) -> bool {
        self.block_cache.covers(
            &request.source_id,
            &request.var.name,
            &request.request.selections,
            request.anim_dim,
            step,
        )
    }

    /// The step layer `id` shows: the shared current step, which an overlay
    /// with a shorter animated dimension clamps into its own extent.
    pub(crate) fn layer_step(&self, id: LayerId) -> usize {
        if id == LayerId::BASE {
            return self.current_timestep;
        }
        let extent = self
            .layer_animated_dim(id)
            .and_then(|dim| self.layer_variable_info(id)?.shape.get(dim).copied())
            .unwrap_or(0) as usize;
        match extent {
            0 => self.current_timestep,
            n => self.current_timestep.min(n - 1),
        }
    }

    /// `layer_request` windowed along `anim_dim` around the current step, and
    /// that step clamped into the layer's extent. The base layer moves the
    /// shared step there; every layer records it in its selections.
    pub(crate) fn request_at_current_step(
        &mut self,
        id: LayerId,
        layer_request: &LayerRequest,
        anim_dim: Option<usize>,
    ) -> (SliceRequest, usize) {
        let mut selections = layer_request.request.selections.clone();
        let mut step = self.current_timestep;
        if let Some(anim_dim) = anim_dim {
            let shape = &layer_request.var.shape;
            let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
            if full_extent > 0 {
                step = step.min(full_extent - 1);
            }
            if id == LayerId::BASE {
                self.current_timestep = step;
            }
            self.set_layer_step_index(id, anim_dim, step);
            if anim_dim < selections.len() {
                let (start, end, _) = self.animated_window_bounds(
                    step,
                    full_extent,
                    anim_dim,
                    &layer_request.var.chunk_shape,
                    shape,
                    &selections,
                );
                selections[anim_dim] = DimensionSelection::Range { start, end };
            }
        }
        (SliceRequest::new(&layer_request.var.name, selections), step)
    }

    /// Puts `step` into layer `id`'s staged and shown index along `anim_dim`.
    fn set_layer_step_index(&mut self, id: LayerId, anim_dim: usize, step: usize) {
        if let Some(index) = self
            .staged_selection_mut(id)
            .and_then(|s| s.dim_indices.get_mut(anim_dim))
        {
            *index = step;
        }
        if let Some(index) = self
            .layers
            .get_mut(id)
            .and_then(|l| l.selection_mut().dim_indices.get_mut(anim_dim))
        {
            *index = step;
        }
    }
}
