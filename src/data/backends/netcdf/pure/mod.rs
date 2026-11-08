//! The reader over `hdf5-pure`: netCDF-4/HDF5 files (`netcdf4.rs`) in the browser's data
//! workers, and MATLAB v7.3 `.mat` files (`mat.rs`) on both targets. The file is read
//! lazily through a [`Source`]; its variables are described once, when it opens.

mod attrs;
mod mat;
mod netcdf4;
pub mod read_ahead;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod mat_tests;

use std::collections::HashMap;

use hdf5_pure::{ChunkCacheConfig, File, FileAccessProperties, MetadataCacheConfig, Source};

use super::coord_read::{c_text, clean_label};
use super::reader::{NcReader, NcVar, Raw};
use crate::data::blocks::BlockStoreError;

/// Parsed metadata kept per open file: a remote one would otherwise pay a round trip per read.
const METADATA_CACHE: usize = 16 << 20;
/// Decoded chunks kept per open file.
const CHUNK_CACHE: usize = 64 << 20;

/// A variable and the HDF5 dataset holding it.
pub(super) struct Entry {
    var: NcVar,
    dataset: String,
}

pub struct PureReader {
    file: File,
    globals: HashMap<String, String>,
    entries: Vec<Entry>,
    by_path: HashMap<String, usize>,
}

/// Why a file is refused, from its first bytes: classic NetCDF and MATLAB files before
/// v7.3 are not HDF5.
fn refusal(head: &[u8]) -> Option<&'static str> {
    if let [b'C', b'D', b'F', 1 | 2 | 5, ..] = head {
        // The desktop reads them through netCDF-C, from disk only.
        return Some(if cfg!(target_arch = "wasm32") {
            "Classic NetCDF files (CDF-1, CDF-2, CDF-5) open only in the Octant desktop app"
        } else {
            "Classic NetCDF files (CDF-1, CDF-2, CDF-5) open from disk only: download the file first"
        });
    }
    let old_mat = head.starts_with(b"MATLAB 5.0 MAT-file") && !hdf5_pure::is_hdf5_bytes(head);
    old_mat.then_some(
        "MATLAB files before v7.3 are not supported: save it again with save(..., '-v7.3')",
    )
}

fn failed(what: &str, e: impl std::fmt::Display) -> BlockStoreError {
    format!("{what}: {e}").into()
}

impl PureReader {
    /// Opens the file `source` reads, `name` naming it in errors.
    pub fn open<S: Source + Send + Sync + 'static>(
        source: S,
        name: &str,
    ) -> Result<Self, BlockStoreError> {
        let mut head = vec![0; 128.min(source.len() as usize)];
        source
            .read_at(0, &mut head)
            .map_err(|e| failed(&format!("Failed to read '{name}'"), e))?;
        if let Some(why) = refusal(&head) {
            return Err(why.into());
        }
        let matlab = head.starts_with(b"MATLAB 7.3 MAT-file");
        let properties = FileAccessProperties::new()
            .with_metadata_cache(MetadataCacheConfig::new(METADATA_CACHE))
            .with_chunk_cache(ChunkCacheConfig::from_h5p_cache(521, CHUNK_CACHE));
        let file = File::from_source_with_options(source, properties)
            .map_err(|e| failed(&format!("Failed to open '{name}' as NetCDF-4/HDF5"), e))?;
        let (globals, entries) = match matlab {
            true => mat::catalog(&file),
            false => netcdf4::catalog(&file),
        }
        .map_err(|e| failed(&format!("Failed to read '{name}'"), e))?;
        let by_path = (entries.iter().enumerate())
            .map(|(i, e)| (e.var.path.clone(), i))
            .collect();
        Ok(Self {
            file,
            globals,
            entries,
            by_path,
        })
    }

    /// Opens the file at `path` on disk.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_path(path: &str) -> Result<Self, BlockStoreError> {
        let handle = std::fs::File::open(path).map_err(|e| failed(path, e))?;
        let source = hdf5_pure::ReadSeekSource::new(handle).map_err(|e| failed(path, e))?;
        Self::open(source, path)
    }

    fn dataset(&self, var: &NcVar) -> Result<hdf5_pure::Dataset, BlockStoreError> {
        let entry = (self.by_path.get(&var.path))
            .and_then(|&i| self.entries.get(i))
            .ok_or_else(|| format!("Variable '{}' not found", var.path))?;
        (self.file.dataset(&entry.dataset)).map_err(|e| failed(&var.path, e))
    }
}

impl NcReader for PureReader {
    fn global_attributes(&self) -> HashMap<String, String> {
        self.globals.clone()
    }

    fn variables(&self) -> Vec<NcVar> {
        self.entries.iter().map(|e| e.var.clone()).collect()
    }

    fn variable(&self, path: &str) -> Option<NcVar> {
        let i = *self.by_path.get(path.trim_matches('/'))?;
        Some(self.entries.get(i)?.var.clone())
    }

    fn read(&self, var: &NcVar, start: &[usize], count: &[usize]) -> Result<Raw, BlockStoreError> {
        let ds = self.dataset(var)?;
        let start: Vec<u64> = start.iter().map(|&s| s as u64).collect();
        let count: Vec<u64> = count.iter().map(|&c| c as u64).collect();
        let what = || format!("Failed reading '{}'", var.path);
        Ok(match var.data_type {
            "float32" => Raw::F32(
                ds.read_f32_region(&start, &count)
                    .map_err(|e| failed(&what(), e))?,
            ),
            _ => Raw::F64(
                ds.read_f64_region(&start, &count)
                    .map_err(|e| failed(&what(), e))?,
            ),
        })
    }

    fn labels(&self, var: &NcVar) -> Option<Vec<String>> {
        let ds = self.dataset(var).ok()?;
        let labels: Vec<String> = match (var.data_type, var.dims.as_slice()) {
            ("string", [_]) => ds
                .read_string()
                .ok()?
                .iter()
                .map(|s| clean_label(s))
                .collect(),
            ("char", [(_, rows), (_, width)]) if *width > 0 => {
                let bytes = ds.read_raw().ok()?;
                (bytes.chunks(*width).take(*rows))
                    .map(|row| clean_label(&String::from_utf8_lossy(c_text(row))))
                    .collect()
            }
            _ => return None,
        };
        (!labels.is_empty()).then_some(labels)
    }
}

/// The type name netCDF-C gives `dtype` (`size`: its element's bytes, telling `NC_CHAR`,
/// one-byte text, from strings).
pub(super) fn type_name(dtype: &hdf5_pure::DType, size: u32) -> &'static str {
    use hdf5_pure::DType as D;
    match dtype {
        D::F32 => "float32",
        D::F64 => "float64",
        D::I8 => "int8",
        D::I16 => "int16",
        D::I32 => "int32",
        D::I64 => "int64",
        D::U8 => "uint8",
        D::U16 => "uint16",
        D::U32 => "uint32",
        D::U64 => "uint64",
        D::String if size == 1 => "char",
        D::String | D::VariableLengthString => "string",
        D::Compound(_) => "compound",
        D::Enum(_) => "enum",
        _ => "opaque",
    }
}
