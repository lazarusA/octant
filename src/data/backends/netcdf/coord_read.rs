//! Reading NetCDF coordinate variables: numbers as calibrated `f64`, text (`NC_STRING`, or
//! `NC_CHAR` shaped `(dim, strlen)`) as labels.

use super::reader::{NcReader, NcVar, Raw};
use crate::data::CoordValues;
use crate::data::blocks::BlockStoreError;

/// Reads the window `start..start + count` of a numeric variable as `f64`, applying CF scale,
/// offset and masking.
pub fn read_numbers_f64(
    reader: &dyn NcReader,
    var: &NcVar,
    start: &[usize],
    count: &[usize],
) -> Result<Vec<f64>, BlockStoreError> {
    if matches!(var.data_type, "char" | "string") {
        return Err(format!("Coordinate type is not numeric: {}", var.data_type).into());
    }
    let mut values = match reader.read(var, start, count)? {
        Raw::F32(v) => v.into_iter().map(f64::from).collect(),
        Raw::F64(v) => v,
    };
    if var.calibration.has_transformation() {
        for v in &mut values {
            *v = var.calibration.transform_f64(*v);
        }
    }
    Ok(values)
}

/// The whole of a 1D numeric coordinate variable, kept as start and step when evenly spaced.
pub fn read_number_coordinate(reader: &dyn NcReader, var: &NcVar) -> Option<CoordValues> {
    let [(_, len)] = var.dims.as_slice() else {
        return None;
    };
    let values = read_numbers_f64(reader, var, &[0], &[*len]).ok()?;
    CoordValues::from_values(values, var.data_type == "float32")
}

/// The dimension a text variable would label, from its metadata alone: a 1D `NC_STRING`
/// variable, or an `NC_CHAR` variable shaped `(dim, strlen)`. `None` for anything else.
pub fn text_dimension(var: &NcVar) -> Option<&str> {
    match (var.data_type, var.dims.as_slice()) {
        ("string", [(dim, _)]) => Some(dim),
        ("char", [(dim, _), (_, strlen)]) if *strlen > 0 => Some(dim),
        _ => None,
    }
}

/// The text of a fixed-width C string: up to its first NUL, as padding or junk follows.
pub fn c_text(row: &[u8]) -> &[u8] {
    row.split(|&b| b == 0).next().unwrap_or_default()
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
pub fn clean_label(label: &str) -> String {
    label
        .trim_matches(|c: char| c == '\0' || c.is_whitespace())
        .to_string()
}
