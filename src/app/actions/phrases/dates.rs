//! Dates said in a command, in any language's form (`2020-01-15`, "15
//! January 2020", "15 de enero de 2020", `2020年1月15日`), written as ISO
//! (`2020`, `2020-01`, `2020-01-15`) with where they were said: the date
//! parts `values.rs` finds, grouped into dates.

use std::ops::Range;

use super::values::{Part, YEAR_MARKERS, parts};
use crate::app::actions::text::{Segment, joins, overlaps};

/// The dates said outside `taken`, as ISO text with their spans. A year
/// takes the month and day said before it ("15 January 2020"), or after it
/// in the year-first order of Chinese, Japanese and Korean; a date said
/// without one takes the year of the date beside it ("15 January to 20
/// February 2020", "2020年1月到3月").
pub(super) fn dates(
    segment: &Segment,
    months: &[&[&str]; 12],
    taken: &[Range<usize>],
) -> Vec<(String, Range<usize>)> {
    let mut out = iso_dates(segment, taken);
    let mut taken = taken.to_vec();
    taken.extend(out.iter().map(|(_, span)| span.clone()));
    let parts = parts(segment, months, &taken);
    let year_first = parts.iter().any(|(r, part)| {
        matches!(part, Part::Year(_)) && segment.text[r.end..].starts_with(YEAR_MARKERS)
    });
    let mut groups = group_parts(parts, year_first);
    lend_parts(&mut groups, year_first);
    out.extend(groups.iter().filter_map(|g| iso(g)));
    out.sort_by_key(|(_, span)| span.start);
    out
}

type Group = Vec<(Range<usize>, Part)>;

/// `parts` in dates: a year ends one (begins one, year first), as does a
/// month or day said again.
fn group_parts(parts: Group, year_first: bool) -> Vec<Group> {
    let same = |a: &Part, b: &Part| std::mem::discriminant(a) == std::mem::discriminant(b);
    let mut groups: Vec<Group> = vec![Vec::new()];
    for part in parts {
        let is_year = matches!(part.1, Part::Year(_));
        let group = groups.last_mut().filter(|g| !g.is_empty());
        let again = group.is_some_and(|g| g.iter().any(|(_, p)| same(p, &part.1)));
        if again || (year_first && is_year) {
            groups.push(Vec::new());
        }
        if let Some(g) = groups.last_mut() {
            g.push(part);
        }
        if !year_first && is_year {
            groups.push(Vec::new());
        }
    }
    groups.retain(|g| !g.is_empty());
    groups
}

/// Each group without a year takes the next group's (the previous one's,
/// year first), and a day said without a month that group's month too
/// ("2020年1月15日到20日").
fn lend_parts(groups: &mut [Group], year_first: bool) {
    let has = |g: &Group, f: fn(&Part) -> bool| g.iter().any(|(_, p)| f(p));
    let is_year = |p: &Part| matches!(p, Part::Year(_));
    let is_month = |p: &Part| matches!(p, Part::Month(_));
    for i in 0..groups.len() {
        if has(&groups[i], is_year) {
            continue;
        }
        let lender = if year_first {
            groups[..i].iter().rev().find(|g| has(g, is_year))
        } else {
            groups[i + 1..].iter().find(|g| has(g, is_year))
        };
        let Some(lender) = lender else {
            continue;
        };
        let lacks_month = !has(&groups[i], is_month);
        let shift = new_year_shift(month_of(&groups[i]), month_of(lender), year_first);
        let lent: Vec<Part> = lender
            .iter()
            .map(|(_, p)| match *p {
                Part::Year(y) => Part::Year(y.saturating_add_signed(shift)),
                p => p,
            })
            .filter(|p| is_year(p) || (lacks_month && is_month(p)))
            .collect();
        // The span stays the group's own: what is lent was said for both.
        let at = groups[i].last().map_or(0..0, |(r, _)| r.end..r.end);
        groups[i].extend(lent.into_iter().map(|p| (at.clone(), p)));
    }
}

fn month_of(group: &Group) -> Option<u32> {
    group.iter().find_map(|(_, p)| match p {
        Part::Month(m) => Some(*m),
        _ => None,
    })
}

/// The years between a date said without one, of month `own`, and the
/// date it borrows its year from, of month `lender`: a range crossing New
/// Year ("December to February 2020" starts in 2019; year first, "2020年12月
/// 30日到1月2日" ends in 2021).
fn new_year_shift(own: Option<u32>, lender: Option<u32>, year_first: bool) -> i32 {
    match (own, lender) {
        (Some(own), Some(lender)) if !year_first && own > lender => -1,
        (Some(own), Some(lender)) if year_first && own < lender => 1,
        _ => 0,
    }
}

/// Dates written as ISO (`2020-01-15`, `2020-01`, `2020/01/15`) outside
/// `taken`.
fn iso_dates(segment: &Segment, taken: &[Range<usize>]) -> Vec<(String, Range<usize>)> {
    let text = &segment.text;
    let mut out = Vec::new();
    for (start, _) in text.char_indices() {
        let alone = !text[..start].chars().next_back().is_some_and(joins);
        if !alone
            || out
                .iter()
                .any(|(_, r): &(String, Range<usize>)| r.contains(&start))
        {
            continue;
        }
        let mut fields = Vec::new();
        let mut end = start;
        for (k, width) in [(0, 4..=4), (1, 1..=2), (2, 1..=2)] {
            let from = if k == 0 {
                end
            } else if matches!(text.as_bytes().get(end), Some(b'-' | b'/')) {
                end + 1
            } else {
                break;
            };
            let digits = text[from..].bytes().take_while(u8::is_ascii_digit).count();
            if !width.contains(&digits) {
                break;
            }
            fields.push(&text[from..from + digits]);
            end = from + digits;
        }
        let alone_after = !text[end..].chars().next().is_some_and(joins);
        let span = start..end;
        if fields.len() >= 2 && alone_after && !taken.iter().any(|t| overlaps(t, &span)) {
            out.push((fields.join("-"), span));
        }
    }
    out
}

/// A group of date parts holding a year as ISO text, with its span.
fn iso(group: &[(Range<usize>, Part)]) -> Option<(String, Range<usize>)> {
    let (mut year, mut month, mut day) = (None, None, None);
    for (_, part) in group {
        match *part {
            Part::Year(y) => year = Some(y),
            Part::Month(m) => month = Some(m),
            Part::Day(d) => day = Some(d),
        }
    }
    let span = group.first()?.0.start..group.last()?.0.end;
    let text = match (year?, month, day) {
        (y, Some(m), Some(d)) => format!("{y:04}-{m:02}-{d:02}"),
        (y, Some(m), None) => format!("{y:04}-{m:02}"),
        (y, None, _) => format!("{y:04}"),
    };
    Some((text, span))
}
