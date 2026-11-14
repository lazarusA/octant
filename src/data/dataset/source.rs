//! User-facing description of a data source.
//!
//! This is intentionally independent of the backend implementation.
//! The UI chooses a DataSource before choosing variables.

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataSourceKind {
    LocalZarr,
    RemoteZarr,

    LocalIcechunk,
    RemoteIcechunk,

    Procedural,

    LocalGeoTiff,
    RemoteGeoTiff,

    NetCdf,

    /// GRIB2 files, local or remote, read through the NetCDF backend's reader seam.
    Grib,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataSource {
    pub id: String,
    pub kind: DataSourceKind,
    pub uri: String,
    pub display_name: String,
}

impl DataSource {
    pub fn new(
        id: impl Into<String>,
        kind: DataSourceKind,
        uri: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            uri: uri.into(),
            display_name: display_name.into(),
        }
    }
}
