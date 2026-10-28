//! Tooltip dimension coordinate formatting with physical units.

use crate::data::coordinates::naming::{contains_ascii_case_insensitive, is_animated_time_name};
use crate::data::{CoordValue, CoordValues, DatasetMetadata, VariableInfo};
use crate::i18n::date::Date;
use crate::i18n::number::Num;
use crate::ui::hover::field::HoverField;
use crate::utils::units::dates::coordinate_date;
use crate::utils::units::{
    COORD_DECIMALS, format_axis_value, format_cardinal_degrees, format_scalar_coordinate,
    is_cf_time_unit, step_decimals,
};

/// Most decimals a hovered coordinate shows, however fine its grid.
const MAX_COORD_DECIMALS: usize = 6;

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
pub(crate) struct DimContext<'a> {
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
    pub(crate) fn resolve(
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
        // Or the axis unit of an OME-Zarr image.
        let units = coord_var
            .or(var.filter(|v| v.name.eq_ignore_ascii_case(name)))
            .and_then(|v| attr(v, &v.units, "units"))
            .or_else(|| var.and_then(|v| v.dimension_unit(name)));
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

    /// Whether this dimension is time, by its name or CF units.
    pub(crate) fn is_time(&self) -> bool {
        self.is_time
    }

    /// The date a coordinate value of this dimension stands for, if any.
    pub(crate) fn date(&self, val: f64) -> Option<Date> {
        coordinate_date(val, self.units, self.time_start, self.is_time)
    }

    /// `val` with `decimals` decimals where it is a plain number.
    fn scalar(&self, val: f64, decimals: usize) -> String {
        let (units, start) = (self.units, self.time_start);
        format_scalar_coordinate(self.name, val, units, start, self.is_time, decimals)
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
    let decimals = (spacing(coords, ctx.idx, ctx.total_len)).map_or(COORD_DECIMALS, |step| {
        step_decimals(step).clamp(COORD_DECIMALS, MAX_COORD_DECIMALS)
    });
    if coords.matches(ctx.total_len) {
        match coords.get(ctx.idx)? {
            CoordValue::Number(v) => return Some(ctx.scalar(v, decimals)),
            CoordValue::Label(l) if !l.trim().is_empty() => return Some(l.trim().to_string()),
            CoordValue::Label(_) => {}
        }
    }
    if let Some(v) = coords.number_for(ctx.idx, ctx.total_len) {
        return Some(ctx.scalar(v, decimals));
    }
    let first = coords.label(0).map(str::trim);
    let first_is_date =
        first.is_some_and(|f| f.contains('-') || f.contains(':') || f.contains('T'));
    if ctx.is_time && first_is_date {
        return Some(ctx.time(first));
    }
    ctx.is_time.then(|| ctx.time(ctx.time_start))
}

/// The distance from index `idx` to its next neighbor (its previous one at the end) of a
/// `len` dimension, so neighboring values can be told apart.
fn spacing(coords: &CoordValues, idx: usize, len: usize) -> Option<f64> {
    let other = if idx + 1 < len {
        idx + 1
    } else {
        idx.checked_sub(1)?
    };
    Some((coords.number_for(other, len)? - coords.number_for(idx, len)?).abs())
}

fn from_geo(dim_name: &str, geo: f32) -> String {
    if contains_ascii_case_insensitive(dim_name, "lon") {
        format_cardinal_degrees(geo as f64, true, COORD_DECIMALS)
    } else if contains_ascii_case_insensitive(dim_name, "lat") {
        format_cardinal_degrees(geo as f64, false, COORD_DECIMALS)
    } else {
        format!("{}°", Num(format_args!("{geo:.2}")))
    }
}

#[cfg(test)]
#[path = "format_tests.rs"]
mod tests;
