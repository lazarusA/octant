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
    pub field_count: Option<u64>,
}

impl OmePlate {
    /// The well at `(row, column)`.
    pub fn well_at(&self, row: usize, column: usize) -> Option<&PlateWell> {
        self.wells
            .iter()
            .find(|w| w.row == row && w.column == column)
    }
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
    /// Fields of `acquisition` (all fields when `None` or none are tagged).
    pub fn fields_of(&self, acquisition: Option<u64>) -> impl Iterator<Item = &WellField> {
        self.fields.iter().filter(move |f| {
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
    /// `(group path, name)`: names of `bioformats2raw` series groups.
    pub group_names: Vec<(String, String)>,
}

impl OmeDataset {
    /// The name of the group at `path`.
    pub fn group_name(&self, path: &str) -> Option<&str> {
        let path = path.trim_matches('/');
        self.group_names
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, n)| n.as_str())
    }
}

pub fn plate_info(plate: &Plate) -> OmePlate {
    OmePlate {
        name: plate.name.clone(),
        rows: plate.rows.iter().map(|r| r.name.clone()).collect(),
        columns: plate.columns.iter().map(|c| c.name.clone()).collect(),
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
            .iter()
            .flatten()
            .map(|a| PlateAcquisition {
                id: a.id,
                name: a.name.clone(),
            })
            .collect(),
        field_count: plate.field_count.map(|n| n.get()),
    }
}

pub fn well_info(well: &Well) -> OmeWell {
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
