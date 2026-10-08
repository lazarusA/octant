//! The variable and dimension selection one plot is drawn from: the UI's
//! staged selection (`OctantApp::selected`) and each plotted layer's.

use crate::app::{DimConfig, StoreKind};
use crate::data::DatasetMetadata;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::plots::PlotType;

/// A dataset variable with its dimension roles, indices and ranges.
#[derive(Debug, Clone)]
pub struct VariableSelection {
    pub store_kind: StoreKind,
    pub store_target: String,
    pub metadata: Option<DatasetMetadata>,
    /// Counts changes of the staged `metadata` (copied along with it); keys
    /// caches and focus that belong to one loaded dataset.
    pub metadata_generation: u64,
    pub variable_idx: usize,
    /// One per dimension.
    pub dim_config: Vec<DimConfig>,
    /// Collapsed index per dimension.
    pub dim_indices: Vec<usize>,
    /// Range per dimension.
    pub dim_ranges: Vec<(usize, usize)>,
    /// Dimensions assigned X, Y, Z.
    pub spatial_dims: Vec<usize>,
    /// Dimension assigned Animated.
    pub animated_dim: Option<usize>,
    pub plot_type: PlotType,
}

/// The store opened on first launch.
const DEFAULT_STORE: &str =
    "https://s3.bgc-jena.mpg.de:9000/esdl-esdc-v3.0.2/esdc-16d-2.5deg-46x72x1440-3.0.2.zarr";

impl Default for VariableSelection {
    fn default() -> Self {
        Self {
            store_kind: StoreKind::RemoteZarr,
            store_target: DEFAULT_STORE.to_string(),
            metadata: None,
            metadata_generation: 0,
            variable_idx: 0,
            dim_config: Vec::new(),
            dim_indices: Vec::new(),
            dim_ranges: Vec::new(),
            spatial_dims: Vec::new(),
            animated_dim: None,
            plot_type: PlotType::Heatmap,
        }
    }
}

impl VariableSelection {
    /// The selected variable's metadata.
    pub fn variable_info(&self) -> Option<&crate::data::VariableInfo> {
        self.metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.variable_idx))
    }

    /// The request reading `var_name` (of `shape`) over the selected ranges:
    /// collapsed dimensions as one index, `composite_dim` (the channels of a
    /// composite) whole.
    pub fn slice_request(
        &self,
        var_name: &str,
        shape: &[u64],
        composite_dim: Option<usize>,
    ) -> SliceRequest {
        let selections = shape
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let dim_size = s as usize;
                if composite_dim == Some(i) {
                    return DimensionSelection::Range {
                        start: 0,
                        end: dim_size,
                    };
                }
                let (mut start, mut end) = self
                    .dim_ranges
                    .get(i)
                    .copied()
                    .unwrap_or((0, dim_size.saturating_sub(1)));
                if start > end {
                    std::mem::swap(&mut start, &mut end);
                }
                if start == end {
                    DimensionSelection::Index(start)
                } else {
                    DimensionSelection::Range {
                        start,
                        end: (end + 1).min(dim_size),
                    }
                }
            })
            .collect();

        SliceRequest {
            variable: var_name.to_string(),
            selections,
        }
    }
}
