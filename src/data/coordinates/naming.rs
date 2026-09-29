//! Coordinate and dimension name heuristics and title formatting.

use std::borrow::Cow;

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
#[inline]
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
#[inline]
pub fn is_spatial_x_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("x")
        || contains_ascii_case_insensitive(clean, "lon")
        || contains_ascii_case_insensitive(clean, "col")
        || is_healpix_dim_name(clean)
}

/// Checks if a dimension name matches Spatial Y heuristics (latitude / Y / row) with zero allocation.
#[inline]
pub fn is_spatial_y_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("y")
        || contains_ascii_case_insensitive(clean, "lat")
        || contains_ascii_case_insensitive(clean, "row")
}

/// Checks if a dimension name matches Spatial Z heuristics (depth / level / height / alt / sigma / Z) with zero allocation.
#[inline]
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

/// Checks if a dimension name matches animated time heuristics (time / t / ti / step / date / epoch) with zero allocation.
#[inline]
pub fn is_animated_time_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("t")
        || clean.eq_ignore_ascii_case("ti")
        || clean.eq_ignore_ascii_case("date")
        || clean.eq_ignore_ascii_case("datetime")
        || clean.eq_ignore_ascii_case("epoch")
        || contains_ascii_case_insensitive(clean, "time")
        || contains_ascii_case_insensitive(clean, "step")
        || contains_ascii_case_insensitive(clean, "forecast")
}

/// Checks if a dimension name matches channel heuristics (c / chan / channel / band / wavelength) with zero allocation.
#[inline]
pub fn is_channel_dim_name(dim_name: &str) -> bool {
    let clean = dim_name.trim();
    clean.eq_ignore_ascii_case("c")
        || clean.eq_ignore_ascii_case("chan")
        || clean.eq_ignore_ascii_case("band")
        || clean.eq_ignore_ascii_case("bands")
        || contains_ascii_case_insensitive(clean, "channel")
        || contains_ascii_case_insensitive(clean, "wavelength")
}

/// Formats a dimension name into a human-friendly axis title with standard units with minimal allocations.
pub fn format_dimension_axis_title<'a>(dim_name: &'a str) -> Cow<'a, str> {
    let clean = dim_name.trim();
    if clean.is_empty() {
        return Cow::Borrowed("Index");
    }
    if clean.contains(['[', '(']) {
        return Cow::Borrowed(dim_name);
    }

    if is_channel_dim_name(clean) {
        Cow::Borrowed("Channel")
    } else if is_healpix_dim_name(clean) {
        Cow::Owned(format!("{dim_name} [Cell Index]"))
    } else if contains_ascii_case_insensitive(clean, "lon") {
        Cow::Owned(format!("{dim_name} [°E]"))
    } else if contains_ascii_case_insensitive(clean, "lat") {
        Cow::Owned(format!("{dim_name} [°N]"))
    } else if contains_ascii_case_insensitive(clean, "depth")
        || contains_ascii_case_insensitive(clean, "height")
        || contains_ascii_case_insensitive(clean, "alt")
    {
        Cow::Owned(format!("{dim_name} [m]"))
    } else if is_animated_time_name(clean) {
        Cow::Borrowed(dim_name)
    } else if clean.eq_ignore_ascii_case("x") {
        Cow::Borrowed("X [px]")
    } else if clean.eq_ignore_ascii_case("y") {
        Cow::Borrowed("Y [px]")
    } else if clean.eq_ignore_ascii_case("z") {
        Cow::Borrowed("Z [Slice]")
    } else {
        Cow::Borrowed(dim_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_dimension_heuristics() {
        assert!(is_animated_time_name("t"));
        assert!(is_animated_time_name("T"));
        assert!(is_animated_time_name("ti"));
        assert!(is_animated_time_name("Ti"));
        assert!(is_animated_time_name("TI"));
        assert!(is_animated_time_name("time"));
        assert!(is_animated_time_name("Time"));
        assert!(is_animated_time_name("TIME"));
        assert!(is_animated_time_name("time_counter"));
        assert!(is_animated_time_name("valid_time"));
        assert!(is_animated_time_name("forecast_time"));
        assert!(is_animated_time_name("leadtime"));
        assert!(is_animated_time_name("timestep"));
        assert!(is_animated_time_name("datetime"));
        assert!(is_animated_time_name("epoch"));
        assert!(is_animated_time_name("date"));

        assert!(!is_animated_time_name("lon"));
        assert!(!is_animated_time_name("latitude"));
        assert!(!is_animated_time_name("depth"));
        assert!(!is_animated_time_name("channel"));
    }

    #[test]
    fn test_format_dimension_axis_title_allocations() {
        assert_eq!(format_dimension_axis_title("longitude"), "longitude [°E]");
        assert_eq!(format_dimension_axis_title("latitude"), "latitude [°N]");
        assert_eq!(format_dimension_axis_title("depth"), "depth [m]");
        assert_eq!(format_dimension_axis_title("time"), "time");
        assert_eq!(format_dimension_axis_title("Ti"), "Ti");
        assert_eq!(format_dimension_axis_title("c"), "Channel");
        assert_eq!(format_dimension_axis_title("x"), "X [px]");
        assert_eq!(format_dimension_axis_title(""), "Index");
        assert_eq!(
            format_dimension_axis_title("temperature [K]"),
            "temperature [K]"
        );
    }
}
