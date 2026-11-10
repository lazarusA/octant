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

    /// What layer `id` shows: its plotted selection once it has one, else
    /// the staged one (the base layer before its first plot). Every `layer_*`
    /// and `effective_*` accessor reads this one view, never a mix of both.
    pub(crate) fn layer_view(&self, id: LayerId) -> Option<&VariableSelection> {
        let (shown, staged) = self.layer_selections(id)?;
        Some(if shown.metadata.is_some() {
            shown
        } else {
            staged
        })
    }

    /// Layer `id`'s dimension configs (`layer_view`).
    pub fn layer_dim_config(&self, id: LayerId) -> &[DimConfig] {
        self.layer_view(id).map_or(&[], |s| &s.dim_config)
    }

    /// Layer `id`'s animated dimension (`layer_view`).
    pub fn layer_animated_dim(&self, id: LayerId) -> Option<usize> {
        self.layer_view(id)?.animated_dim()
    }

    /// Layer `id`'s dataset metadata (`layer_view`).
    pub fn layer_dataset_metadata(&self, id: LayerId) -> Option<&crate::data::DatasetMetadata> {
        self.layer_view(id)?.metadata.as_ref()
    }

    /// Layer `id`'s variable (`layer_view`).
    pub fn layer_variable_info(&self, id: LayerId) -> Option<&crate::data::VariableInfo> {
        self.layer_view(id)?.variable_info()
    }

    /// Returns the active dimension configs: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_dim_config(&self) -> &[DimConfig] {
        self.layer_dim_config(LayerId::BASE)
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

    /// Copies the staged selection into the plotted one, on the level of
    /// its stack whose variable is `shown` (a block that arrived, which may
    /// be another level than the canvas asks for now), else the level it
    /// reads.
    pub(crate) fn copy_selected_to_plotted(&mut self, shown: Option<&str>) {
        let moved = match shown {
            Some(path) => crate::app::multiscale::at_level_named(&self.selected, path),
            None => self.resolve_level(&self.selected),
        };
        match moved {
            Some(level) => *self.layers.base.selection_mut() = level,
            None => self.layers.base.selection_mut().clone_from(&self.selected),
        }
        // A label image draws as labels, whatever layer shows it; data shown
        // the other way is dropped, never drawn in the wrong colors.
        if self.layers.base.source.follow_variable() {
            self.layers.base.clear_2d();
            self.layers.base.data.var_key = None;
        }
    }

    /// Copies the plotted selection back into the staged one, on level 0 of
    /// a multiscale variable.
    pub(crate) fn copy_plotted_to_selected(&mut self) {
        match crate::app::multiscale::at_base(self.layers.base.selection()) {
            Some(base) => self.selected = base,
            None => self.selected.clone_from(self.layers.base.selection()),
        }
    }
}
