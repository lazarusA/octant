//! High-content screening plates and wells, and what a whole OME-Zarr store
//! holds beyond its images.

use std::sync::Arc;

use ome_zarr_metadata::v0_5::{Plate, Well};

/// One well of a plate: its group path and grid position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlateWell {
    pub path: String,
    pub row: usize,
    pub column: usize,
}

/// One acquisition (a run over the plate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlateAcquisition {
    pub id: u64,
    pub name: Option<String>,
}

/// A plate's layout.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OmePlate {
    pub name: Option<String>,
    pub rows: Vec<String>,
    pub columns: Vec<String>,
    pub wells: Vec<PlateWell>,
    pub acquisitions: Vec<PlateAcquisition>,
}

/// One field of view of a well.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WellField {
    pub path: String,
    pub acquisition: Option<u64>,
}

/// A well's fields of view.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OmeWell {
    pub fields: Vec<WellField>,
}

impl OmeWell {
    /// Fields of `acquisition` (all fields when `None` or none are tagged),
    /// with their index in `fields`.
    pub fn fields_of(&self, acquisition: Option<u64>) -> impl Iterator<Item = (usize, &WellField)> {
        self.fields.iter().enumerate().filter(move |(_, f)| {
            acquisition.is_none() || f.acquisition.is_none() || f.acquisition == acquisition
        })
    }
}

/// What an OME-Zarr store holds beyond its image variables.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OmeDataset {
    /// The version of the store's root metadata.
    pub version: String,
    pub plate: Option<Arc<OmePlate>>,
    pub well: Option<Arc<OmeWell>>,
    /// `(group path, name)`: names of `bioformats2raw` series groups, sorted
    /// by path.
    pub group_names: Vec<(String, String)>,
}

impl OmeDataset {
    /// The name of the group at `path`.
    pub fn group_name(&self, path: &str) -> Option<&str> {
        let path = path.trim_matches('/');
        let i = self
            .group_names
            .binary_search_by(|(p, _)| p.as_str().cmp(path))
            .ok()?;
        Some(self.group_names[i].1.as_str())
    }
}

pub(super) fn plate_info(plate: Plate) -> OmePlate {
    OmePlate {
        name: plate.name,
        rows: plate.rows.into_iter().map(|r| r.name).collect(),
        columns: plate.columns.into_iter().map(|c| c.name).collect(),
        wells: plate
            .wells
            .iter()
            .map(|w| PlateWell {
                path: w.path.to_string_lossy().trim_matches('/').to_string(),
                row: w.row_index,
                column: w.column_index,
            })
            .collect(),
        acquisitions: plate
            .acquisitions
            .into_iter()
            .flatten()
            .map(|a| PlateAcquisition {
                id: a.id,
                name: a.name,
            })
            .collect(),
    }
}

pub fn well_info(well: Well) -> OmeWell {
    OmeWell {
        fields: well
            .images
            .iter()
            .map(|i| WellField {
                path: i.path.to_string_lossy().trim_matches('/').to_string(),
                acquisition: i.acquisition,
            })
            .collect(),
    }
}
