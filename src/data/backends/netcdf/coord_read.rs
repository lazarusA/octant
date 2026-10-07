//! Reading NetCDF coordinate variables: numbers as calibrated `f64`, text (`NC_STRING`, or
//! `NC_CHAR` shaped `(dim, strlen)`) as labels.

use netcdf::types::{FloatType, IntType, NcVariableType};
use netcdf::{Extent, Extents};

use super::attrs::extract_variable_calibration;
use crate::data::CoordValues;
use crate::data::blocks::BlockStoreError;

/// Reads `extents` of a numeric variable as `f64`, applying CF scale, offset and masking.
pub fn read_numbers_f64(
    var: &netcdf::Variable<'_>,
    extents: &Extents,
) -> Result<Vec<f64>, BlockStoreError> {
    macro_rules! read {
        ($t:ty) => {{
            let raw: Vec<$t> = var
                .get_values::<$t, _>(extents)
                .map_err(|e| format!("Failed reading {} coordinate: {e}", stringify!($t)))?;
            raw.into_iter().map(|v| v as f64).collect::<Vec<f64>>()
        }};
    }
    let mut values = match var.vartype() {
        NcVariableType::Float(FloatType::F64) => read!(f64),
        NcVariableType::Float(FloatType::F32) => read!(f32),
        NcVariableType::Int(IntType::I64) => read!(i64),
        NcVariableType::Int(IntType::I32) => read!(i32),
        NcVariableType::Int(IntType::I16) => read!(i16),
        NcVariableType::Int(IntType::I8) => read!(i8),
        NcVariableType::Int(IntType::U64) => read!(u64),
        NcVariableType::Int(IntType::U32) => read!(u32),
        NcVariableType::Int(IntType::U16) => read!(u16),
        NcVariableType::Int(IntType::U8) => read!(u8),
        other => return Err(format!("Coordinate type is not numeric: {other:?}").into()),
    };
    let calibration = extract_variable_calibration(var);
    if calibration.has_transformation() {
        for v in &mut values {
            *v = calibration.transform_f64(*v);
        }
    }
    Ok(values)
}

/// The whole of a 1D numeric coordinate variable, kept as start and step when evenly spaced.
pub fn read_number_coordinate(var: &netcdf::Variable<'_>) -> Option<CoordValues> {
    let [dim] = var.dimensions() else {
        return None;
    };
    let extents = Extents::from(vec![Extent::SliceCount {
        start: 0,
        count: dim.len(),
        stride: 1,
    }]);
    let values = read_numbers_f64(var, &extents).ok()?;
    let f32_source = var.vartype() == NcVariableType::Float(FloatType::F32);
    CoordValues::from_values(values, f32_source)
}

/// The labels of a text coordinate and the dimension they label: one per element of a 1D
/// `NC_STRING` variable, or one per row of an `NC_CHAR` variable shaped `(dim, strlen)`.
pub fn read_label_coordinate(var: &netcdf::Variable<'_>) -> Option<(String, Vec<String>)> {
    let dims = var.dimensions();
    let labels = match (var.vartype(), dims) {
        (NcVariableType::String, [dim]) => (0..dim.len())
            .map(|i| var.get_string([i]).ok().map(|s| clean_label(&s)))
            .collect::<Option<Vec<_>>>()?,
        (NcVariableType::Char, [dim, strlen]) if strlen.len() > 0 => {
            let bytes = var.get_raw_values(..).ok()?;
            let rows = bytes.chunks(strlen.len()).take(dim.len());
            rows.map(|row| clean_label(&String::from_utf8_lossy(row)))
                .collect()
        }
        _ => return None,
    };
    let dim = dims.first()?.name();
    (!labels.is_empty()).then_some((dim, labels))
}

/// Group paths from `group` up to the root: `a/b`, `a`, then `` (the root).
pub fn group_ancestors(group: &str) -> impl Iterator<Item = &str> {
    let group = group.trim_matches('/');
    std::iter::successors(Some(group), |g| {
        (!g.is_empty()).then(|| g.rsplit_once('/').map_or("", |(parent, _)| parent))
    })
}

/// The group path of a variable path: `a/b/var` lives in `a/b`.
pub fn group_of(var_path: &str) -> &str {
    var_path
        .trim_matches('/')
        .rsplit_once('/')
        .map_or("", |(group, _)| group)
}

/// Strips the NUL padding of fixed-width strings and surrounding whitespace.
fn clean_label(label: &str) -> String {
    label
        .trim_matches(|c: char| c == '\0' || c.is_whitespace())
        .to_string()
}
