//! The leading dimensions of GRIB variables: reference time, valid time (or, where the
//! reference time varies, the step from it, as cfgrib does: an hourly reanalysis's valid
//! times follow its reference times, and the two as dimensions would make an n x n grid
//! holding only its diagonal), ensemble member
//! and level (a layer's bottom, or its top where the bottoms are all alike: 0-1000 m and
//! 0-6000 m wind shear), each a dimension only where a variable's messages take more than
//! one value of it (one value is an attribute). Variables sharing a dimension's values share it;
//! other values get their own name (`time`, `time_1`, ...).

use std::collections::HashMap;
use std::sync::Arc;

use gribberish::message_metadata::MessageMetadata;
use gribberish::templates::product::tables::FixedSurfaceType;

use super::entry::{Entry, attrs};

const TIME_UNITS: &str = "seconds since 1970-01-01 00:00:00";

/// One leading dimension of a variable: its values, sorted, and how a message places on it.
pub struct Axis {
    pub base: String,
    pub values: Vec<f64>,
    pub attrs: HashMap<String, String>,
    pub role: Role,
}

/// What an axis orders messages by.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Reference,
    Valid,
    /// Hours from the reference time to the valid time.
    Step,
    Member,
    Level,
    /// The top of layers sharing their bottom.
    LevelTop,
}

impl Role {
    /// The value of the message `m` along this axis: seconds since 1970 for times (valid
    /// at the end of an accumulation or average), NaN where a message has none.
    fn of(self, m: &MessageMetadata) -> f64 {
        match self {
            Role::Reference => m.reference_date.timestamp() as f64,
            Role::Valid => m.forecast_end_date.unwrap_or(m.forecast_date).timestamp() as f64,
            Role::Step => (Role::Valid.of(m) - Role::Reference.of(m)) / 3600.0,
            Role::Member => m.perturbation_number.map_or(f64::NAN, f64::from),
            Role::Level => m.first_fixed_surface_value.unwrap_or(f64::NAN),
            Role::LevelTop => m.second_fixed_surface_value.unwrap_or(f64::NAN),
        }
    }
}

impl Axis {
    /// The index along this axis of the message `meta` describes.
    pub fn index(&self, meta: &MessageMetadata) -> Option<usize> {
        let v = self.role.of(meta);
        self.values.iter().position(|x| x.total_cmp(&v).is_eq())
    }
}

/// The units of levels on `surface`, where they have any.
fn level_units(surface: &FixedSurfaceType) -> Option<&'static str> {
    use FixedSurfaceType as S;
    match surface {
        S::IsobaricSurface | S::LevelAtSpecifiedPressureDifferenceFromGroundToLevel => Some("Pa"),
        S::SpecifiedHeightLevelAboveGround
        | S::SpecificAltitudeAboveMeanSeaLevel
        | S::DepthBelowLandSurface
        | S::DepthBelowSeaLevel
        | S::DepthBelowWaterSurface => Some("m"),
        _ => None,
    }
}

/// The values `metas` take along `role`, sorted, each once.
fn values(metas: &[&MessageMetadata], role: Role) -> Vec<f64> {
    let mut values: Vec<f64> = metas.iter().map(|m| role.of(m)).collect();
    values.sort_by(f64::total_cmp);
    values.dedup_by(|a, b| a.total_cmp(b).is_eq());
    values
}

fn axis(
    metas: &[&MessageMetadata],
    base: &str,
    attrs: HashMap<String, String>,
    role: Role,
) -> Axis {
    let values = values(metas, role);
    let base = base.to_owned();
    Axis {
        base,
        values,
        attrs,
        role,
    }
}

/// The four candidate axes of the messages `metas` (one variable's), with their values.
pub fn candidates(metas: &[&MessageMetadata]) -> [Axis; 4] {
    let time = |name| attrs(&[("units", TIME_UNITS), ("standard_name", name)]);
    let reference = axis(
        metas,
        "reference_time",
        time("forecast_reference_time"),
        Role::Reference,
    );
    let valid = match reference.values.len() > 1 {
        true => {
            let step = attrs(&[("units", "hours"), ("long_name", "forecast step")]);
            axis(metas, "step", step, Role::Step)
        }
        false => axis(metas, "time", time("time"), Role::Valid),
    };
    let member = attrs(&[("long_name", "ensemble member")]);
    let member = axis(metas, "member", member, Role::Member);
    [reference, valid, member, level(metas)]
}

/// The level axis of `metas`: by its layers' bottoms, else, when the bottoms are all alike
/// and the tops are not, by their tops (gribberish's own grouping does the same,
/// `python/src/dataset.rs`); named and with units by its surface type.
fn level(metas: &[&MessageMetadata]) -> Axis {
    let surface = &metas[0].first_fixed_surface_type;
    let description = surface.to_string();
    let mut level_attrs = attrs(&[("long_name", &description)]);
    if let Some(units) = level_units(surface) {
        level_attrs.insert("units".into(), units.into());
    }
    let base = match surface.coordinate_name() {
        "" => "level",
        name => name,
    };
    let bottoms = axis(metas, base, level_attrs, Role::Level);
    let tops = values(metas, Role::LevelTop);
    match bottoms.values.len() < 2 && tops.len() > 1 {
        true => Axis {
            values: tops,
            role: Role::LevelTop,
            ..bottoms
        },
        false => bottoms,
    }
}

/// What a variable says of an axis it has one value of, as an attribute; `None` when there
/// is nothing to say (no level, not an ensemble member).
pub fn scalar(axis: &Axis, meta: &MessageMetadata) -> Option<(String, String)> {
    let value = *axis.values.first()?;
    let text = match axis.role {
        Role::Reference => meta.reference_date.to_rfc3339(),
        Role::Valid => (meta.forecast_end_date)
            .unwrap_or(meta.forecast_date)
            .to_rfc3339(),
        _ if value.is_nan() => return None,
        _ => match axis.attrs.get("units") {
            Some(units) => format!("{value} {units}"),
            None => value.to_string(),
        },
    };
    Some((axis.base.clone(), text))
}

/// The names given to axes, each set of values once.
#[derive(Default)]
pub struct Axes {
    named: Vec<(Axis, String)>,
    used: HashMap<String, usize>,
}

impl Axes {
    /// The dimension name of `axis`: the one an axis with the same values already has, else
    /// its base, numbered when taken.
    pub fn name(&mut self, axis: Axis) -> String {
        let same = |a: &Axis| {
            a.base == axis.base
                && a.values.len() == axis.values.len()
                && (a.values.iter().zip(&axis.values)).all(|(x, y)| x.total_cmp(y).is_eq())
        };
        if let Some((_, name)) = self.named.iter().find(|(a, _)| same(a)) {
            return name.clone();
        }
        let count = self.used.entry(axis.base.clone()).or_insert(0);
        let name = match *count {
            0 => axis.base.clone(),
            n => format!("{}_{n}", axis.base),
        };
        *count += 1;
        self.named.push((axis, name.clone()));
        name
    }

    /// The coordinate variables of the named axes.
    pub fn entries(self) -> impl Iterator<Item = Entry> {
        (self.named.into_iter())
            .map(|(axis, name)| Entry::coordinate(&name, Arc::from(axis.values), axis.attrs))
    }
}
