//! Variable metadata and spatial role resolution.

use std::collections::HashMap;
use std::sync::Arc;

use crate::data::ome::OmeLabelInfo;

#[derive(Debug, Clone, Default)]
pub struct VariableInfo {
    pub name: String,
    pub data_type: String,
    pub shape: Vec<u64>,
    pub dimension_names: Vec<String>,
    /// The read unit: a sharded array's inner chunks, else its chunks.
    pub chunk_shape: Vec<u64>,
    /// The shard (stored object) shape of a sharded array; `None` when
    /// it is not sharded.
    pub shard_shape: Option<Vec<u64>>,
    pub file_size: u64,
    pub units: Option<String>,
    pub long_name: Option<String>,
    pub time_coverage_start: Option<String>,
    pub time_coverage_end: Option<String>,
    pub temporal_resolution: Option<String>,
    pub attributes: HashMap<String, String>,
    /// The OME-Zarr image and level this variable is, for OME-Zarr stores.
    pub ome: Option<crate::data::ome::OmeRef>,
    /// The multiscale stack and level this variable is, when its format
    /// groups it with other resolutions of the same data.
    pub multiscale: Option<crate::data::multiscale::LevelRef>,
}

impl VariableInfo {
    /// Returns the leaf name of the variable (e.g. "u_wind" for "atmosphere/forecast/u_wind").
    pub fn leaf_name(&self) -> &str {
        let clean = self.name.trim_start_matches('/').trim_end_matches('/');
        clean.rsplit('/').next().unwrap_or(clean)
    }

    /// Returns the group prefix path of the variable (e.g. "atmosphere/forecast" or None for root variables).
    pub fn group_path(&self) -> Option<&str> {
        let clean = self.name.trim_start_matches('/').trim_end_matches('/');
        if let Some(pos) = clean.rfind('/') {
            Some(&clean[..pos])
        } else {
            None
        }
    }

    /// The stack this variable is level 0 of.
    pub fn stack_base(&self) -> Option<&crate::data::multiscale::LevelRef> {
        self.multiscale.as_ref().filter(|r| r.is_base())
    }

    /// Whether this is a level other than 0 of a multiscale stack: listed
    /// under its stack's level 0, not on its own.
    pub fn is_coarser_level(&self) -> bool {
        self.multiscale.as_ref().is_some_and(|r| !r.is_base())
    }

    /// The name the variable is listed under: its stack's for level 0 of a
    /// multiscale stack, else its leaf name.
    pub fn display_name(&self) -> &str {
        match self.stack_base() {
            Some(r) if !r.set.name.is_empty() => &r.set.name,
            _ => self.leaf_name(),
        }
    }

    /// The group the variable is listed in: its stack's for level 0 of a
    /// multiscale stack, else its own.
    pub fn display_group(&self) -> &str {
        match self.stack_base() {
            Some(r) => r.set.display_group(),
            None => self.group_path().unwrap_or(""),
        }
    }

    /// Index of dimension `dim`, matched ASCII case-insensitively.
    pub fn dimension_index(&self, dim: &str) -> Option<usize> {
        self.dimension_names
            .iter()
            .position(|d| d.eq_ignore_ascii_case(dim))
    }

    /// The physical unit symbol (`µm`, `s`) of dimension `dim`, for OME-Zarr images.
    pub fn dimension_unit(&self, dim: &str) -> Option<&str> {
        let ome = self.ome.as_ref()?;
        ome.image
            .axes
            .get(self.dimension_index(dim)?)?
            .unit_symbol()
    }

    /// The physical step between indices of dimension `axis` (positive and
    /// finite), for OME-Zarr images.
    pub fn ome_scale(&self, axis: usize) -> Option<f64> {
        let scale = self.ome.as_ref()?.scale(axis)?.abs();
        (scale.is_finite() && scale > 0.0).then_some(scale)
    }

    /// Whether this is an OME-Zarr label image (segmentation ids).
    pub fn is_label_image(&self) -> bool {
        self.ome.as_ref().is_some_and(|o| o.labels)
    }

    /// A label image's colors and properties.
    pub fn label_info(&self) -> Option<&Arc<OmeLabelInfo>> {
        self.ome
            .as_ref()
            .filter(|o| o.labels)?
            .image
            .label_info
            .as_ref()
    }

    /// The dimension a multi-channel overlay runs across: the OME-Zarr
    /// channel axis, else a dimension named like a channel.
    pub fn channel_axis(&self) -> Option<usize> {
        if let Some(ome) = &self.ome {
            return ome.image.axis_of_kind(crate::data::ome::AxisKind::Channel);
        }
        self.dimension_names
            .iter()
            .position(|d| crate::data::coordinates::naming::is_channel_dim_name(d))
    }

    /// Resolves spatial X, Y, and Z dimension indices for this variable using explicit spatial role configs
    /// or fallback dimension name heuristics.
    pub fn resolve_spatial_dim_indices(
        &self,
        dim_configs: &[crate::app::DimConfig],
    ) -> (Option<usize>, Option<usize>, Option<usize>) {
        let explicit_grid = (0..self.dimension_names.len()).find(|&d| {
            dim_configs
                .get(d)
                .is_some_and(|c| c.spatial == crate::app::SpatialRole::Grid)
        });

        if let Some(grid_d) = explicit_grid {
            let explicit_z = (0..self.dimension_names.len()).find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::Z)
            });
            return (Some(grid_d), None, explicit_z);
        }

        let explicit_x = (0..self.dimension_names.len())
            .find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::X)
            })
            .or_else(|| {
                self.dimension_names
                    .iter()
                    .rposition(|d| crate::data::coordinates::naming::is_spatial_x_name(d))
            });

        let explicit_y = (0..self.dimension_names.len())
            .find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::Y)
            })
            .or_else(|| {
                self.dimension_names
                    .iter()
                    .rposition(|d| crate::data::coordinates::naming::is_spatial_y_name(d))
            });

        let explicit_z = (0..self.dimension_names.len())
            .find(|&d| {
                dim_configs
                    .get(d)
                    .is_some_and(|c| c.spatial == crate::app::SpatialRole::Z)
            })
            .or_else(|| {
                self.dimension_names
                    .iter()
                    .rposition(|d| crate::data::coordinates::naming::is_spatial_z_name(d))
            });

        (explicit_x, explicit_y, explicit_z)
    }
}
