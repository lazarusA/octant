//! Outcomes in the UI language: what was done as a short line, why nothing
//! was as a notice. Names from the dataset or the user stay as they are.

use super::{DimRole, Done, Failure, NameKind};
use crate::i18n::{DatasetKey, KeymapKey, PlaybackKey, VariablesKey, filled, tr};
use crate::ui::toast::{Notice, Severity};

pub(super) fn done_text(done: &Done) -> String {
    match done {
        Done::Opening(name) => filled(tr(PlaybackKey::VoiceDidOpen), &[name]),
        Done::CatalogOpened => tr(DatasetKey::CatalogTitle).to_string(),
        Done::DimSet { dim, value } => filled(tr(PlaybackKey::VoiceDidSetDim), &[dim, value]),
        Done::RangeSet { dim, from, to } => {
            filled(tr(PlaybackKey::VoiceDidSetRange), &[dim, from, to])
        }
        Done::RoleSet { dim, role } => {
            filled(tr(PlaybackKey::VoiceDidSetDim), &[dim, &role_name(*role)])
        }
        Done::DatasetShown {
            name, variables, ..
        } => filled(tr(PlaybackKey::VoiceDatasetSummary), &[name, variables]),
        Done::Searching { text, matches } => {
            filled(tr(PlaybackKey::VoiceSearching), &[text, matches])
        }
        Done::VariableShown {
            name,
            long_name,
            units,
            ..
        } => variable_line(name, long_name.as_deref(), units.as_deref()),
        Done::ViewShown {
            variable,
            plot_type,
            ..
        } => filled(
            tr(PlaybackKey::VoiceViewSummary),
            &[variable, &plot_type.display_name()],
        ),
        Done::Selected(name) => filled(tr(PlaybackKey::VoiceDidSelect), &[name]),
        Done::Plotted {
            variable,
            plot_type,
        } => filled(
            tr(PlaybackKey::VoiceDidPlot),
            &[variable, &plot_type.display_name()],
        ),
        Done::Playing => tr(PlaybackKey::Play).to_string(),
        Done::Paused => tr(PlaybackKey::Pause).to_string(),
        Done::Stepped(by) if *by < 0 => tr(PlaybackKey::PreviousStep).to_string(),
        Done::Stepped(_) => tr(PlaybackKey::NextStep).to_string(),
        Done::Colormap(key) => filled(tr(PlaybackKey::VoiceDidColormap), &[key]),
        Done::ViewReset => tr(KeymapKey::ResetView).to_string(),
    }
}

/// A variable as `name - long name [units]`, the parts it has.
fn variable_line(name: &str, long_name: Option<&str>, units: Option<&str>) -> String {
    let long = long_name.map(|l| format!(" - {l}")).unwrap_or_default();
    let units = units.map(|u| format!(" [{u}]")).unwrap_or_default();
    format!("{name}{long}{units}")
}

/// What an answer shows beyond its line: a dataset's entries, a
/// variable's dimensions, the plot's step, colormap, overlays and the
/// variable staged, one per line.
pub(super) fn done_detail(done: &Done) -> Option<String> {
    let lines: Vec<String> = match done {
        Done::DatasetShown { entries, .. } => vec![entries.join(", ")],
        Done::VariableShown { dims, .. } => dims
            .iter()
            .map(|d| match d.size {
                0 | 1 => format!("{}: {}", d.name, d.first),
                n => format!("{} ({n}): {} - {}", d.name, d.first, d.last),
            })
            .collect(),
        Done::ViewShown {
            step,
            colormap,
            overlays,
            staged,
            ..
        } => {
            let step = step
                .iter()
                .map(|(dim, value)| filled(tr(PlaybackKey::VoiceDidSetDim), &[dim, value]));
            let colormap = std::iter::once(filled(tr(PlaybackKey::VoiceDidColormap), &[colormap]));
            let overlays =
                (*overlays > 0).then(|| filled(tr(PlaybackKey::VoiceOverlayCount), &[overlays]));
            let staged = staged
                .iter()
                .map(|s| filled(tr(PlaybackKey::VoiceStaged), &[s]));
            step.chain(colormap).chain(overlays).chain(staged).collect()
        }
        _ => return None,
    };
    Some(lines.join("\n"))
}

/// The notice of `failure`; none when the app already gave one.
pub(super) fn failure_notice(failure: &Failure) -> Option<Notice> {
    let (title, detail) = match failure {
        Failure::Refused | Failure::CoordinatesPending => return None,
        Failure::NoValue { dim, value } => (
            filled(tr(PlaybackKey::VoiceNoValue), &[dim, value]),
            String::new(),
        ),
        Failure::NoDataset => (tr(PlaybackKey::VoiceNoDataset).to_string(), String::new()),
        Failure::NotFound {
            kind,
            text,
            closest,
        } => {
            let key = match kind {
                NameKind::Source => PlaybackKey::VoiceNoSource,
                NameKind::Group => PlaybackKey::VoiceNoGroup,
                NameKind::Dimension => PlaybackKey::VoiceNoDimension,
                NameKind::Variable => PlaybackKey::VoiceNoVariable,
                NameKind::PlotType => PlaybackKey::VoiceNoPlotType,
                NameKind::Colormap => PlaybackKey::VoiceNoColormap,
            };
            (filled(tr(key), &[text]), did_you_mean(closest))
        }
        Failure::Ambiguous { text, matches, .. } => (
            filled(tr(PlaybackKey::VoiceAmbiguous), &[text]),
            did_you_mean(matches),
        ),
        Failure::Blocked { plot_type, reason } => (
            filled(
                tr(PlaybackKey::VoiceCannotPlot),
                &[&plot_type.display_name()],
            ),
            reason.clone(),
        ),
        Failure::NothingToAnimate => (
            tr(PlaybackKey::VoiceNothingToAnimate).to_string(),
            String::new(),
        ),
        Failure::NotUnderstood(text) => (
            tr(PlaybackKey::VoiceNotUnderstood).to_string(),
            text.clone(),
        ),
    };
    Some(Notice::new(Severity::Warning, title, detail))
}

fn did_you_mean(names: &[String]) -> String {
    if names.is_empty() {
        return String::new();
    }
    filled(tr(PlaybackKey::VoiceDidYouMean), &[&names.join(", ")])
}

/// A role as the Dimensions panel's selects name it.
fn role_name(role: DimRole) -> &'static str {
    match role {
        DimRole::X => "X",
        DimRole::Y => "Y",
        DimRole::Z => "Z",
        DimRole::Animate => tr(VariablesKey::RoleAnimated),
        DimRole::None => tr(VariablesKey::RoleNone),
    }
}
