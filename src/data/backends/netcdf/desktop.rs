//! Desktop NetCDF storage backend: local files through netCDF-C.

use std::sync::Arc;

use super::c_reader::CReader;
use super::pure::PureReader;
use super::reader::NcReader;
use crate::data::blocks::BlockStoreError;

#[derive(Clone)]
pub struct NetCdfBlockStore {
    pub(crate) file_path: String,
    /// The file open in `hdf5-pure` (the parity tests); netCDF files are opened by netCDF-C
    /// per read.
    pure: Option<Arc<PureReader>>,
}

impl std::fmt::Debug for NetCdfBlockStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetCdfBlockStore")
            .field("file_path", &self.file_path)
            .finish()
    }
}

impl NetCdfBlockStore {
    /// Opens a NetCDF or HDF5 dataset from a local filesystem path.
    pub fn open_local(path: &str) -> Result<Self, BlockStoreError> {
        let clean_path = path
            .strip_prefix("file://")
            .or_else(|| path.strip_prefix("netcdf://"))
            .or_else(|| path.strip_prefix("hdf5://"))
            .unwrap_or(path)
            .trim()
            .trim_matches('\'')
            .trim_matches('"');
        let p = crate::utils::expand_tilde(clean_path);
        if !p.exists() {
            return Err(format!("File not found: {}", p.display()).into());
        }
        if p.is_dir() {
            return Err(format!(
                "'{}' is a directory. Please specify a .nc / .h5 / .hdf5 / .cdf file path.",
                p.display()
            )
            .into());
        }
        let file_path = p.to_string_lossy().to_string();
        // Verify that the file can be opened and parsed by netcdf
        CReader::open(&file_path)?;
        Ok(Self {
            file_path,
            pure: None,
        })
    }

    /// The file at `path` read by `hdf5-pure`, whatever it holds, for the parity tests.
    #[cfg(test)]
    pub(super) fn open_pure(path: &str) -> Result<Self, BlockStoreError> {
        let pure = Some(Arc::new(PureReader::open_path(path)?));
        Ok(Self {
            file_path: path.to_owned(),
            pure,
        })
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Runs `f` on the file's reader.
    pub(super) fn with_reader<R>(
        &self,
        f: impl FnOnce(&dyn NcReader) -> Result<R, BlockStoreError>,
    ) -> Result<R, BlockStoreError> {
        match &self.pure {
            Some(reader) => f(reader.as_ref()),
            None => f(&CReader::open(&self.file_path)?),
        }
    }
}
