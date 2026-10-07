//! Dimension configuration and role metadata.

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
    /// Returns the active dimension configs: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_dim_config(&self) -> &[DimConfig] {
        if !self.plotted_dim_config.is_empty() {
            &self.plotted_dim_config
        } else {
            &self.dim_config
        }
    }

    /// Returns the active dimension ranges: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_selected_dim_ranges(&self) -> &[(usize, usize)] {
        if !self.plotted_selected_dim_ranges.is_empty() {
            &self.plotted_selected_dim_ranges
        } else {
            &self.selected_dim_ranges
        }
    }

    /// Returns the active dimension indices: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_selected_dim_indices(&self) -> &[usize] {
        if !self.plotted_selected_dim_indices.is_empty() {
            &self.plotted_selected_dim_indices
        } else {
            &self.selected_dim_indices
        }
    }

    /// Returns the active animated dimension: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_animated_dim(&self) -> Option<usize> {
        self.plotted_animated_dim.or(self.animated_dim)
    }

    /// Returns the active dataset metadata: plotted if present, falling back to selected.
    #[inline]
    pub fn effective_dataset_metadata(&self) -> Option<&crate::data::DatasetMetadata> {
        self.plotted_dataset_metadata
            .as_ref()
            .or(self.active_dataset_metadata.as_ref())
    }

    /// Copies all active staged selection fields into the plotted fields.
    pub(crate) fn copy_selected_to_plotted(&mut self) {
        self.plotted_store_kind = self.selected_store_kind;
        self.plotted_store_target_input = self.store_target_input.clone();
        self.plotted_dataset_metadata = self.active_dataset_metadata.clone();
        self.plotted_metadata_generation = self.metadata_generation;
        self.plotted_variable_idx = self.selected_variable_idx;
        self.plotted_dim_config = self.dim_config.clone();
        self.plotted_selected_dim_indices = self.selected_dim_indices.clone();
        self.plotted_selected_dim_ranges = self.selected_dim_ranges.clone();
        self.plotted_spatial_dims = self.spatial_dims.clone();
        self.plotted_animated_dim = self.animated_dim;
        self.plotted_plot_type = self.active_plot_type;
    }

    /// Copies plotted fields back into the staged selection fields.
    pub(crate) fn copy_plotted_to_selected(&mut self) {
        self.selected_store_kind = self.plotted_store_kind;
        self.store_target_input = self.plotted_store_target_input.clone();
        self.active_dataset_metadata = self.plotted_dataset_metadata.clone();
        self.metadata_generation = self.plotted_metadata_generation;
        self.selected_variable_idx = self.plotted_variable_idx;
        self.dim_config = self.plotted_dim_config.clone();
        self.selected_dim_indices = self.plotted_selected_dim_indices.clone();
        self.selected_dim_ranges = self.plotted_selected_dim_ranges.clone();
        self.spatial_dims = self.plotted_spatial_dims.clone();
        self.animated_dim = self.plotted_animated_dim;
        self.active_plot_type = self.plotted_plot_type;
    }
}
