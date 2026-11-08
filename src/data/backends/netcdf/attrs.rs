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

/// Helper to convert a NetCDF `AttributeValue` into an `f64` for scale/offset/fill calculations.
pub fn attribute_value_to_f64(attr: &AttributeValue) -> Option<f64> {
    match attr {
        AttributeValue::Double(d) => Some(*d),
        AttributeValue::Float(f) => Some(*f as f64),
        AttributeValue::Int(i) => Some(*i as f64),
        AttributeValue::Uint(u) => Some(*u as f64),
        AttributeValue::Short(s) => Some(*s as f64),
        AttributeValue::Ushort(u) => Some(*u as f64),
        AttributeValue::Schar(i) => Some(*i as f64),
        AttributeValue::Uchar(u) => Some(*u as f64),
        AttributeValue::Longlong(l) => Some(*l as f64),
        AttributeValue::Ulonglong(u) => Some(*u as f64),
        AttributeValue::Doubles(v) => v.first().copied(),
        AttributeValue::Floats(v) => v.first().map(|&f| f as f64),
        AttributeValue::Ints(v) => v.first().map(|&i| i as f64),
        AttributeValue::Uints(v) => v.first().map(|&u| u as f64),
        AttributeValue::Shorts(v) => v.first().map(|&s| s as f64),
        AttributeValue::Ushorts(v) => v.first().map(|&u| u as f64),
        AttributeValue::Schars(v) => v.first().map(|&i| i as f64),
        AttributeValue::Uchars(v) => v.first().map(|&u| u as f64),
        AttributeValue::Longlongs(v) => v.first().map(|&l| l as f64),
        AttributeValue::Ulonglongs(v) => v.first().map(|&u| u as f64),
        AttributeValue::Str(s) => s.trim().parse::<f64>().ok(),
        AttributeValue::Strs(v) => v.first().and_then(|s| s.trim().parse::<f64>().ok()),
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
    DataCalibration::from_numbers(|name| attribute_value_to_f64(&var.attribute_value(name)?.ok()?))
}
