use std::fmt;

#[derive(Debug)]
pub enum OctantError {
    StoreIo(String),
    ZarrDecode(String),
    MetadataError(String),
    SystemIo(std::io::Error),
    General(String),
}

impl fmt::Display for OctantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OctantError::StoreIo(msg) => write!(f, "DataStore I/O Error: {}", msg),
            OctantError::ZarrDecode(msg) => write!(f, "Zarr Decode Error: {}", msg),
            OctantError::MetadataError(msg) => write!(f, "Dataset Metadata Error: {}", msg),
            OctantError::SystemIo(err) => write!(f, "System I/O Error: {}", err),
            OctantError::General(msg) => write!(f, "Octant Error: {}", msg),
        }
    }
}

impl std::error::Error for OctantError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OctantError::SystemIo(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for OctantError {
    fn from(err: std::io::Error) -> Self {
        OctantError::SystemIo(err)
    }
}

impl From<String> for OctantError {
    fn from(msg: String) -> Self {
        OctantError::General(msg)
    }
}

impl From<&str> for OctantError {
    fn from(msg: &str) -> Self {
        OctantError::General(msg.to_string())
    }
}
