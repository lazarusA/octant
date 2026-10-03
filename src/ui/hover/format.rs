//! Tooltip dimension coordinate formatting with physical units.

use crate::data::coordinates::naming::{contains_ascii_case_insensitive, is_animated_time_name};
use crate::data::{DatasetMetadata, VariableInfo};
use crate::ui::hover::field::HoverField;
use crate::utils::units::{
    format_axis_value, format_cardinal_degrees, format_scalar_coordinate, is_cf_time_unit,
};

/// Formats a dimension coordinate as a `label` / `value` row with physical units, cardinal
/// degrees, pressure, or datetime.
pub fn format_dimension_coord(
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    store_target: Option<&str>,
    dim_name: &str,
    idx: usize,
    total_len: usize,
    geo_fallback: Option<f32>,
) -> HoverField {
    let ctx = DimContext::resolve(meta, var, store_target, dim_name, idx, total_len);
    let value = meta
        .and_then(|m| from_metadata(m, var, &ctx))
        .or_else(|| geo_fallback.map(|geo| from_geo(dim_name, geo)))
        .unwrap_or_else(|| ctx.fallback());
    HoverField::new(dim_name, value)
}

/// One dimension position plus the attributes every formatting branch needs.
struct DimContext<'a> {
    name: &'a str,
    units: Option<&'a str>,
    time_start: Option<&'a str>,
    temporal_res: Option<&'a str>,
    is_time: bool,
    store_target: Option<&'a str>,
    idx: usize,
    total_len: usize,
}

impl<'a> DimContext<'a> {
    fn resolve(
        meta: Option<&'a DatasetMetadata>,
        var: Option<&'a VariableInfo>,
        store_target: Option<&'a str>,
        name: &'a str,
        idx: usize,
        total_len: usize,
    ) -> Self {
        // The dimension's own coordinate variable, if the dataset has one.
        let coord_var = meta.and_then(|m| {
            m.variables
                .iter()
                .find(|v| v.name.eq_ignore_ascii_case(name))
        });
        // Units only come from a variable that *is* this dimension.
        let units = coord_var
            .or(var.filter(|v| v.name.eq_ignore_ascii_case(name)))
            .and_then(|v| attr(v, &v.units, "units"));
        let inherited = |pick: fn(&'a VariableInfo) -> Option<&'a str>| {
            coord_var.and_then(pick).or_else(|| var.and_then(pick))
        };
        Self {
            name,
            units,
            time_start: inherited(|v| attr(v, &v.time_coverage_start, "time_coverage_start")),
            temporal_res: inherited(|v| attr(v, &v.temporal_resolution, "temporal_resolution")),
            is_time: is_animated_time_name(name) || units.is_some_and(is_cf_time_unit),
            store_target,
            idx,
            total_len,
        }
    }

    /// Position of `idx` along the dimension, from 0 (first) to 1 (last).
    fn fraction(&self) -> f64 {
        if self.total_len > 1 {
            self.idx as f64 / (self.total_len - 1) as f64
        } else {
            0.0
        }
    }

    fn scalar(&self, val: f64) -> String {
        format_scalar_coordinate(self.name, val, self.units, self.time_start, self.is_time)
    }

    fn time(&self, start: Option<&str>) -> String {
        format_axis_value(
            self.idx,
            self.total_len,
            Some(self.name),
            self.units,
            start,
            self.temporal_res,
            self.store_target,
        )
    }

    /// Without coordinates: a date from the time attributes, or the 1-based position.
    fn fallback(&self) -> String {
        if self.is_time {
            self.time(self.time_start)
        } else if self.total_len > 1 {
            format!("{} / {}", self.idx + 1, self.total_len)
        } else {
            self.idx.to_string()
        }
    }
}

/// A typed metadata field, falling back to the raw attribute of the same name.
fn attr<'a>(v: &'a VariableInfo, typed: &'a Option<String>, key: &str) -> Option<&'a str> {
    typed
        .as_deref()
        .or_else(|| v.attributes.get(key).map(String::as_str))
}

fn from_metadata(
    meta: &DatasetMetadata,
    var: Option<&VariableInfo>,
    ctx: &DimContext,
) -> Option<String> {
    let var_name = var.map(|v| v.name.as_str());
    if let Some(value) = meta
        .get_dim_coords(var_name, ctx.name)
        .and_then(|coords| from_coords(coords, ctx))
    {
        return Some(value);
    }
    let (min_b, max_b) = meta.get_coord_bounds_for_var(var_name, ctx.name)?;
    Some(ctx.scalar(min_b + ctx.fraction() * (max_b - min_b)))
}

/// Reads the coordinate at `ctx.idx`. A list whose length does not match the dimension is
/// only trusted for its endpoints (dates or a numeric range), never for a label by index.
fn from_coords(coords: &[String], ctx: &DimContext) -> Option<String> {
    if coords.len() == ctx.total_len
        && let Some(c) = coords.get(ctx.idx).map(|c| c.trim())
        && !c.is_empty()
    {
        // Dates and labels never parse as numbers, so they are shown as stored.
        return Some(
            c.parse::<f64>()
                .map_or_else(|_| c.to_string(), |v| ctx.scalar(v)),
        );
    }

    if let [first, .., last] = coords {
        let first_is_date = first.contains('-') || first.contains(':') || first.contains('T');
        if ctx.is_time && first_is_date {
            return Some(ctx.time(Some(first)));
        }
        if let (Ok(f_v), Ok(l_v)) = (first.parse::<f64>(), last.parse::<f64>()) {
            return Some(ctx.scalar(f_v + ctx.fraction() * (l_v - f_v)));
        }
    }

    ctx.is_time.then(|| ctx.time(ctx.time_start))
}

fn from_geo(dim_name: &str, geo: f32) -> String {
    if contains_ascii_case_insensitive(dim_name, "lon") {
        format_cardinal_degrees(geo as f64, true)
    } else if contains_ascii_case_insensitive(dim_name, "lat") {
        format_cardinal_degrees(geo as f64, false)
    } else {
        format!("{:.2}°", geo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(name: &str, idx: usize, total_len: usize) -> DimContext<'_> {
        DimContext::resolve(None, None, None, name, idx, total_len)
    }

    fn coords(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_format_dimension_coord_fallback() {
        let res_dim = format_dimension_coord(None, None, None, "dim0", 5, 10, None);
        assert_eq!(res_dim, HoverField::new("dim0", "6 / 10"));

        let res_step = format_dimension_coord(None, None, None, "step", 5, 10, None);
        assert_eq!(res_step, HoverField::new("step", "Step 6 / 10"));
    }

    #[test]
    fn test_format_dimension_coord_geo_fallback() {
        let res_lon = format_dimension_coord(None, None, None, "lon", 0, 1, Some(-45.5));
        assert_eq!(res_lon, HoverField::new("lon", "45.50°W"));

        let res_lat = format_dimension_coord(None, None, None, "lat", 0, 1, Some(12.25));
        assert_eq!(res_lat, HoverField::new("lat", "12.25°N"));
    }

    #[test]
    fn negative_coordinates_are_formatted_like_positive_ones() {
        let lons = coords(&["-45.5", "45.5"]);
        assert_eq!(
            from_coords(&lons, &ctx("lon", 0, 2)).as_deref(),
            Some("45.50°W")
        );
        assert_eq!(
            from_coords(&lons, &ctx("lon", 1, 2)).as_deref(),
            Some("45.50°E")
        );
    }

    #[test]
    fn labels_and_dates_are_shown_as_stored() {
        let regions = coords(&["Europe", "Africa"]);
        assert_eq!(
            from_coords(&regions, &ctx("region", 1, 2)).as_deref(),
            Some("Africa")
        );
        let dates = coords(&[" 2024-01-15 ", "2024-01-16"]);
        assert_eq!(
            from_coords(&dates, &ctx("day", 0, 2)).as_deref(),
            Some("2024-01-15")
        );
    }

    #[test]
    fn mismatched_label_list_is_not_read_by_index() {
        // Three labels for a ten-step dimension: none of them belongs to index 7.
        let members = coords(&["r1", "r2", "r3"]);
        assert_eq!(from_coords(&members, &ctx("member", 7, 10)), None);
        // A numeric range with the wrong length still interpolates its endpoints.
        let levels = coords(&["0", "90"]);
        let mid = from_coords(&levels, &ctx("depth", 5, 10));
        assert_eq!(mid.as_deref(), Some("50.00 m"));
    }
}
