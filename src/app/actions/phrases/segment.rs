//! One command of an utterance read into actions: its verbs, then the names
//! it holds (a plot type, a colormap after a colormap verb, the catalog,
//! dimensions and values, a variable, a dataset to open).

use super::named::{Named, named};
use super::{Parsed, Phrases, Verb};
use crate::app::actions::AppAction;
use crate::app::actions::resolve::{Resolution, Vocabulary};
use crate::app::actions::text::Segment;

/// The actions of one command.
pub(super) fn parse_segment(
    segment: &Segment,
    phrases: &Phrases,
    vocab: &Vocabulary,
) -> Vec<Parsed> {
    let n = named(segment, phrases, vocab);
    let mut out = if n.verbs.is_empty() {
        names_alone(&n, segment)
    } else {
        verb_actions(&n, segment, vocab)
    };
    // Dimensions are set once the variable is chosen, before it is plotted
    // or played.
    let starts_plot = |p: &Parsed| {
        matches!(
            p,
            Parsed::Action(
                AppAction::Plot { .. }
                    | AppAction::Play
                    | AppAction::Pause
                    | AppAction::Step { .. }
            )
        )
    };
    let at = out.iter().position(starts_plot).unwrap_or(out.len());
    out.splice(at..at, n.dims.into_iter().map(Parsed::Action));
    // A field opens before what is done with its data.
    out.splice(0..0, n.layers.plate.map(Parsed::Action));
    out
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
        (None, None, None) if !n.dims.is_empty() || n.layers.plate.is_some() => {}
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
    // A variable a command names without choosing it is the layer it
    // colors ("colormap viridis for sst").
    let chooses = n
        .verbs
        .iter()
        .any(|v| matches!(v, Verb::Plot | Verb::Select | Verb::Open));
    let layer = n.variable.clone().filter(|_| !chooses);
    for &verb in &n.verbs {
        if last.replace(verb) == Some(verb) {
            continue;
        }
        let parsed = match verb {
            Verb::Open | Verb::Select => {
                // A well and field said are what is opened.
                if (selected && n.variable.is_some()) || n.layers.plate.is_some() {
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
                if n.layers.overlay {
                    action(AppAction::AddOverlay)
                } else {
                    plot(n.plot_type.as_deref())
                }
            }
            Verb::Goto if !n.dims.is_empty() => continue,
            _ => plain_action(n, verb, segment, layer.as_deref()),
        };
        out.push(parsed);
    }
    out
}

/// The action of a verb that needs no other verb's: asking, layers,
/// colors, export, playback.
fn plain_action(n: &Named, verb: Verb, segment: &Segment, layer: Option<&str>) -> Parsed {
    match verb {
        Verb::Search if n.query.is_empty() => not_understood(segment),
        Verb::Search => action(AppAction::Search {
            text: n.query.clone(),
        }),
        Verb::Describe => action(describe(n)),
        Verb::List => action(AppAction::DescribeDataset {
            group: n.group.clone(),
        }),
        Verb::Goto if n.dims.is_empty() => not_understood(segment),
        Verb::Hide | Verb::Reveal | Verb::Remove => layer_action(n, verb, segment),
        Verb::Range => match n.layers.range {
            Some((min, max)) => action(AppAction::SetColorRange {
                min,
                max,
                layer: layer.map(str::to_string),
            }),
            None => not_understood(segment),
        },
        Verb::Export => action(AppAction::Export {
            format: n.layers.format.clone(),
        }),
        Verb::Play => action(AppAction::Play),
        Verb::Pause => action(AppAction::Pause),
        Verb::Next => action(AppAction::Step { by: 1 }),
        Verb::Previous => action(AppAction::Step { by: -1 }),
        Verb::ResetView => action(AppAction::ResetView),
        Verb::Colormap => match or_rest(n, &n.colormap) {
            Some(name) => action(AppAction::SetColormap {
                name,
                layer: layer.map(str::to_string),
            }),
            None => not_understood(segment),
        },
        Verb::Open | Verb::Select | Verb::Plot | Verb::Goto => not_understood(segment),
    }
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

/// What "describe" asks about: a variable named, a group, the dataset, else
/// the plot.
fn describe(n: &Named) -> AppAction {
    match (&n.variable, &n.group) {
        (Some(path), _) => AppAction::DescribeVariable {
            path: Some(path.clone()),
        },
        (None, Some(group)) => AppAction::DescribeDataset {
            group: Some(group.clone()),
        },
        (None, None) if n.dataset => AppAction::DescribeDataset { group: None },
        (None, None) => AppAction::DescribeView,
    }
}

/// "hide", "show again" or "remove" the layer of the variable named (or the
/// leftover words).
fn layer_action(n: &Named, verb: Verb, segment: &Segment) -> Parsed {
    let Some(layer) = or_rest(n, &n.variable) else {
        return not_understood(segment);
    };
    action(match verb {
        Verb::Remove => AppAction::RemoveOverlay { layer },
        _ => AppAction::SetLayerVisible {
            layer,
            visible: verb == Verb::Reveal,
        },
    })
}
