//! Dimension configuration and role metadata.

use crate::app::VariableSelection;
use crate::app::layers::{Layer, LayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialRole {
    None,
    X,
    Y,
    Z,
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationRole {
    None,
    Animated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimConfig {
    pub spatial: SpatialRole,
    pub animation: AnimationRole,
    pub active: bool, // true = range (expanded), false = single index (collapsed)
    pub index: usize, // selected single step / index
    pub range: (usize, usize), // selected (start, end) range
}

impl Default for DimConfig {
    fn default() -> Self {
        Self {
            spatial: SpatialRole::None,
            animation: AnimationRole::None,
            active: false,
            index: 0,
            range: (0, 0),
        }
    }
}

impl DimConfig {
    pub fn new(
        spatial: SpatialRole,
        animation: AnimationRole,
        active: bool,
        index: usize,
        range: (usize, usize),
    ) -> Self {
        Self {
            spatial,
            animation,
            active,
            index,
            range,
        }
    }

    pub fn grid_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::Grid)
    }

    pub fn x_dim(configs: &[DimConfig]) -> Option<usize> {
        configs
            .iter()
            .position(|c| c.spatial == SpatialRole::X || c.spatial == SpatialRole::Grid)
    }

    pub fn y_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::Y)
    }

    pub fn z_dim(configs: &[DimConfig]) -> Option<usize> {
        configs.iter().position(|c| c.spatial == SpatialRole::Z)
    }

    pub fn animated_dim(configs: &[DimConfig]) -> Option<usize> {
        configs
            .iter()
            .position(|c| c.animation == AnimationRole::Animated)
    }

    pub fn spatial_dims(configs: &[DimConfig]) -> Vec<usize> {
        let mut list = Vec::new();
        if let Some(i) = Self::x_dim(configs) {
            list.push(i);
        }
        if let Some(i) = Self::y_dim(configs) {
            list.push(i);
        }
        if let Some(i) = Self::z_dim(configs) {
            list.push(i);
        }
        list
    }
}

impl crate::app::OctantApp {
    /// Layer `id`'s selection and the staged one it falls back to while it is
    /// empty: the base layer's is `selected` (the UI stages the next plot
    /// there), any other layer stages its own. `None` once the layer is gone.
    pub(crate) fn layer_selections(
        &self,
        id: LayerId,
    ) -> Option<(&VariableSelection, &VariableSelection)> {
        let layer = self.layers.get(id)?;
        let staged = if id == LayerId::BASE {
            &self.selected
        } else {
            layer.selection()
        };
        Some((layer.selection(), staged))
    }

    /// The selection the UI stages for layer `id`, mutably.
    pub(crate) fn staged_selection_mut(&mut self, id: LayerId) -> Option<&mut VariableSelection> {
        if id == LayerId::BASE {
            Some(&mut self.selected)
        } else {
            self.layers.get_mut(id).map(Layer::selection_mut)
        }
    }

    /// Layer `id`'s `field`, or its staged one while the shown one is empty;
    /// empty once the layer is gone.
    fn shown_or_staged<T>(&self, id: LayerId, field: fn(&VariableSelection) -> &[T]) -> &[T] {
        let Some((shown, staged)) = self.layer_selections(id) else {
            return &[];
        };
        let values = field(shown);
        if values.is_empty() {
            field(staged)
        } else {
            values
        }
    }

    /// Layer `id`'s dimension configs, falling back to its staged ones.
    pub fn layer_dim_config(&self, id: LayerId) -> &[DimConfig] {
        self.shown_or_staged(id, |s| &s.dim_config)
    }

    /// Layer `id`'s dimension ranges, falling back to its staged ones.
    pub fn layer_dim_ranges(&self, id: LayerId) -> &[(usize, usize)] {
        self.shown_or_staged(id, |s| &s.dim_ranges)
    }

    /// Layer `id`'s dimension indices, falling back to its staged ones.
    pub fn layer_dim_indices(&self, id: LayerId) -> &[usize] {
        self.shown_or_staged(id, |s| &s.dim_indices)
    }

    /// Layer `id`'s animated dimension, falling back to its staged one.
    pub fn layer_animated_dim(&self, id: LayerId) -> Option<usize> {
        let (shown, staged) = self.layer_selections(id)?;
        shown.animated_dim.or(staged.animated_dim)
    }

    /// Layer `id`'s dataset metadata, falling back to its staged one.
    pub fn layer_dataset_metadata(&self, id: LayerId) -> Option<&crate::data::DatasetMetadata> {
        let (shown, staged) = self.layer_selections(id)?;
        shown.metadata.as_ref().or(staged.metadata.as_ref())
    }

    /// Layer `id`'s variable, falling back to its staged one.
    pub fn layer_variable_info(&self, id: LayerId) -> Option<&crate::data::VariableInfo> {
        let (shown, staged) = self.layer_selections(id)?;
        shown.variable_info().or_else(|| staged.variable_info())
    }

    /// Returns the active dimension configs: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_dim_config(&self) -> &[DimConfig] {
        self.layer_dim_config(LayerId::BASE)
    }

    /// Returns the active dimension ranges: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_selected_dim_ranges(&self) -> &[(usize, usize)] {
        self.layer_dim_ranges(LayerId::BASE)
    }

    /// Returns the active animated dimension: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_animated_dim(&self) -> Option<usize> {
        self.layer_animated_dim(LayerId::BASE)
    }

    /// Returns the active dataset metadata: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_dataset_metadata(&self) -> Option<&crate::data::DatasetMetadata> {
        self.layer_dataset_metadata(LayerId::BASE)
    }

    /// The selection shown on the canvas (the base layer's).
    #[inline]
    pub fn plotted(&self) -> &crate::app::VariableSelection {
        self.layers.base.selection()
    }

    /// Copies the staged selection into the plotted one.
    pub(crate) fn copy_selected_to_plotted(&mut self) {
        self.layers.base.selection_mut().clone_from(&self.selected);
    }

    /// Copies the plotted selection back into the staged one.
    pub(crate) fn copy_plotted_to_selected(&mut self) {
        self.selected.clone_from(self.layers.base.selection());
    }
}
