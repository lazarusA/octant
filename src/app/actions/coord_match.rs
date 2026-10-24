//! A value said for a dimension to its index: a date against the dates its
//! coordinates stand for (read as the hover reads them, `DimContext`), a
//! number against its numbers, a label against its labels; without
//! coordinates, the 1-based position the hover shows.

use super::text::Name;
use crate::data::{CoordValue, CoordValues, DatasetMetadata, VariableInfo};
use crate::ui::hover::format::{DimContext, format_dimension_coord};
use crate::utils::units::days_from_civil;

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
/// coordinates, if any). A date names the earliest coordinate within the
/// year, month, day or hour it covers, or with `last` the latest (the end
/// of a range); the nearest when none falls within it.
pub(super) fn index_of(
    dim: &Dim,
    coords: Option<&CoordValues>,
    value: &str,
    last: bool,
) -> Option<usize> {
    let Some(coords) = coords else {
        let position: usize = value.trim().parse().ok()?;
        return (1..=dim.len).contains(&position).then(|| position - 1);
    };
    let context = dim.context(0);
    if let Some((start, end)) = iso_span(value) {
        let hours = |i| date_hours(&context, dim.len, coords, i);
        let inside = (0..dim.len).filter_map(|i| {
            hours(i)
                .filter(|h| (start..end).contains(h))
                .map(|h| (i, h))
        });
        let found = if last {
            inside.max_by(|a, b| a.1.total_cmp(&b.1))
        } else {
            inside.min_by(|a, b| a.1.total_cmp(&b.1))
        };
        if let Some(idx) = found
            .map(|(i, _)| i)
            .or_else(|| nearest(dim.len, hours, start))
        {
            return Some(idx);
        }
    }
    if let Ok(number) = value.trim().parse::<f64>()
        && let Some(idx) = nearest(dim.len, |i| coords.number_for(i, dim.len), number)
    {
        return Some(idx);
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

/// Hours since 1970 of coordinate `i` when it is a date; `context` (the
/// dimension's units and start date, the same for every index) reads it.
fn date_hours(context: &DimContext, len: usize, coords: &CoordValues, i: usize) -> Option<f64> {
    if let Some(CoordValue::Label(label)) = coords.get(i) {
        return iso_span(label).map(|(start, _)| start);
    }
    let date = context.date(coords.number_for(i, len)?)?;
    let days = days_from_civil(date.year as i64, date.month as u32, date.day as u32);
    Some((days * 24 + date.hour.unwrap_or(0) as i64) as f64)
}

/// The hours since 1970 an ISO date covers, `[start, end)`: a year
/// (`2020`), a month (`2020-03`), a day (`2020-03-15`) or an hour
/// (`2020-03-15T12:00`).
fn iso_span(text: &str) -> Option<(f64, f64)> {
    let text = text.trim();
    let (date, time) = text.split_once(['T', ' ']).unwrap_or((text, ""));
    let mut parts = date.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: Option<u32> = parts.next().map(str::parse).transpose().ok()?;
    let d: Option<u32> = parts.next().map(str::parse).transpose().ok()?;
    let hour: Option<i64> = match time {
        "" => None,
        t => Some(t.split(':').next()?.parse().ok()?),
    };
    let valid = (1000..=9999).contains(&y)
        && m.is_none_or(|m| (1..=12).contains(&m))
        && d.is_none_or(|d| (1..=31).contains(&d))
        && hour.is_none_or(|h| (0..24).contains(&h) && d.is_some());
    if !valid || parts.next().is_some() {
        return None;
    }
    let start = days_from_civil(y, m.unwrap_or(1), d.unwrap_or(1)) * 24 + hour.unwrap_or(0);
    let end = match (m, d, hour) {
        (_, _, Some(_)) => start + 1,
        (_, Some(_), None) => start + 24,
        (Some(12), None, None) => days_from_civil(y + 1, 1, 1) * 24,
        (Some(m), None, None) => days_from_civil(y, m + 1, 1) * 24,
        (None, ..) => days_from_civil(y + 1, 1, 1) * 24,
    };
    Some((start as f64, end as f64))
}
