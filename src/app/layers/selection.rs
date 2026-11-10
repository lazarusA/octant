//! The variable and dimension selection one plot is drawn from: the UI's
//! staged selection (`OctantApp::selected`) and each plotted layer's.

use crate::app::{DimConfig, StoreKind};
use crate::data::DatasetMetadata;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::plots::PlotType;

/// Which level of a multiscale variable a plot reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelChoice {
    /// The level the plot type asks for (`PlotController::resolution_demand`).
    #[default]
    Auto,
    /// This level, or a coarser one when it exceeds the memory budget.
    Pinned(usize),
}

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
    /// One per dimension: its roles, index and range.
    pub dim_config: Vec<DimConfig>,
    pub plot_type: PlotType,
    /// The level read when the variable is a multiscale stack's level 0;
    /// dimension indices and ranges stay on level 0 (the plotted copy holds
    /// the level's own, `app::multiscale`).
    pub level: LevelChoice,
    /// On a copy moved onto a coarser level, the level-0 indices it came from
    /// (`app::multiscale_pins`); `None` on level 0.
    pub level0: Option<std::sync::Arc<crate::app::multiscale_pins::Level0Indices>>,
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
            plot_type: PlotType::Heatmap,
            level: LevelChoice::Auto,
            level0: None,
        }
    }
}

impl VariableSelection {
    /// The dimension assigned Animated.
    pub fn animated_dim(&self) -> Option<usize> {
        DimConfig::animated_dim(&self.dim_config)
    }

    /// Dimension `i` reads one `index`: what its collapsed slider sets.
    pub fn set_index(&mut self, i: usize, index: usize) {
        self.set_range(i, (index, index), index);
    }

    /// Dimension `i` reads `range` at `index` within it: what its range
    /// slider sets.
    pub fn set_range(&mut self, i: usize, range: (usize, usize), index: usize) {
        if let Some(c) = self.dim_config.get_mut(i) {
            c.range = range;
            c.index = index;
        }
    }

    /// Dimension `i` (animated) shows step `step`: an expanded one keeps its
    /// range, a collapsed one reads only the step.
    pub fn set_step(&mut self, i: usize, step: usize) {
        if let Some(c) = self.dim_config.get_mut(i) {
            c.index = step;
            if !c.active {
                c.range = (step, step);
            }
        }
    }

    /// Dimension `i` (of last index `last`) shows `step`; a range that
    /// misses it moves to hold it, keeping its width.
    pub fn hold_step(&mut self, i: usize, step: usize, last: usize) {
        let Some(&DimConfig {
            range: (start, end),
            ..
        }) = self.dim_config.get(i)
        else {
            return;
        };
        let step = step.min(last);
        let width = end.saturating_sub(start).min(last);
        let range = if (start..=end).contains(&step) {
            (start, end)
        } else {
            let start = step.min(last - width);
            (start, start + width)
        };
        self.set_range(i, range, step);
    }

    /// The selected variable's metadata.
    pub fn variable_info(&self) -> Option<&crate::data::VariableInfo> {
        self.metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.variable_idx))
    }

    /// The request reading `var_name` (of `shape`) over the selected ranges:
    /// collapsed dimensions as one index; a composite's channels read their
    /// range like any dimension. A dimension without a range (ranges of another
    /// variable) reads index 0, never the whole dimension, and is logged.
    pub fn slice_request(&self, var_name: &str, shape: &[u64]) -> SliceRequest {
        if self.dim_config.len() < shape.len() {
            log::warn!(
                "Request for '{var_name}' has {} ranges for {} dimensions; \
                 the missing ones read index 0",
                self.dim_config.len(),
                shape.len()
            );
        }
        let selections = shape
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let dim_size = s as usize;
                let (mut start, mut end) =
                    self.dim_config.get(i).map(|c| c.range).unwrap_or((0, 0));
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
