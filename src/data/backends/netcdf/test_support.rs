//! Shared helpers for NetCDF tests: serialized file creation, self-removing paths, and
//! writing small files.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use netcdf::types::{NcTypeDescriptor, NcVariableType};

use super::inspect::inspect_netcdf_file;
use crate::data::metadata::DatasetMetadata;

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
    inspect_netcdf_file(nc.path()).expect("inspect")
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
