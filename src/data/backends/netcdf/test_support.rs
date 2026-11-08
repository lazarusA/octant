//! Shared helpers for NetCDF tests: serialized file creation, self-removing paths, and
//! writing small files.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use netcdf::types::{NcTypeDescriptor, NcVariableType};

use super::NetCdfBlockStore;
use crate::data::blocks::{BlockStore, BlockStoreError};
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

/// The netCDF-C library is not reentrant: tests that create files hold this lock.
pub fn netcdf_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

/// A temporary `.nc` path, unique per process and name, removed when dropped.
pub struct TempNc(pub PathBuf);

impl TempNc {
    pub fn new(name: &str) -> Self {
        let file = format!("octant_{name}_{}.nc", std::process::id());
        Self(std::env::temp_dir().join(file))
    }

    pub fn path(&self) -> &str {
        self.0.to_str().unwrap_or_default()
    }
}

impl Drop for TempNc {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// An `NC_CHAR` element, for writing fixed-width text in tests.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct NcChar(u8);

// SAFETY: `NcChar` is a transparent byte, the storage of `NC_CHAR`.
unsafe impl NcTypeDescriptor for NcChar {
    fn type_descriptor() -> NcVariableType {
        NcVariableType::Char
    }
}

pub fn inspect(nc: &TempNc, build: impl FnOnce(&mut netcdf::FileMut)) -> DatasetMetadata {
    {
        let mut file = netcdf::create(&nc.0).expect("create netcdf file");
        build(&mut file);
    }
    Parity::open(nc.path())
        .expect("open")
        .inspect()
        .expect("inspect")
}

/// A file read by netCDF-C and by `hdf5-pure` (`pure/`) at once: every inspection and
/// block must come out the same from both, so the browser shows what the desktop shows.
pub struct Parity {
    c: NetCdfBlockStore,
    pure: NetCdfBlockStore,
}

impl Parity {
    pub fn open(path: &str) -> Result<Self, BlockStoreError> {
        Ok(Self {
            c: NetCdfBlockStore::open_local(path)?,
            pure: NetCdfBlockStore::open_pure(path)?,
        })
    }
}

/// What a reader's metadata says, in an order both readers share.
fn comparable(meta: &DatasetMetadata) -> Vec<String> {
    let mut lines: Vec<String> = (meta.variables.iter())
        .map(|v| {
            let mut attrs: Vec<_> = v.attributes.iter().collect();
            attrs.sort();
            format!(
                "{} {} {:?} {:?} {:?} {:?} {:?}",
                v.name, v.data_type, v.shape, v.dimension_names, v.units, v.long_name, attrs
            )
        })
        .collect();
    lines.sort();
    let mut coords: Vec<_> = meta.dimension_coordinates.iter().collect();
    coords.sort_by(|a, b| a.0.cmp(b.0));
    lines.push(format!("{} {coords:?}", meta.name));
    lines
}

/// A block as text, its values by their bits so missing values compare.
fn block_text(b: &OctantBlock) -> String {
    let bits: Vec<u32> = b.values.iter().map(|v| v.to_bits()).collect();
    let mut coords: Vec<_> = b.coordinates.iter().collect();
    coords.sort_by(|a, b| a.0.cmp(b.0));
    let mut attrs: Vec<_> = b.attributes.iter().collect();
    attrs.sort();
    format!(
        "{} {:?} {:?} {:?} {bits:?} {coords:?} {attrs:?} {:?}",
        b.variable_name, b.shape, b.dimension_names, b.origin, b.flipped_dims
    )
}

impl BlockStore for Parity {
    fn backend_name(&self) -> &str {
        self.c.backend_name()
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.c.variables()
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        let meta = self.c.inspect()?;
        let pure = self.pure.inspect()?;
        assert_eq!(
            comparable(&meta),
            comparable(&pure),
            "netCDF-C and hdf5-pure inspect apart"
        );
        Ok(meta)
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        let block = self.c.fetch_block(request)?;
        let pure = self.pure.fetch_block(request)?;
        assert_eq!(
            block_text(&block),
            block_text(&pure),
            "netCDF-C and hdf5-pure read apart"
        );
        Ok(block)
    }
}

pub fn put<T: NcTypeDescriptor + Copy>(
    file: &mut netcdf::FileMut,
    name: &str,
    dims: &[&str],
    v: &[T],
) {
    let mut var = file.add_variable::<T>(name, dims).expect("add variable");
    var.put_values(v, ..).expect("put values");
}

pub fn char_rows(rows: &[&str], width: usize) -> Vec<NcChar> {
    rows.iter()
        .flat_map(|r| {
            let mut bytes = r.as_bytes().to_vec();
            bytes.resize(width, 0);
            bytes.into_iter().map(NcChar)
        })
        .collect()
}
