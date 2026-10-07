//! Tooltip dimension coordinate formatting with physical units.

use crate::data::coordinates::naming::{contains_ascii_case_insensitive, is_animated_time_name};
use crate::data::{CoordValue, CoordValues, DatasetMetadata, VariableInfo};
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
    let coords = meta.get_dim_coords(var.map(|v| v.name.as_str()), ctx.name)?;
    from_coords(coords, ctx)
}

/// Reads the coordinate at `ctx.idx`. Values that do not match the dimension one to one
/// are only trusted for their endpoints (a start date or a numeric range), never for a
/// label by index.
fn from_coords(coords: &CoordValues, ctx: &DimContext) -> Option<String> {
    if coords.matches(ctx.total_len) {
        match coords.get(ctx.idx)? {
            CoordValue::Number(v) => return Some(ctx.scalar(v)),
            CoordValue::Label(l) if !l.trim().is_empty() => return Some(l.trim().to_string()),
            CoordValue::Label(_) => {}
        }
    }
    if let Some(v) = coords.number_for(ctx.idx, ctx.total_len) {
        return Some(ctx.scalar(v));
    }
    let first = coords.label(0).map(str::trim);
    let first_is_date =
        first.is_some_and(|f| f.contains('-') || f.contains(':') || f.contains('T'));
    if ctx.is_time && first_is_date {
        return Some(ctx.time(first));
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

    fn numbers(values: &[f64]) -> CoordValues {
        CoordValues::from_values(values.to_vec(), false).expect("numbers")
    }

    fn labels(values: &[&str]) -> CoordValues {
        let labels = values.iter().map(|s| s.to_string()).collect();
        CoordValues::from_labels(labels).expect("labels")
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
        let lons = numbers(&[-45.5, 45.5]);
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
        let regions = labels(&["Europe", "Africa"]);
        assert_eq!(
            from_coords(&regions, &ctx("region", 1, 2)).as_deref(),
            Some("Africa")
        );
        let dates = labels(&[" 2024-01-15 ", "2024-01-16"]);
        assert_eq!(
            from_coords(&dates, &ctx("day", 0, 2)).as_deref(),
            Some("2024-01-15")
        );
    }

    #[test]
    fn mismatched_label_list_is_not_read_by_index() {
        // Three labels for a ten-step dimension: none of them belongs to index 7.
        let members = labels(&["r1", "r2", "r3"]);
        assert_eq!(from_coords(&members, &ctx("member", 7, 10)), None);
        // A numeric range with the wrong length still interpolates its endpoints.
        let levels = numbers(&[0.0, 90.0]);
        let mid = from_coords(&levels, &ctx("depth", 5, 10));
        assert_eq!(mid.as_deref(), Some("50.00 m"));
    }

    #[test]
    fn uneven_levels_show_their_stored_value() {
        let levels = numbers(&[1000.0, 925.0, 850.0, 700.0, 500.0, 300.0]);
        assert!(matches!(levels, CoordValues::Values(_)));
        let at = |i| from_coords(&levels, &ctx("depth", i, 6));
        assert_eq!(at(2).as_deref(), Some("850.00 m"));
        assert_eq!(at(5).as_deref(), Some("300.00 m"));
    }
}
