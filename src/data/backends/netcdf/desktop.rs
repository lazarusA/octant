//! Desktop NetCDF storage backend: local files through netCDF-C, MATLAB v7.3 `.mat` files
//! through `hdf5-pure`.

use std::sync::Arc;

use super::c_reader::CReader;
use super::pure::PureReader;
use super::reader::NcReader;
use crate::data::blocks::BlockStoreError;

#[derive(Clone)]
pub struct NetCdfBlockStore {
    pub(crate) file_path: String,
    /// The file open in `hdf5-pure` (`.mat` files); netCDF files are opened by netCDF-C
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

/// Whether the file at `path` starts as a MATLAB file does (`MATLAB 7.3 MAT-file`, or an
/// older one, which `PureReader` refuses with its reason).
fn is_matlab(path: &std::path::Path) -> bool {
    use std::io::Read;
    let mut head = [0; 6];
    std::fs::File::open(path).is_ok_and(|mut f| f.read_exact(&mut head).is_ok())
        && &head == b"MATLAB"
}

impl NetCdfBlockStore {
    /// Opens a NetCDF, HDF5 or MATLAB v7.3 dataset from a local filesystem path.
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
                "'{}' is a directory. Please specify a .nc / .h5 / .hdf5 / .cdf / .mat file path.",
                p.display()
            )
            .into());
        }
        let file_path = p.to_string_lossy().to_string();
        let pure = match is_matlab(&p) {
            true => Some(Arc::new(PureReader::open_path(&file_path)?)),
            false => {
                // Verify that the file can be opened and parsed by netcdf
                CReader::open(&file_path)?;
                None
            }
        };
        Ok(Self { file_path, pure })
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
