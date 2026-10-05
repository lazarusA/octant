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

impl OctantApp {
    /// Adds a secondary dimensionally-compatible variable layer for multi-variable plotting.
    pub fn add_plotted_layer(&mut self, layer: PlottedVariableState) -> Result<(), String> {
        if let (Some(existing_meta), Some(new_meta)) =
            (&self.plotted_dataset_metadata, &layer.dataset_metadata)
        {
            let existing_var = existing_meta.variables.get(self.plotted_variable_idx);
            let new_var = new_meta.variables.get(layer.variable_idx);

            if let (Some(v_a), Some(v_b)) = (existing_var, new_var) {
                check_dimensional_compatibility(v_a, v_b)?;
            }
        }
        self.multi_plotted_layers.push(layer);
        Ok(())
    }

    /// Clears secondary multi-variable layers.
    pub fn clear_plotted_layers(&mut self) {
        self.multi_plotted_layers.clear();
    }
}

/// Helper function to verify dimensional compatibility between two variables
/// (matching rank, shapes, or spatial extent) for multi-layer plotting.
pub fn check_dimensional_compatibility(
    var_a: &crate::data::VariableInfo,
    var_b: &crate::data::VariableInfo,
) -> Result<(), String> {
    if var_a.shape.len() != var_b.shape.len() {
        return Err(format!(
            "Rank mismatch: '{}' (rank {}) vs '{}' (rank {})",
            var_a.name,
            var_a.shape.len(),
            var_b.name,
            var_b.shape.len()
        ));
    }
    if var_a.shape != var_b.shape {
        return Err(format!(
            "Shape mismatch: '{}' ({:?}) vs '{}' ({:?})",
            var_a.name, var_a.shape, var_b.name, var_b.shape
        ));
    }
    Ok(())
}
