//! Coordinate and dimension name heuristics and title formatting.

/// Fast zero-allocation case-insensitive ASCII substring search.
#[inline]
pub fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))
}

/// Checks if a dimension name matches HEALPix discrete cell heuristics with zero allocation.
pub fn is_healpix_dim_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("cell")
        || clean.eq_ignore_ascii_case("cells")
        || clean.eq_ignore_ascii_case("pix")
        || clean.eq_ignore_ascii_case("pixels")
        || clean.eq_ignore_ascii_case("ncells")
        || contains_ascii_case_insensitive(clean, "healpix")
}

/// Checks if a dimension name matches Spatial X heuristics (longitude / X / column / HEALPix cell) with zero allocation.
pub fn is_spatial_x_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("x")
        || contains_ascii_case_insensitive(clean, "lon")
        || contains_ascii_case_insensitive(clean, "col")
        || is_healpix_dim_name(clean)
}

/// Checks if a dimension name matches Spatial Y heuristics (latitude / Y / row) with zero allocation.
pub fn is_spatial_y_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("y")
        || contains_ascii_case_insensitive(clean, "lat")
        || contains_ascii_case_insensitive(clean, "row")
}

/// Checks if a dimension name matches Spatial Z heuristics (depth / level / height / alt / sigma / Z) with zero allocation.
pub fn is_spatial_z_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("z")
        || contains_ascii_case_insensitive(clean, "depth")
        || contains_ascii_case_insensitive(clean, "lev")
        || contains_ascii_case_insensitive(clean, "layer")
        || contains_ascii_case_insensitive(clean, "height")
        || contains_ascii_case_insensitive(clean, "alt")
        || contains_ascii_case_insensitive(clean, "sigma")
}

/// Checks if a dimension name matches animated time heuristics (time / t / step) with zero allocation.
pub fn is_animated_time_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("t")
        || contains_ascii_case_insensitive(clean, "time")
        || contains_ascii_case_insensitive(clean, "step")
}

/// Checks if a dimension name matches channel heuristics (c / chan / channel / band / wavelength) with zero allocation.
pub fn is_channel_dim_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("c")
        || clean.eq_ignore_ascii_case("chan")
        || clean.eq_ignore_ascii_case("channel")
        || clean.eq_ignore_ascii_case("channels")
        || clean.eq_ignore_ascii_case("band")
        || clean.eq_ignore_ascii_case("bands")
        || contains_ascii_case_insensitive(clean, "channel")
        || contains_ascii_case_insensitive(clean, "wavelength")
}

/// Formats a dimension name into a human-friendly axis title with standard units.
pub fn format_dimension_axis_title(dim_name: &str) -> String {
    let clean = dim_name.trim();
    if clean.is_empty() {
        return "Index".to_string();
    }
    if clean.contains('[') || clean.contains('(') {
        return dim_name.to_string();
    }

    if is_channel_dim_name(clean) {
        "Channel".to_string()
    } else if is_healpix_dim_name(clean) {
        format!("{dim_name} [Cell Index]")
    } else if contains_ascii_case_insensitive(clean, "lon") {
        format!("{dim_name} [°E]")
    } else if contains_ascii_case_insensitive(clean, "lat") {
        format!("{dim_name} [°N]")
    } else if contains_ascii_case_insensitive(clean, "depth")
        || contains_ascii_case_insensitive(clean, "height")
        || contains_ascii_case_insensitive(clean, "alt")
    {
        format!("{dim_name} [m]")
    } else if contains_ascii_case_insensitive(clean, "time") {
        dim_name.to_string()
    } else if clean.eq_ignore_ascii_case("x") {
        "X [px]".to_string()
    } else if clean.eq_ignore_ascii_case("y") {
        "Y [px]".to_string()
    } else if clean.eq_ignore_ascii_case("z") {
        "Z [Slice]".to_string()
    } else {
        dim_name.to_string()
    }
}
