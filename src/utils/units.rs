/// Calculate uncompressed file/variable size in bytes based on shape and data type string.
pub fn calculate_variable_size_bytes(shape: &[u64], data_type: &str) -> u64 {
    let element_count: u64 = shape.iter().product();
    let bytes_per_elem: u64 = match data_type.to_lowercase().as_str() {
        "float64" | "double" | "f64" | "i64" | "u64" | "int64" | "uint64" => 8,
        "float32" | "float" | "f32" | "i32" | "u32" | "int32" | "uint32" => 4,
        "float16" | "f16" | "i16" | "u16" | "int16" | "uint16" => 2,
        "i8" | "u8" | "int8" | "uint8" | "bool" => 1,
        _ => 4,
    };
    element_count.saturating_mul(bytes_per_elem)
}

/// Map of CF time unit names and short aliases to milliseconds per unit.
pub fn unit_to_milliseconds(unit: &str) -> Option<u64> {
    let clean_unit = unit.trim().to_lowercase();
    let singular = if clean_unit.ends_with('s') {
        clean_unit.trim_end_matches('s')
    } else {
        &clean_unit
    };

    match singular {
        "millisecond" | "msec" | "ms" => Some(1),
        "second" | "sec" | "s" => Some(1_000),
        "minute" | "min" => Some(60 * 1_000),
        "hour" | "hr" | "h" => Some(60 * 60 * 1_000),
        "day" | "d" => Some(24 * 60 * 60 * 1_000),
        _ => None,
    }
}

/// Parses CF time unit string (e.g. "seconds since 1970-01-01" or bare duration units "hours").
/// Returns `(scale_in_ms, offset_timestamp_ms)`.
pub fn parse_time_unit(units_str: Option<&str>) -> (u64, i64) {
    let units = match units_str {
        Some(u) if !u.trim().is_empty() && u != "Default" => u.trim(),
        _ => return (1, 0),
    };

    let lower = units.to_lowercase();
    if let Some((unit_part, ref_date_str)) = lower.split_once(" since ") {
        let _ref_date = ref_date_str.trim();
        let scale = unit_to_milliseconds(unit_part.trim()).unwrap_or(1);
        // Simple offset calculation or timestamp parser
        (scale, 0)
    } else if let Some(scale) = unit_to_milliseconds(&lower) {
        (scale, 0)
    } else {
        (1, 0)
    }
}
