//! Byte widths, memory metrics, and formatting helpers.

/// Returns the byte width of a data type string (e.g. "f64" -> 8, "f32" -> 4, "f16" -> 2, "i8" -> 1).
pub fn data_type_bytes(data_type: &str) -> u64 {
    let clean = data_type.trim();
    if clean.eq_ignore_ascii_case("float64")
        || clean.eq_ignore_ascii_case("double")
        || clean.eq_ignore_ascii_case("f64")
        || clean.eq_ignore_ascii_case("i64")
        || clean.eq_ignore_ascii_case("u64")
        || clean.eq_ignore_ascii_case("int64")
        || clean.eq_ignore_ascii_case("uint64")
    {
        8
    } else if clean.eq_ignore_ascii_case("float32")
        || clean.eq_ignore_ascii_case("float")
        || clean.eq_ignore_ascii_case("f32")
        || clean.eq_ignore_ascii_case("i32")
        || clean.eq_ignore_ascii_case("u32")
        || clean.eq_ignore_ascii_case("int32")
        || clean.eq_ignore_ascii_case("uint32")
    {
        4
    } else if clean.eq_ignore_ascii_case("float16")
        || clean.eq_ignore_ascii_case("f16")
        || clean.eq_ignore_ascii_case("i16")
        || clean.eq_ignore_ascii_case("u16")
        || clean.eq_ignore_ascii_case("int16")
        || clean.eq_ignore_ascii_case("uint16")
    {
        2
    } else if clean.eq_ignore_ascii_case("i8")
        || clean.eq_ignore_ascii_case("u8")
        || clean.eq_ignore_ascii_case("int8")
        || clean.eq_ignore_ascii_case("uint8")
        || clean.eq_ignore_ascii_case("bool")
    {
        1
    } else {
        4
    }
}

/// Formats a byte count into a human-readable string (e.g. "500 B", "50 KB", "50 MB", "100 GB", "1.5 TB").
pub fn format_byte_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        let tb = b / TB;
        if (tb.fract() * 10.0).round() == 0.0 {
            format!("{:.0} TB", tb)
        } else {
            format!("{:.1} TB", tb)
        }
    } else if b >= GB {
        let gb = b / GB;
        if (gb.fract() * 10.0).round() == 0.0 {
            format!("{:.0} GB", gb)
        } else {
            format!("{:.1} GB", gb)
        }
    } else if b >= MB {
        let mb = b / MB;
        if (mb.fract() * 10.0).round() == 0.0 {
            format!("{:.0} MB", mb)
        } else {
            format!("{:.1} MB", mb)
        }
    } else if b >= KB {
        let kb = b / KB;
        if (kb.fract() * 10.0).round() == 0.0 {
            format!("{:.0} KB", kb)
        } else {
            format!("{:.1} KB", kb)
        }
    } else {
        format!("{} B", bytes)
    }
}

/// Formats a large element count with SI metric prefixes (e.g., "6.48M", "500K").
pub fn format_count_metric(count: usize) -> String {
    if count >= 1_000_000 {
        let m = count as f64 / 1_000_000.0;
        if (m.fract() * 100.0).round() == 0.0 {
            format!("{:.0}M", m)
        } else {
            format!("{:.2}M", m)
        }
    } else if count >= 1_000 {
        let k = count as f64 / 1_000.0;
        if (k.fract() * 10.0).round() == 0.0 {
            format!("{:.0}K", k)
        } else {
            format!("{:.1}K", k)
        }
    } else {
        count.to_string()
    }
}

/// Calculate uncompressed file/variable size in bytes based on shape and data type string.
pub fn calculate_variable_size_bytes(shape: &[u64], data_type: &str) -> u64 {
    let element_count: u64 = shape
        .iter()
        .try_fold(1u64, |acc, &d| acc.checked_mul(d))
        .unwrap_or(u64::MAX);
    element_count.saturating_mul(data_type_bytes(data_type))
}
