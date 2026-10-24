//! A value said for a dimension to its index: a date against the dates its
//! coordinates stand for (read as the hover reads them, `DimContext`), a
//! number against its numbers, a label against its labels; without
//! coordinates, the 1-based position the hover shows.

use super::text::Name;
use crate::data::{CoordValue, CoordValues, DatasetMetadata, VariableInfo};
use crate::ui::hover::format::{DimContext, format_dimension_coord};
use crate::utils::units::{days_from_civil, parse_iso_date};

/// One dimension of a variable, as the hover resolves it.
pub(super) struct Dim<'a> {
    pub meta: &'a DatasetMetadata,
    pub var: &'a VariableInfo,
    pub store: &'a str,
    pub name: &'a str,
    pub len: usize,
}

impl Dim<'_> {
    fn context(&self, idx: usize) -> DimContext<'_> {
        let (meta, var) = (Some(self.meta), Some(self.var));
        DimContext::resolve(meta, var, Some(self.store), self.name, idx, self.len)
    }

    /// The coordinate at `idx` as the hover writes it.
    pub fn text(&self, idx: usize) -> String {
        let (meta, var) = (Some(self.meta), Some(self.var));
        format_dimension_coord(meta, var, Some(self.store), self.name, idx, self.len, None).value
    }
}

/// The index `value` names along `dim`, read through `coords` (its own
/// coordinates, if any).
pub(super) fn index_of(dim: &Dim, coords: Option<&CoordValues>, value: &str) -> Option<usize> {
    let Some(coords) = coords else {
        let position: usize = value.trim().parse().ok()?;
        return (1..=dim.len).contains(&position).then(|| position - 1);
    };
    if let Some(target) = iso_hours(value)
        && let Some(idx) = nearest(dim.len, |i| date_hours(dim, coords, i), target as f64)
    {
        return Some(idx);
    }
    if let Ok(number) = value.trim().parse::<f64>() {
        return nearest(dim.len, |i| coords.number_for(i, dim.len), number);
    }
    let heard = Name::new(value);
    let labels = coords.labels()?;
    labels
        .iter()
        .position(|label| Name::new(label).compact == heard.compact)
}

/// The index whose `key` is closest to `target`.
fn nearest(len: usize, key: impl Fn(usize) -> Option<f64>, target: f64) -> Option<usize> {
    (0..len)
        .filter_map(|i| key(i).map(|k| (i, (k - target).abs())))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Hours since 1970 of coordinate `i` when it is a date.
fn date_hours(dim: &Dim, coords: &CoordValues, i: usize) -> Option<f64> {
    if let Some(CoordValue::Label(label)) = coords.get(i) {
        return iso_hours(label).map(|h| h as f64);
    }
    let date = dim.context(i).date(coords.number_for(i, dim.len)?)?;
    let days = days_from_civil(date.year as i64, date.month as u32, date.day as u32);
    Some((days * 24 + date.hour.unwrap_or(0) as i64) as f64)
}

/// Hours since 1970 of an ISO date: `2020`, `2020-03` or `2020-03-15`
/// (a time of day ignored), at its start.
fn iso_hours(text: &str) -> Option<i64> {
    let text = text.trim();
    let (y, m, d) = parse_iso_date(text).or_else(|| {
        let mut parts = text.split('-');
        let y = parts.next()?.parse().ok()?;
        let m = parts.next().map_or(Some(1), |m| m.parse().ok())?;
        parts.next().is_none().then_some((y, m, 1))
    })?;
    let valid = (1..=12).contains(&m) && (1..=31).contains(&d) && (1000..=9999).contains(&y);
    valid.then(|| days_from_civil(y as i64, m as u32, d as u32) * 24)
}
