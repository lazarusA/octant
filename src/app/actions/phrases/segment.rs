//! One command of an utterance read into actions: its verbs, then the
//! names it holds (a plot type, a colormap after a colormap verb, a
//! variable), each found outside the others.

use std::ops::Range;

use super::{Parsed, Phrases, Verb, verbs_in};
use crate::app::actions::AppAction;
use crate::app::actions::resolve::{Resolution, Vocabulary};
use crate::app::actions::text::{Segment, overlaps};

fn spans(found: &[(Range<usize>, Verb)]) -> Vec<Range<usize>> {
    found.iter().map(|(r, _)| r.clone()).collect()
}

/// What one command names: its verbs, then a colormap (only after a
/// colormap verb), a plot type and a variable, each outside the others.
struct Named {
    verbs: Vec<Verb>,
    colormap: Option<String>,
    plot_type: Option<String>,
    variable: Option<String>,
    /// The words left over: a name not found, or filler.
    rest: String,
}

fn named(segment: &Segment, phrases: &Phrases, vocab: &Vocabulary) -> Named {
    let mut found = verbs_in(phrases, segment);
    // A verb and a plot type name overlapping ("color map" and "map", 画
    // and 热力图): the longer is meant.
    let mut plot_span = vocab.find_plot_type(segment, &[]);
    if let Some(p) = &plot_span {
        let overlapping = |r: &Range<usize>| overlaps(r, p);
        if found
            .iter()
            .any(|(r, _)| overlapping(r) && r.len() >= p.len())
        {
            plot_span = vocab.find_plot_type(segment, &spans(&found));
        } else {
            found.retain(|(r, _)| !overlapping(r));
        }
    }
    let mut taken = spans(&found);
    let verbs: Vec<Verb> = found.into_iter().map(|(_, v)| v).collect();
    let take = |span: Option<Range<usize>>, taken: &mut Vec<Range<usize>>| {
        let text = span.as_ref().map(|r| segment.text[r.clone()].to_string());
        taken.extend(span);
        text
    };
    let plot_type = take(plot_span, &mut taken);
    let colormap = if verbs.contains(&Verb::Colormap) {
        take(vocab.find_colormap(segment, &taken), &mut taken)
    } else {
        None
    };
    let variable = take(vocab.find_variable(segment, &taken), &mut taken);
    Named {
        rest: segment.rest(&taken),
        verbs,
        colormap,
        plot_type,
        variable,
    }
}

/// The actions of one command.
pub(super) fn parse_segment(
    segment: &Segment,
    phrases: &Phrases,
    vocab: &Vocabulary,
) -> Vec<Parsed> {
    let n = named(segment, phrases, vocab);
    if n.verbs.is_empty() {
        names_alone(&n, segment)
    } else {
        verb_actions(&n, segment, vocab)
    }
}

/// A command without a verb: a plot type plots (the variable named, if
/// any), a variable alone is selected.
fn names_alone(n: &Named, segment: &Segment) -> Vec<Parsed> {
    let mut out = Vec::new();
    match (&n.variable, &n.plot_type) {
        (variable, Some(plot_type)) => {
            if let Some(v) = variable {
                select(&mut out, v);
            }
            out.push(plot(Some(plot_type)));
        }
        (Some(v), None) => select(&mut out, v),
        (None, None) => out.push(not_understood(segment)),
    }
    out
}

/// Each verb's action, a verb said twice in a row once; the variable is
/// selected once, by the first verb naming one.
fn verb_actions(n: &Named, segment: &Segment, vocab: &Vocabulary) -> Vec<Parsed> {
    let mut out = Vec::new();
    // What a verb names: one found, else the leftover words.
    let or_rest = |found: &Option<String>| {
        found
            .clone()
            .or_else(|| (!n.rest.is_empty()).then(|| n.rest.clone()))
    };
    let mut selected = false;
    let mut last = None;
    for &verb in &n.verbs {
        if last.replace(verb) == Some(verb) {
            continue;
        }
        let action = |a| Parsed::Action(a);
        let parsed = match verb {
            Verb::Select => match or_rest(&n.variable) {
                Some(_) if selected => continue,
                Some(path) => {
                    selected = true;
                    action(AppAction::SelectVariable { path })
                }
                None => not_understood(segment),
            },
            Verb::Plot => {
                if !selected && let Some(v) = plot_target(n, vocab) {
                    select(&mut out, &v);
                    selected = true;
                }
                plot(n.plot_type.as_deref())
            }
            Verb::Play => action(AppAction::Play),
            Verb::Pause => action(AppAction::Pause),
            Verb::Next => action(AppAction::Step { by: 1 }),
            Verb::Previous => action(AppAction::Step { by: -1 }),
            Verb::ResetView => action(AppAction::ResetView),
            Verb::Colormap => match or_rest(&n.colormap) {
                Some(name) => action(AppAction::SetColormap { name }),
                None => not_understood(segment),
            },
        };
        out.push(parsed);
    }
    out
}

/// The variable "plot" names: one found, else the leftover words when they
/// come close to a variable's name ("plot the temperatur"), so a mistaken
/// name answers with the closest ones; other leftovers ("plot it as a
/// volume") are filler and the staged variable is plotted.
fn plot_target(n: &Named, vocab: &Vocabulary) -> Option<String> {
    if let Some(v) = &n.variable {
        return Some(v.clone());
    }
    if n.rest.is_empty() {
        return None;
    }
    match vocab.variable(&n.rest) {
        Resolution::NotFound(closest) if closest.is_empty() => None,
        _ => Some(n.rest.clone()),
    }
}

fn not_understood(segment: &Segment) -> Parsed {
    Parsed::NotUnderstood(segment.text.trim().to_string())
}

fn select(out: &mut Vec<Parsed>, path: &str) {
    out.push(Parsed::Action(AppAction::SelectVariable {
        path: path.to_string(),
    }));
}

fn plot(plot_type: Option<&str>) -> Parsed {
    Parsed::Action(AppAction::Plot {
        plot_type: plot_type.map(str::to_string),
    })
}
