//! One command of an utterance read into actions: its verbs, then the
//! names it holds (a plot type, a colormap after a colormap verb, a
//! variable, a dataset to open, the catalog), each found outside the verbs.

use std::ops::Range;

use super::{Parsed, Phrases, Verb, verbs_in};
use crate::app::actions::AppAction;
use crate::app::actions::resolve::{Resolution, Vocabulary};
use crate::app::actions::text::{Name, Segment, overlaps};

fn spans(found: &[(Range<usize>, Verb)]) -> Vec<Range<usize>> {
    found.iter().map(|(r, _)| r.clone()).collect()
}

/// What one command names.
struct Named {
    verbs: Vec<Verb>,
    colormap: Option<String>,
    plot_type: Option<String>,
    variable: Option<String>,
    /// A dataset to open: a landing sample, catalog entry or open dataset.
    source: Option<String>,
    /// Whether it names the dataset catalog.
    catalog: bool,
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
    let catalog_word = phrases
        .catalog
        .iter()
        .filter_map(|w| segment.find(&Name::new(w)))
        .find(|s| !taken.iter().any(|t| overlaps(t, s)));
    let catalog = take(catalog_word, &mut taken).is_some();
    // A variable and a dataset may share a name; the verb decides.
    let names_from = taken.clone();
    let variable = take(vocab.find_variable(segment, &names_from), &mut taken);
    let source = take(vocab.find_source(segment, &names_from), &mut taken);
    Named {
        rest: segment.rest(&taken),
        verbs,
        colormap,
        plot_type,
        variable,
        source,
        catalog,
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
/// any), a variable alone is selected, a dataset opened, the catalog shown.
fn names_alone(n: &Named, segment: &Segment) -> Vec<Parsed> {
    let mut out = Vec::new();
    match (&n.variable, &n.plot_type, &n.source) {
        (variable, Some(plot_type), _) => {
            if let Some(v) = variable {
                out.push(select(v));
            }
            out.push(plot(Some(plot_type)));
        }
        (Some(v), None, _) => out.push(select(v)),
        (None, None, Some(source)) => out.push(open(source)),
        (None, None, None) if n.catalog => out.push(action(AppAction::OpenCatalog)),
        (None, None, None) => out.push(not_understood(segment)),
    }
    out
}

/// Each verb's action, a verb said twice in a row once; the variable is
/// selected once, by the first verb naming one.
fn verb_actions(n: &Named, segment: &Segment, vocab: &Vocabulary) -> Vec<Parsed> {
    let mut out = Vec::new();
    let mut selected = false;
    let mut last = None;
    for &verb in &n.verbs {
        if last.replace(verb) == Some(verb) {
            continue;
        }
        let parsed = match verb {
            Verb::Open | Verb::Select => {
                if selected && n.variable.is_some() {
                    continue;
                }
                selected = true;
                choose(n, verb, segment)
            }
            Verb::Plot if n.catalog && n.variable.is_none() => action(AppAction::OpenCatalog),
            Verb::Plot => {
                if !selected && let Some(v) = plot_target(n, vocab) {
                    out.push(select(&v));
                    selected = true;
                }
                plot(n.plot_type.as_deref())
            }
            Verb::Play => action(AppAction::Play),
            Verb::Pause => action(AppAction::Pause),
            Verb::Next => action(AppAction::Step { by: 1 }),
            Verb::Previous => action(AppAction::Step { by: -1 }),
            Verb::ResetView => action(AppAction::ResetView),
            Verb::Colormap => match or_rest(n, &n.colormap) {
                Some(name) => action(AppAction::SetColormap { name }),
                None => not_understood(segment),
            },
        };
        out.push(parsed);
    }
    out
}

/// "open" or "select": a dataset for "open" (a variable when only one is
/// named), a variable for "select" (a dataset when only one is named), the
/// catalog when named alone, else the leftover words as what was meant.
fn choose(n: &Named, verb: Verb, segment: &Segment) -> Parsed {
    let opens = verb == Verb::Open;
    match (&n.source, &n.variable) {
        (Some(s), None) => open(s),
        (Some(s), Some(_)) if opens => open(s),
        (_, Some(v)) => select(v),
        (None, None) if n.catalog => action(AppAction::OpenCatalog),
        (None, None) => match or_rest(n, &None) {
            Some(rest) if opens => open(&rest),
            Some(rest) => select(&rest),
            None => not_understood(segment),
        },
    }
}

/// A name found, else the leftover words.
fn or_rest(n: &Named, found: &Option<String>) -> Option<String> {
    found
        .clone()
        .or_else(|| (!n.rest.is_empty()).then(|| n.rest.clone()))
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

fn action(action: AppAction) -> Parsed {
    Parsed::Action(action)
}

fn not_understood(segment: &Segment) -> Parsed {
    Parsed::NotUnderstood(segment.text.trim().to_string())
}

fn select(path: &str) -> Parsed {
    action(AppAction::SelectVariable {
        path: path.to_string(),
    })
}

fn open(source: &str) -> Parsed {
    action(AppAction::OpenSource {
        source: source.to_string(),
    })
}

fn plot(plot_type: Option<&str>) -> Parsed {
    action(AppAction::Plot {
        plot_type: plot_type.map(str::to_string),
    })
}
