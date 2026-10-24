//! What a dimension is (time, latitude, longitude or a vertical level),
//! whatever the provider named it (`valid_time`, `rlat`, `nav_lon`,
//! `dim_1`): its coordinate's CF attributes (`axis`, `standard_name`,
//! `units`, `positive`), else its OME-Zarr axis type, else the role the
//! selection gives it, and only then its name.

use crate::app::{DimConfig, SpatialRole};
use crate::data::coordinates::naming::{
    contains_ascii_case_insensitive as has, is_animated_time_name, is_spatial_x_name,
    is_spatial_y_name, is_spatial_z_name,
};
use crate::data::ome::AxisKind;
use crate::data::{DatasetMetadata, VariableInfo};

/// A dimension name heuristic of `coordinates::naming`.
type NameCheck = fn(&str) -> bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Axis {
    Time,
    Latitude,
    Longitude,
    Level,
}

impl Axis {
    /// The axis a word names ("time", "latitude", "lat", "level"), the
    /// words the phrase tables read axes as among them.
    pub fn of_word(word: &str) -> Option<Self> {
        let checks: [(NameCheck, Self); 4] = [
            (is_animated_time_name, Self::Time),
            (is_spatial_y_name, Self::Latitude),
            (is_spatial_x_name, Self::Longitude),
            (is_spatial_z_name, Self::Level),
        ];
        checks
            .into_iter()
            .find(|(is, _)| is(word))
            .map(|(_, axis)| axis)
    }
}

/// The axis CF attributes give a coordinate variable.
fn of_cf(coord: &VariableInfo) -> Option<Axis> {
    let attr = |key: &str| coord.attributes.get(key).map(String::as_str);
    let by_axis = match attr("axis").map(str::trim) {
        Some("T" | "t") => Some(Axis::Time),
        Some("Y" | "y") => Some(Axis::Latitude),
        Some("X" | "x") => Some(Axis::Longitude),
        Some("Z" | "z") => Some(Axis::Level),
        _ => None,
    };
    let name = attr("standard_name").unwrap_or("");
    let units = coord.units.as_deref().or(attr("units")).unwrap_or("");
    by_axis.or_else(|| {
        if name == "time" || has(units, " since ") {
            Some(Axis::Time)
        } else if has(name, "latitude")
            || name == "projection_y_coordinate"
            || has(units, "degrees_n")
            || has(units, "degree_n")
        {
            Some(Axis::Latitude)
        } else if has(name, "longitude")
            || name == "projection_x_coordinate"
            || has(units, "degrees_e")
            || has(units, "degree_e")
        {
            Some(Axis::Longitude)
        } else {
            let vertical = ["pressure", "depth", "height", "altitude", "level"];
            let pressure = ["hpa", "mbar", "millibar"];
            let is_level = attr("positive").is_some()
                || vertical.iter().any(|v| has(name, v))
                || pressure.iter().any(|p| has(units, p))
                || units.trim() == "Pa";
            is_level.then_some(Axis::Level)
        }
    })
}

/// What dimension `i` of `var` is, `config` being the role the selection
/// gives it.
pub(super) fn axis_of(
    meta: &DatasetMetadata,
    var: &VariableInfo,
    i: usize,
    config: Option<&DimConfig>,
) -> Option<Axis> {
    let name = var.dimension_names.get(i)?;
    let coord = meta.variables.iter().find(|v| {
        v.name.trim_matches('/').eq_ignore_ascii_case(name)
            || v.leaf_name().eq_ignore_ascii_case(name)
    });
    if let Some(axis) = coord.and_then(of_cf) {
        return Some(axis);
    }
    let ome = var.ome.as_ref().and_then(|o| o.image.axes.get(i));
    if ome.is_some_and(|a| a.kind == AxisKind::Time) {
        return Some(Axis::Time);
    }
    let by_role = config.and_then(|c| match c.spatial {
        SpatialRole::Y => Some(Axis::Latitude),
        SpatialRole::X => Some(Axis::Longitude),
        SpatialRole::Z => Some(Axis::Level),
        _ => None,
    });
    by_role.or_else(|| Axis::of_word(name))
}
