//! Opening a file through the readers that read any byte source: classic NetCDF
//! (`classic/`) by its `CDF` signature, GRIB2 (`backends/grib/`) by its `GRIB` one, else
//! HDF5 (`pure/`: netCDF-4, HDF5, MATLAB v7.3).

use std::sync::Arc;

use super::classic::{ClassicReader, is_classic};
use super::pure::PureReader;
use super::pure::read_ahead::RangeSource;
use super::reader::NcReader;
use crate::data::backends::grib::GribReader;
use crate::data::blocks::BlockStoreError;

/// An open file, shared by the store's reads.
pub type Shared = Arc<dyn NcReader + Send + Sync>;

/// Opens the file `source` reads, `name` naming it in errors.
pub fn open_source<S: RangeSource + Send + Sync + 'static>(
    source: S,
    name: &str,
) -> Result<Shared, BlockStoreError> {
    let len = source
        .len()
        .min(crate::data::backends::grib::SIGNATURE_BYTES as u64);
    let mut head = vec![0; len as usize];
    (source.read_at(0, &mut head)).map_err(|e| format!("Failed to read '{name}': {e}"))?;
    if crate::data::backends::grib::is_grib(&head) {
        return Ok(Arc::new(GribReader::open(Box::new(source), name)?));
    }
    Ok(match is_classic(&head) {
        true => Arc::new(ClassicReader::open(Box::new(source), name)?),
        false => Arc::new(PureReader::open(source, name)?),
    })
}

/// Opens the file at `path` on disk.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_path(path: &str) -> Result<Shared, BlockStoreError> {
    let handle = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
    let source = hdf5_pure::ReadSeekSource::new(handle).map_err(|e| format!("{path}: {e}"))?;
    open_source(source, path)
}
