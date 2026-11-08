//! MATLAB v7.3 conventions over HDF5: numeric and `logical` datasets are variables
//! (`MATLAB_class`), struct groups are groups; `#refs#` and `#subsystem#` hold the internals
//! of cells and objects and are hidden. Text, cells, empty and sparse arrays, complex numbers
//! and objects are left out, named in one warning. MATLAB stores arrays column-major, so HDF5 holds the
//! shape reversed: values keep HDF5's order and the dimensions are named for MATLAB's
//! (`dim_1` is the last stored one).

use std::collections::HashMap;

use hdf5_pure::mat::MatClass;
use hdf5_pure::{AttrValue, DType, Dataset, File, Group};

use super::attrs::{calibration, text, texts};
use super::{Entry, type_name};
use crate::data::backends::netcdf::reader::{NcVar, join};
use crate::ui::toast::Severity;

/// Every numeric variable of `file`; MATLAB files have no global attributes.
pub(super) fn catalog(
    file: &File,
) -> Result<(HashMap<String, String>, Vec<Entry>), hdf5_pure::Error> {
    let (mut entries, mut left_out) = (Vec::new(), Vec::new());
    walk(&file.root(), "", &mut entries, &mut left_out)?;
    if !left_out.is_empty() {
        let title = crate::i18n::tr(crate::i18n::DatasetKey::MatArraysLeftOut);
        let title = crate::i18n::filled(title, &[&left_out.len()]);
        crate::ui::toast::report(Severity::Warning, &title, left_out.join(", "));
    }
    Ok((HashMap::new(), entries))
}

fn class_of(attrs: &HashMap<String, AttrValue>) -> Option<MatClass> {
    MatClass::parse(&text(attrs.get("MATLAB_class")?)?).ok()
}

/// Collects `group`'s variables into `entries`, and what it leaves out into `left_out`.
fn walk(
    group: &Group,
    prefix: &str,
    entries: &mut Vec<Entry>,
    left_out: &mut Vec<String>,
) -> Result<(), hdf5_pure::Error> {
    for (name, ds) in group.iter_datasets()? {
        let path = join(prefix, &name);
        let attrs = ds.attrs()?;
        match why_left_out(&ds, &attrs)? {
            Some(why) => {
                log::info!("[mat] '{path}' left out: {why}");
                left_out.push(path);
            }
            None => entries.push(variable(&ds, path, &attrs)?),
        }
    }
    for (name, sub) in group.iter_groups()? {
        let struct_group = class_of(&sub.attrs()?) == Some(MatClass::Struct);
        if name.starts_with('#') || !struct_group {
            continue;
        }
        walk(&sub, &join(prefix, &name), entries, left_out)?;
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
    attrs: &HashMap<String, AttrValue>,
) -> Result<Entry, hdf5_pure::Error> {
    let shape = ds.shape()?;
    let rank = shape.len();
    let dims = (shape.iter().enumerate())
        .map(|(i, &len)| (format!("dim_{}", rank - i), len as usize))
        .collect();
    let var = NcVar {
        path: path.clone(),
        data_type: type_name(&ds.dtype()?, ds.datatype()?.type_size()),
        dims,
        attributes: texts(attrs, |_| false),
        calibration: calibration(attrs),
    };
    Ok(Entry { var, dataset: path })
}
