//! The desktop's reader: netCDF-C (classic, 64-bit offset, CDF-5, netCDF-4/HDF5).

use std::collections::HashMap;

use netcdf::types::{FloatType, IntType, NcVariableType};
use netcdf::{Extent, Extents};

use super::attrs::{
    extract_global_attributes, extract_variable_attributes, extract_variable_calibration,
    var_type_to_string,
};
use super::coord_read::{c_text, clean_label};
use super::reader::{NcReader, NcVar, Raw};
use crate::data::blocks::BlockStoreError;

pub struct CReader {
    file: netcdf::File,
}

impl CReader {
    pub fn open(path: &str) -> Result<Self, BlockStoreError> {
        let file =
            netcdf::open(path).map_err(|e| format!("Failed to open NetCDF file '{path}': {e}"))?;
        Ok(Self { file })
    }
}

fn describe(var: &netcdf::Variable<'_>, path: String) -> NcVar {
    NcVar {
        path,
        data_type: var_type_to_string(&var.vartype()),
        dims: (var.dimensions().iter())
            .map(|d| (d.name(), d.len()))
            .collect(),
        attributes: extract_variable_attributes(var),
        calibration: extract_variable_calibration(var),
    }
}

fn join(prefix: &str, name: &str) -> String {
    match prefix {
        "" => name.to_owned(),
        _ => format!("{prefix}/{name}"),
    }
}

fn collect_group(group: &netcdf::Group<'_>, prefix: &str, out: &mut Vec<NcVar>) {
    out.extend(
        group
            .variables()
            .map(|v| describe(&v, join(prefix, &v.name()))),
    );
    for sub in group.groups() {
        collect_group(&sub, &join(prefix, &sub.name()), out);
    }
}

impl NcReader for CReader {
    fn global_attributes(&self) -> HashMap<String, String> {
        extract_global_attributes(&self.file)
    }

    fn variables(&self) -> Vec<NcVar> {
        let mut out: Vec<NcVar> = (self.file.variables())
            .map(|v| describe(&v, v.name()))
            .collect();
        if let Ok(groups) = self.file.groups() {
            for group in groups {
                collect_group(&group, &group.name(), &mut out);
            }
        }
        out
    }

    fn variable(&self, path: &str) -> Option<NcVar> {
        let clean = path.trim_matches('/').to_owned();
        with_netcdf_variable(&self.file, path, |v| Ok(describe(v, clean))).ok()
    }

    fn read(&self, var: &NcVar, start: &[usize], count: &[usize]) -> Result<Raw, BlockStoreError> {
        let extents = Extents::from(
            (start.iter().zip(count))
                .map(|(&start, &count)| Extent::SliceCount {
                    start,
                    count,
                    stride: 1,
                })
                .collect::<Vec<_>>(),
        );
        with_netcdf_variable(&self.file, &var.path, |v| read_raw(v, &extents))
    }

    fn labels(&self, var: &NcVar) -> Option<Vec<String>> {
        with_netcdf_variable(&self.file, &var.path, |v| Ok(read_labels(v)))
            .ok()
            .flatten()
    }
}

fn read_raw(var: &netcdf::Variable<'_>, extents: &Extents) -> Result<Raw, BlockStoreError> {
    macro_rules! read {
        ($t:ty) => {{
            let raw: Vec<$t> = var
                .get_values::<$t, _>(extents)
                .map_err(|e| format!("Failed reading {} hyperslab: {e}", stringify!($t)))?;
            Raw::F64(raw.into_iter().map(|v| v as f64).collect())
        }};
    }
    Ok(match var.vartype() {
        NcVariableType::Float(FloatType::F32) => Raw::F32(
            var.get_values::<f32, _>(extents)
                .map_err(|e| format!("Failed reading float32 hyperslab: {e}"))?,
        ),
        NcVariableType::Float(FloatType::F64) => read!(f64),
        NcVariableType::Int(IntType::I64) => read!(i64),
        NcVariableType::Int(IntType::I32) => read!(i32),
        NcVariableType::Int(IntType::I16) => read!(i16),
        NcVariableType::Int(IntType::I8) => read!(i8),
        NcVariableType::Int(IntType::U64) => read!(u64),
        NcVariableType::Int(IntType::U32) => read!(u32),
        NcVariableType::Int(IntType::U16) => read!(u16),
        NcVariableType::Int(IntType::U8) | NcVariableType::Char => read!(u8),
        other => return Err(format!("Unsupported NetCDF variable type: {other:?}").into()),
    })
}

/// One label per `NC_STRING` element or per `NC_CHAR` row.
fn read_labels(var: &netcdf::Variable<'_>) -> Option<Vec<String>> {
    let labels: Vec<String> = match (var.vartype(), var.dimensions()) {
        (NcVariableType::String, [dim]) => (0..dim.len())
            .map(|i| var.get_string([i]).ok().map(|s| clean_label(&s)))
            .collect::<Option<_>>()?,
        (NcVariableType::Char, [dim, strlen]) if strlen.len() > 0 => {
            let bytes = var.get_raw_values(..).ok()?;
            let rows = bytes.chunks(strlen.len()).take(dim.len());
            rows.map(|row| clean_label(&String::from_utf8_lossy(c_text(row))))
                .collect()
        }
        _ => return None,
    };
    (!labels.is_empty()).then_some(labels)
}

fn with_netcdf_variable<R>(
    file: &netcdf::File,
    name: &str,
    f: impl FnOnce(&netcdf::Variable) -> Result<R, BlockStoreError>,
) -> Result<R, BlockStoreError> {
    let clean = name.trim_matches('/');
    if let Some(var) = file.variable(clean) {
        return f(&var);
    }
    let segments: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    if let [first, rest @ ..] = segments.as_slice()
        && !rest.is_empty()
        && let Ok(Some(top)) = file.group(first)
    {
        return find_in_group(&top, rest, f);
    }
    Err(format!("Variable '{name}' not found in NetCDF file").into())
}

fn find_in_group<R>(
    group: &netcdf::Group,
    segments: &[&str],
    f: impl FnOnce(&netcdf::Variable) -> Result<R, BlockStoreError>,
) -> Result<R, BlockStoreError> {
    match segments {
        [name] => {
            if let Some(var) = group.variable(name) {
                return f(&var);
            }
        }
        [first, rest @ ..] => {
            if let Some(sub) = group.group(first) {
                return find_in_group(&sub, rest, f);
            }
        }
        [] => {}
    }
    Err(format!(
        "Variable '{}' not found in NetCDF group",
        segments.join("/")
    )
    .into())
}
