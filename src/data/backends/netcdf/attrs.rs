//! NetCDF attribute string formatting and variable calibration extraction.

use std::collections::HashMap;

use netcdf::AttributeValue;
use netcdf::types::{FloatType, IntType, NcVariableType};

use crate::data::DataCalibration;

/// Helper to convert a NetCDF `AttributeValue` into a displayable string.
pub fn attribute_value_to_string(attr: &AttributeValue) -> String {
    match attr {
        AttributeValue::Str(s) => s.clone(),
        AttributeValue::Float(f) => format!("{f}"),
        AttributeValue::Double(d) => format!("{d}"),
        AttributeValue::Schar(i) => format!("{i}"),
        AttributeValue::Uchar(u) => format!("{u}"),
        AttributeValue::Short(i) => format!("{i}"),
        AttributeValue::Ushort(u) => format!("{u}"),
        AttributeValue::Int(i) => format!("{i}"),
        AttributeValue::Uint(u) => format!("{u}"),
        AttributeValue::Longlong(i) => format!("{i}"),
        AttributeValue::Ulonglong(u) => format!("{u}"),
        AttributeValue::Floats(v) => format!("{v:?}"),
        AttributeValue::Doubles(v) => format!("{v:?}"),
        AttributeValue::Schars(v) => format!("{v:?}"),
        AttributeValue::Uchars(v) => format!("{v:?}"),
        AttributeValue::Shorts(v) => format!("{v:?}"),
        AttributeValue::Ushorts(v) => format!("{v:?}"),
        AttributeValue::Ints(v) => format!("{v:?}"),
        AttributeValue::Uints(v) => format!("{v:?}"),
        AttributeValue::Longlongs(v) => format!("{v:?}"),
        AttributeValue::Ulonglongs(v) => format!("{v:?}"),
        AttributeValue::Strs(v) => v.join(", "),
    }
}

/// The `i`-th number of `attr`, or of its text (numbers separated by commas).
fn nth_number(attr: &AttributeValue, i: usize) -> Option<f64> {
    let one = |v: f64| (i == 0).then_some(v);
    match attr {
        AttributeValue::Double(x) => one(*x),
        AttributeValue::Float(x) => one(*x as f64),
        AttributeValue::Int(x) => one(*x as f64),
        AttributeValue::Uint(x) => one(*x as f64),
        AttributeValue::Short(x) => one(*x as f64),
        AttributeValue::Ushort(x) => one(*x as f64),
        AttributeValue::Schar(x) => one(*x as f64),
        AttributeValue::Uchar(x) => one(*x as f64),
        AttributeValue::Longlong(x) => one(*x as f64),
        AttributeValue::Ulonglong(x) => one(*x as f64),
        AttributeValue::Doubles(v) => v.get(i).copied(),
        AttributeValue::Floats(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Ints(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Uints(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Shorts(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Ushorts(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Schars(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Uchars(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Longlongs(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Ulonglongs(v) => v.get(i).map(|&x| x as f64),
        AttributeValue::Str(s) => s.split(',').nth(i)?.trim().parse().ok(),
        AttributeValue::Strs(v) => v.get(i)?.trim().parse().ok(),
    }
}

/// Extracts all variable-level attributes as a string map.
pub fn extract_variable_attributes(var: &netcdf::Variable<'_>) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    for attr in var.attributes() {
        if let Ok(val) = attr.value() {
            attributes.insert(attr.name().to_string(), attribute_value_to_string(&val));
        }
    }
    attributes
}

/// Extracts all dataset-level (global) attributes as a string map.
pub fn extract_global_attributes(file: &netcdf::File) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    for attr in file.attributes() {
        if let Ok(val) = attr.value() {
            attributes.insert(attr.name().to_string(), attribute_value_to_string(&val));
        }
    }
    attributes
}

/// Converts a NetCDF `NcVariableType` to a descriptive string for metadata.
pub fn var_type_to_string(vartype: &NcVariableType) -> &'static str {
    match vartype {
        NcVariableType::Float(FloatType::F32) => "float32",
        NcVariableType::Float(FloatType::F64) => "float64",
        NcVariableType::Int(IntType::I32) => "int32",
        NcVariableType::Int(IntType::I16) => "int16",
        NcVariableType::Int(IntType::I8) => "int8",
        NcVariableType::Int(IntType::I64) => "int64",
        NcVariableType::Int(IntType::U32) => "uint32",
        NcVariableType::Int(IntType::U16) => "uint16",
        NcVariableType::Int(IntType::U8) => "uint8",
        NcVariableType::Int(IntType::U64) => "uint64",
        NcVariableType::Char => "char",
        NcVariableType::String => "string",
        NcVariableType::Compound(_) => "compound",
        NcVariableType::Opaque(_) => "opaque",
        NcVariableType::Enum(_) => "enum",
        NcVariableType::Vlen(_) => "vlen",
    }
}

/// CF scale, offset, fill and valid range from `var`'s attributes.
pub fn extract_variable_calibration(var: &netcdf::Variable<'_>) -> DataCalibration {
    DataCalibration::from_numbers(|name, i| nth_number(&var.attribute_value(name)?.ok()?, i))
}
