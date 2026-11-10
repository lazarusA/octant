//! MATLAB v7.3 conventions over HDF5: numeric and `logical` datasets are variables
//! (`MATLAB_class`), struct groups are groups; `#refs#` and `#subsystem#` hold the internals
//! of cells and objects and are hidden. Text, cells, empty and sparse arrays, complex numbers
//! and objects are left out, named in one warning, as are arrays too large for the
//! browser's 32 bits in another. MATLAB stores arrays column-major, so HDF5 holds the
//! shape reversed: values keep HDF5's order and the dimensions are named for MATLAB's
//! (`dim_1` is the last stored one).

use std::collections::HashMap;

use hdf5_pure::mat::MatClass;
use hdf5_pure::{AttrValue, DType, Dataset, File, Group};

use super::attrs::{calibration, text, texts};
use super::{Entry, report_left_out, shape_of, type_name};
use crate::data::backends::netcdf::reader::{NcVar, join};
use crate::i18n::DatasetKey;

/// Every numeric variable of `file`; MATLAB files have no global attributes.
pub(super) fn catalog(
    file: &File,
) -> Result<(HashMap<String, String>, Vec<Entry>), hdf5_pure::Error> {
    let mut found = Found::default();
    walk(&file.root(), "", &mut found)?;
    report_left_out(DatasetKey::MatArraysLeftOut, &found.unsupported);
    report_left_out(DatasetKey::VariablesTooLargeForBrowser, &found.too_large);
    Ok((HashMap::new(), found.entries))
}

fn class_of(attrs: &HashMap<String, AttrValue>) -> Option<MatClass> {
    MatClass::parse(&text(attrs.get("MATLAB_class")?)?).ok()
}

/// The variables found, and the paths of the arrays left out.
#[derive(Default)]
struct Found {
    entries: Vec<Entry>,
    /// Arrays of a kind Octant does not show.
    unsupported: Vec<String>,
    /// Arrays whose lengths do not fit `usize` (the browser's 32 bits).
    too_large: Vec<String>,
}

/// Collects `group`'s variables, and what it leaves out, into `found`.
fn walk(group: &Group, prefix: &str, found: &mut Found) -> Result<(), hdf5_pure::Error> {
    for (name, ds) in group.iter_datasets()? {
        let path = join(prefix, &name);
        let attrs = ds.attrs()?;
        if let Some(why) = why_left_out(&ds, &attrs)? {
            log::info!("[mat] '{path}' left out: {why}");
            found.unsupported.push(path);
            continue;
        }
        match shape_of(&ds)? {
            Some(shape) => found.entries.push(variable(&ds, path, shape, &attrs)?),
            None => found.too_large.push(path),
        }
    }
    for (name, sub) in group.iter_groups()? {
        let struct_group = class_of(&sub.attrs()?) == Some(MatClass::Struct);
        if name.starts_with('#') || !struct_group {
            continue;
        }
        walk(&sub, &join(prefix, &name), found)?;
    }
    Ok(())
}

/// Why a dataset is no variable, or `None` for a numeric or `logical` array.
fn why_left_out(
    ds: &Dataset,
    attrs: &HashMap<String, AttrValue>,
) -> Result<Option<&'static str>, hdf5_pure::Error> {
    if attrs.contains_key("MATLAB_empty") {
        return Ok(Some("empty"));
    }
    if attrs.contains_key("MATLAB_sparse") {
        return Ok(Some("sparse"));
    }
    Ok(match class_of(attrs) {
        Some(MatClass::Char) => Some("text"),
        Some(MatClass::Cell) => Some("cell array"),
        Some(_) if matches!(ds.dtype()?, DType::Compound(_)) => Some("complex"),
        Some(MatClass::Struct) | None => Some("not a numeric array"),
        Some(_) => None,
    })
}

fn variable(
    ds: &Dataset,
    path: String,
    shape: Vec<usize>,
    attrs: &HashMap<String, AttrValue>,
) -> Result<Entry, hdf5_pure::Error> {
    let rank = shape.len();
    let dims = (shape.into_iter().enumerate())
        .map(|(i, len)| (format!("dim_{}", rank - i), len))
        .collect();
    let var = NcVar {
        path: path.clone(),
        data_type: type_name(&ds.dtype()?, ds.datatype()?.type_size()),
        dims,
        attributes: texts(attrs, |_| false).into(),
        calibration: calibration(attrs),
    };
    Ok(Entry { var, dataset: path })
}
