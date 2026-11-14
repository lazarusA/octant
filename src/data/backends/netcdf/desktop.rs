//! Desktop NetCDF storage backend: local files through netCDF-C; MATLAB v7.3 `.mat` files,
//! GRIB2 files (`backends/grib/`) and remote (`http(s)://`, `s3://`) files through the
//! readers of any byte source, the remote ones read by ranged requests.

use super::c_reader::CReader;
use super::open::{Shared, open_path, open_source};
use super::pure::read_ahead::{RangeSource, ReadAhead};
use super::reader::NcReader;
use crate::data::blocks::BlockStoreError;

#[derive(Clone)]
pub struct NetCdfBlockStore {
    pub(crate) file_path: String,
    /// The file open in `hdf5-pure` (`.mat` files); netCDF files are opened by netCDF-C
    /// per read.
    pure: Option<Shared>,
}

impl std::fmt::Debug for NetCdfBlockStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetCdfBlockStore")
            .field("file_path", &self.file_path)
            .finish()
    }
}

/// Whether the file at `path` is one netCDF-C does not read: a MATLAB file (`MATLAB 7.3
/// MAT-file`, or an older one, which `hdf5-pure`'s reader refuses with its reason) or a
/// GRIB file.
fn is_pure_only(path: &std::path::Path) -> bool {
    use std::io::Read;
    let bytes = crate::data::backends::grib::SIGNATURE_BYTES;
    let mut head = Vec::with_capacity(bytes);
    let read = std::fs::File::open(path).and_then(|f| f.take(bytes as u64).read_to_end(&mut head));
    read.is_ok() && (head.starts_with(b"MATLAB") || crate::data::backends::grib::is_grib(&head))
}

impl NetCdfBlockStore {
    /// Opens a NetCDF, HDF5 or MATLAB v7.3 dataset from a local filesystem path, or a
    /// NetCDF-4/HDF5 one from an `http(s)` URL.
    pub fn open_local(path: &str) -> Result<Self, BlockStoreError> {
        let remote = ["s3://", "https://", "http://"];
        if remote.iter().any(|scheme| path.starts_with(scheme)) {
            let reader = open_remote(&crate::utils::remote::s3_as_https(path), path)?;
            return Ok(Self {
                file_path: path.to_owned(),
                pure: Some(reader),
            });
        }
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
                "'{}' is a directory. Please specify a .nc / .h5 / .hdf5 / .cdf / .mat / .grib2 file path.",
                p.display()
            )
            .into());
        }
        let file_path = p.to_string_lossy().to_string();
        let pure = match is_pure_only(&p) {
            true => Some(open_path(&file_path)?),
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
        let pure = Some(open_path(path)?);
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

/// The remote file at `url`, `name` naming it in errors; a GRIB file through its sidecar
/// index when it has one (`grib::sidecar_offsets`), its messages' heads read in parallel.
fn open_remote(url: &str, name: &str) -> Result<Shared, BlockStoreError> {
    let source = http_source(url)?;
    if crate::utils::path::is_grib_name(&url.to_ascii_lowercase())
        && let Some(offsets) =
            crate::data::backends::grib::sidecar_offsets(url, hdf5_pure::Source::len(&source))
    {
        use crate::data::backends::grib::GribReader;
        // An index not matching its file (rewritten since) is passed over for a scan.
        match GribReader::open_at(Box::new(source), name, &offsets) {
            Ok(reader) => return Ok(std::sync::Arc::new(reader)),
            Err(e) => log::warn!("'{name}': its sidecar index does not fit, scanning: {e}"),
        }
        return open_source(http_source(url)?, name);
    }
    open_source(source, name)
}

/// The remote file at `url`, read by ranged requests on the shared runtime through a
/// [`ReadAhead`]. Reads come from the background pool's threads, never from the runtime's
/// own, which `block_on` forbids.
fn http_source(url: &str) -> Result<impl RangeSource + Send + Sync + 'static, BlockStoreError> {
    let runtime = crate::utils::executor::get_shared_tokio_rt();
    let len = runtime.block_on(crate::data::backends::http::fetch_url_size(url))?;
    let url = url.to_owned();
    Ok(ReadAhead::new(len, move |offset, length| {
        let read = crate::data::backends::http::fetch_url_byte_range(&url, offset, length);
        (runtime.block_on(read)).map_err(hdf5_pure::FormatError::Source)
    }))
}
