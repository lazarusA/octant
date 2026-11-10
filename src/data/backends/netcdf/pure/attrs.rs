//! `hdf5-pure` attribute values as netCDF-C shows them: one value as itself, several as a
//! list, text as is; and the numbers of CF calibration.

use std::collections::HashMap;

use hdf5_pure::AttrValue;

use crate::data::DataCalibration;
use crate::data::backends::netcdf::reader::list;

/// The text of `value`, or `None` for kinds netCDF has no text for.
pub fn text(value: &AttrValue) -> Option<String> {
    use AttrValue as A;
    Some(match value {
        A::F32(v) => v.to_string(),
        A::F64(v) => v.to_string(),
        A::I8(v) => v.to_string(),
        A::I16(v) => v.to_string(),
        A::I32(v) => v.to_string(),
        A::I64(v) => v.to_string(),
        A::U8(v) => v.to_string(),
        A::U16(v) => v.to_string(),
        A::U32(v) => v.to_string(),
        A::U64(v) => v.to_string(),
        A::F32Array(v) => list(v),
        A::F64Array(v) => list(v),
        A::I8Array(v) => list(v),
        A::I16Array(v) => list(v),
        A::I32Array(v) => list(v),
        A::I64Array(v) => list(v),
        A::U8Array(v) => list(v),
        A::U16Array(v) => list(v),
        A::U32Array(v) => list(v),
        A::U64Array(v) => list(v),
        A::String(s) | A::AsciiString(s) | A::VarLenString(s) | A::VarLenAsciiString(s) => {
            s.clone()
        }
        A::StringSized { value, .. } | A::AsciiStringSized { value, .. } => value.clone(),
        A::StringArray(v)
        | A::AsciiStringArray(v)
        | A::VarLenAsciiCharArray(v)
        | A::VarLenStringArray(v)
        | A::VarLenAsciiStringArray(v)
        | A::StringArraySized { values: v, .. }
        | A::AsciiStringArraySized { values: v, .. } => v.join(", "),
        _ => return None,
    })
}

/// The `i`-th number of `value`, or of its text (numbers separated by commas).
fn nth_number(value: &AttrValue, i: usize) -> Option<f64> {
    use AttrValue as A;
    let one = |v: f64| (i == 0).then_some(v);
    match value {
        A::F32(v) => one(f64::from(*v)),
        A::F64(v) => one(*v),
        A::I8(v) => one(f64::from(*v)),
        A::I16(v) => one(f64::from(*v)),
        A::I32(v) => one(f64::from(*v)),
        A::I64(v) => one(*v as f64),
        A::U8(v) => one(f64::from(*v)),
        A::U16(v) => one(f64::from(*v)),
        A::U32(v) => one(f64::from(*v)),
        A::U64(v) => one(*v as f64),
        A::F32Array(v) => v.get(i).map(|&x| f64::from(x)),
        A::F64Array(v) => v.get(i).copied(),
        A::I8Array(v) => v.get(i).map(|&x| f64::from(x)),
        A::I16Array(v) => v.get(i).map(|&x| f64::from(x)),
        A::I32Array(v) => v.get(i).map(|&x| f64::from(x)),
        A::I64Array(v) => v.get(i).map(|&x| x as f64),
        A::U8Array(v) => v.get(i).map(|&x| f64::from(x)),
        A::U16Array(v) => v.get(i).map(|&x| f64::from(x)),
        A::U32Array(v) => v.get(i).map(|&x| f64::from(x)),
        A::U64Array(v) => v.get(i).map(|&x| x as f64),
        other => text(other)?.split(',').nth(i)?.trim().parse().ok(),
    }
}

/// The integers of `value` (`_Netcdf4Coordinates`, `_Netcdf4Dimid`).
pub fn ints(value: &AttrValue) -> Option<Vec<i64>> {
    use AttrValue as A;
    Some(match value {
        A::I32(v) => vec![i64::from(*v)],
        A::I32Array(v) => v.iter().map(|&x| i64::from(x)).collect(),
        A::I64(v) => vec![*v],
        A::I64Array(v) => v.clone(),
        A::U32(v) => vec![i64::from(*v)],
        A::U32Array(v) => v.iter().map(|&x| i64::from(x)).collect(),
        _ => return None,
    })
}

/// Every attribute of `raw` as text, but those `hidden` names.
pub fn texts(
    raw: &HashMap<String, AttrValue>,
    hidden: fn(&str) -> bool,
) -> HashMap<String, String> {
    (raw.iter())
        .filter(|(name, _)| !hidden(name))
        .filter_map(|(name, value)| Some((name.clone(), text(value)?)))
        .collect()
}

/// CF scale, offset, fill and valid range from `raw`, read as netCDF-C's reader does.
pub fn calibration(raw: &HashMap<String, AttrValue>) -> DataCalibration {
    DataCalibration::from_numbers(|name, i| nth_number(raw.get(name)?, i))
}
