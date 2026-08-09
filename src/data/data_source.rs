//! User-facing description of a data source.
//!
//! This is intentionally independent of the backend implementation.
//! The UI chooses a DataSource before choosing variables.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataSourceKind {
    LocalZarr,
    RemoteZarr,

    LocalIcechunk,
    RemoteIcechunk,

    NetCdf,
    GeoTiff,

    /// Reserved for future readers.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
