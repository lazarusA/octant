//! The dimension a command names and the values said for it, as actions:
//! "time 2020-01-15", "latitude 30 to 60", "go to March 2010" (a date alone
//! moves the time dimension), "lat as y", "animate time".

use std::ops::Range;

use super::values::{dates, numbers};
use super::{AXES, Phrases, Verb};
use crate::app::actions::resolve::Vocabulary;
use crate::app::actions::text::{Name, Segment, overlaps};
use crate::app::actions::{AppAction, DimRole};

/// Role letters said with a dimension ("lat as y"), the same in every
/// language as the Dimensions panel's selects show them.
const ROLE_LETTERS: [(&str, DimRole); 3] =
    [("x", DimRole::X), ("y", DimRole::Y), ("z", DimRole::Z)];

/// The dimension named outside `taken`: one of the dataset's, else an axis
/// word of the language, read as `AXES`.
fn named_dim(
    segment: &Segment,
    phrases: &Phrases,
    vocab: &Vocabulary,
    taken: &[Range<usize>],
) -> Option<(String, Range<usize>)> {
    if let Some(span) = vocab.find_dim(segment, taken) {
        return Some((segment.text[span.clone()].to_string(), span));
    }
    let free = |r: &Range<usize>| !taken.iter().any(|t| overlaps(t, r));
    phrases.axes.iter().zip(AXES).find_map(|(words, axis)| {
        let span = words
            .iter()
            .filter_map(|w| segment.find(&Name::new(w)))
            .find(free)?;
        Some((axis.to_string(), span))
    })
}

/// A role letter said as a word of its own outside `taken`.
fn role_letter(segment: &Segment, taken: &[Range<usize>]) -> Option<(DimRole, Range<usize>)> {
    segment.words.iter().find_map(|w| {
        let (_, role) = ROLE_LETTERS
            .iter()
            .find(|(l, _)| segment.text[w.clone()] == **l)?;
        let free = !taken.iter().any(|t| overlaps(t, w));
        free.then(|| (*role, w.clone()))
    })
}

/// The dimension actions of a command holding `verbs`; the words they use
/// join `taken`. Numbers count only with a named dimension or after "go
/// to", so the 2 of "t 2 m" is no value.
pub(super) fn dim_actions(
    segment: &Segment,
    phrases: &Phrases,
    vocab: &Vocabulary,
    verbs: &[Verb],
    taken: &mut Vec<Range<usize>>,
) -> Vec<AppAction> {
    let dim = named_dim(segment, phrases, vocab, taken);
    let mut around = taken.clone();
    around.extend(dim.as_ref().map(|(_, span)| span.clone()));
    let mut values = dates(segment, &phrases.months, &around);
    if values.is_empty() && (dim.is_some() || verbs.contains(&Verb::Goto)) {
        values = numbers(segment, &around);
    }
    let role = dim.as_ref().and_then(|_| role_letter(segment, &around));
    let name = dim.as_ref().map(|(n, _)| n.clone());
    let animate = values.is_empty() && verbs.contains(&Verb::Play);
    let mut out: Vec<AppAction> = role_action(&name, role.as_ref(), animate, taken)
        .into_iter()
        .collect();
    match values.as_slice() {
        [] => {}
        [(value, _)] => out.push(AppAction::SetDim {
            dim: name.clone(),
            value: value.clone(),
        }),
        [(from, _), (to, _), ..] => out.push(AppAction::SetRange {
            dim: name.clone(),
            from: from.clone(),
            to: to.clone(),
        }),
    }
    if !out.is_empty() {
        taken.extend(values.into_iter().take(2).map(|(_, span)| span));
        taken.extend(dim.map(|(_, span)| span));
    }
    out
}

/// The role a command gives the dimension `name`: a role letter said with
/// it (its word joins `taken`), or animated when it is played.
fn role_action(
    name: &Option<String>,
    role: Option<&(DimRole, Range<usize>)>,
    animate: bool,
    taken: &mut Vec<Range<usize>>,
) -> Option<AppAction> {
    let dim = name.clone()?;
    match role {
        Some((role, span)) => {
            taken.push(span.clone());
            Some(AppAction::SetRole { dim, role: *role })
        }
        None if animate => Some(AppAction::SetRole {
            dim,
            role: DimRole::Animate,
        }),
        None => None,
    }
}
