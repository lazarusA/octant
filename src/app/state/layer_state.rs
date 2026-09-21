//! Multi-variable plotted layer descriptor.

use super::app_state::OctantApp;
use super::dimension_state::DimConfig;
use super::store_kind::StoreKind;
use crate::data::DatasetMetadata;

/// Represents a single plotted variable layer for current and future multi-variable visual overlays
/// (e.g. vector fields (u, v), multi-channel RGB composite layers, dual-curve plots).
#[derive(Debug, Clone)]
pub struct PlottedVariableState {
    pub store_kind: StoreKind,
    pub store_target_input: String,
    pub dataset_metadata: Option<DatasetMetadata>,
    pub variable_idx: usize,
    pub dim_config: Vec<DimConfig>,
    pub selected_dim_indices: Vec<usize>,
    pub selected_dim_ranges: Vec<(usize, usize)>,
    pub spatial_dims: Vec<usize>,
    pub animated_dim: Option<usize>,
}

impl PlottedVariableState {
    pub fn from_app(app: &OctantApp) -> Self {
        Self {
            store_kind: app.plotted_store_kind,
            store_target_input: app.plotted_store_target_input.clone(),
            dataset_metadata: app.plotted_dataset_metadata.clone(),
            variable_idx: app.plotted_variable_idx,
            dim_config: app.plotted_dim_config.clone(),
            selected_dim_indices: app.plotted_selected_dim_indices.clone(),
            selected_dim_ranges: app.plotted_selected_dim_ranges.clone(),
            spatial_dims: app.plotted_spatial_dims.clone(),
            animated_dim: app.plotted_animated_dim,
        }
    }
}
