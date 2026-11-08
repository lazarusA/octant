//! netCDF-4's conventions over HDF5, read as netCDF-C reads them: a variable's dimensions
//! are the ids in its `_Netcdf4Coordinates`, each the dimension-scale dataset holding that
//! `_Netcdf4Dimid`; scales netCDF-C made only to hold a dimension (`NAME` "This is a netCDF
//! dimension but not a netCDF variable") are no variables; bookkeeping attributes are
//! hidden. Plain HDF5 files without them get dimensions by their scales' lengths, else
//! `dim_N`.

use std::collections::HashMap;

use hdf5_pure::{AttrValue, File, Group};

use super::attrs::{calibration, ints, text, texts};
use super::{Entry, report_left_out, shape_of, type_name};
use crate::data::backends::netcdf::coord_read::{group_ancestors, group_of};
use crate::data::backends::netcdf::reader::{NcVar, join};
use crate::i18n::DatasetKey;

const PHONY: &str = "This is a netCDF dimension but not a netCDF variable";
/// netCDF-4's dataset name for a variable named like a dimension it does not hold.
const NON_COORD: &str = "_nc4_non_coord_";

/// netCDF-4 bookkeeping netCDF-C never shows.
fn hidden(name: &str) -> bool {
    matches!(
        name,
        "CLASS"
            | "NAME"
            | "REFERENCE_LIST"
            | "DIMENSION_LIST"
            | "_Netcdf4Coordinates"
            | "_Netcdf4Dimid"
            | "_nc3_strict"
            | "_NCProperties"
            | "_IsNetcdf4"
            | "_SuperblockVersion"
    )
}

/// A dataset as found, before its dimensions are named.
struct Found {
    path: String,
    dataset: String,
    shape: Vec<usize>,
    data_type: &'static str,
    attrs: HashMap<String, AttrValue>,
}

impl Found {
    fn text(&self, name: &str) -> Option<String> {
        self.attrs.get(name).and_then(text)
    }

    fn is_scale(&self) -> bool {
        self.text("CLASS").as_deref() == Some("DIMENSION_SCALE")
    }
}

/// The root's attributes and every variable of `file`.
pub(super) fn catalog(
    file: &File,
) -> Result<(HashMap<String, String>, Vec<Entry>), hdf5_pure::Error> {
    let root = file.root();
    let globals = texts(&root.attrs()?, hidden);
    let (mut found, mut too_large) = (Vec::new(), Vec::new());
    walk(&root, "", &mut found, &mut too_large)?;
    report_left_out(DatasetKey::VariablesTooLargeForBrowser, &too_large);
    let dimids: HashMap<i64, String> = (found.iter())
        .filter(|f| f.is_scale())
        .filter_map(|f| {
            Some((
                *f.attrs.get("_Netcdf4Dimid").and_then(ints)?.first()?,
                leaf(&f.path),
            ))
        })
        .collect();
    let scales: Vec<(&str, String, usize)> = (found.iter())
        .filter(|f| f.is_scale() && f.shape.len() == 1)
        .map(|f| (group_of(&f.path), leaf(&f.path), f.shape[0]))
        .collect();
    let phony = |f: &Found| f.text("NAME").is_some_and(|n| n.starts_with(PHONY));
    let entries = (found.iter())
        .filter(|f| !phony(f))
        .map(|f| Entry {
            var: NcVar {
                path: f.path.clone(),
                data_type: f.data_type,
                dims: dimensions(f, &dimids, &scales),
                attributes: texts(&f.attrs, hidden),
                calibration: calibration(&f.attrs),
            },
            dataset: f.dataset.clone(),
        })
        .collect();
    Ok((globals, entries))
}

fn leaf(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_owned()
}

/// The root's datasets, then each group's followed by its subgroups'; those whose
/// lengths do not fit `usize` (the browser's 32 bits) into `too_large`.
fn walk(
    group: &Group,
    prefix: &str,
    out: &mut Vec<Found>,
    too_large: &mut Vec<String>,
) -> Result<(), hdf5_pure::Error> {
    for (name, ds) in group.iter_datasets()? {
        let Some(shape) = shape_of(&ds)? else {
            too_large.push(join(prefix, &name));
            continue;
        };
        let size = ds.datatype()?.type_size();
        out.push(Found {
            path: join(prefix, name.strip_prefix(NON_COORD).unwrap_or(&name)),
            dataset: join(prefix, &name),
            shape,
            data_type: type_name(&ds.dtype()?, size),
            attrs: ds.attrs()?,
        });
    }
    for (name, sub) in group.iter_groups()? {
        walk(&sub, &join(prefix, &name), out, too_large)?;
    }
    Ok(())
}

/// Each dimension's name and length: from `_Netcdf4Coordinates`, a scale's own name, or
/// the one scale of its length up the group chain; else `dim_N`.
fn dimensions(
    f: &Found,
    dimids: &HashMap<i64, String>,
    scales: &[(&str, String, usize)],
) -> Vec<(String, usize)> {
    let ids = f.attrs.get("_Netcdf4Coordinates").and_then(ints);
    let by_id = |i: usize| Some(dimids.get(ids.as_ref()?.get(i)?)?.clone());
    let own = (f.is_scale() && f.shape.len() == 1).then(|| leaf(&f.path));
    let by_length = |len: usize| {
        let mut visible = (scales.iter()).filter(|(group, _, l)| {
            *l == len && group_ancestors(group_of(&f.path)).any(|g| g == *group)
        });
        match (visible.next(), visible.next()) {
            (Some((_, name, _)), None) => Some(name.clone()),
            _ => None,
        }
    };
    (f.shape.iter().enumerate())
        .map(|(i, &len)| {
            let name = (by_id(i).or_else(|| own.clone()).or_else(|| by_length(len)))
                .unwrap_or_else(|| format!("dim_{i}"));
            (name, len)
        })
        .collect()
}
