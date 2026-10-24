//! Outcomes in the UI language: what was done as a short line, why nothing
//! was as a notice. Names from the dataset or the user stay as they are.

use super::{DimRole, Done, Failure, NameKind};
use crate::i18n::{DatasetKey, ExportKey, KeymapKey, PlaybackKey, PlotKey, filled, tr};
use crate::ui::toast::{Notice, Severity};

pub(super) fn done_text(done: &Done) -> String {
    match done {
        Done::Opening(name) => filled(tr(PlaybackKey::VoiceDidOpen), &[name]),
        Done::CatalogOpened => tr(DatasetKey::CatalogTitle).to_string(),
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
        Done::DimSet { .. }
        | Done::RangeSet { .. }
        | Done::RoleSet { .. }
        | Done::OverlayAdded(_)
        | Done::LayerShown { .. }
        | Done::OverlayRemoved(_)
        | Done::ColorRange { .. }
        | Done::ExportOpened(_)
        | Done::WellPicked(_) => layer_line(done),
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

/// The line of what an action set: a dimension, a layer, the export
/// dialog, a plate's well (empty for other outcomes).
fn layer_line(done: &Done) -> String {
    let number = |v: &f32| crate::i18n::number::localize_string(v.to_string());
    match done {
        Done::DimSet { dim, value } => filled(tr(PlaybackKey::VoiceDidSetDim), &[dim, value]),
        Done::RangeSet { dim, from, to } => {
            filled(tr(PlaybackKey::VoiceDidSetRange), &[dim, from, to])
        }
        Done::RoleSet { dim, role } => {
            filled(tr(PlaybackKey::VoiceDidSetDim), &[dim, &role_name(*role)])
        }
        Done::OverlayAdded(name) => filled(tr(PlaybackKey::VoiceOverlayAdded), &[name]),
        Done::LayerShown { layer, visible } => {
            let key = if *visible {
                PlaybackKey::VoiceLayerShown
            } else {
                PlaybackKey::VoiceLayerHidden
            };
            filled(tr(key), &[layer])
        }
        Done::OverlayRemoved(name) => filled(tr(PlaybackKey::VoiceOverlayRemoved), &[name]),
        Done::ColorRange { layer, min, max } => filled(
            tr(PlaybackKey::VoiceColorRange),
            &[layer, &number(min), &number(max)],
        ),
        Done::ExportOpened(format) => filled(
            tr(PlaybackKey::VoiceDidSetDim),
            &[&tr(ExportKey::DialogTitle), format],
        ),
        Done::WellPicked(well) => filled(tr(PlaybackKey::VoiceWellPicked), &[well]),
        _ => String::new(),
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
        Failure::Refused | Failure::Reading => return None,
        Failure::OverlayUnavailable(reason) => {
            (tr(PlotKey::OverlayCantAdd).to_string(), reason.clone())
        }
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
            let title = filled(tr(not_found_key(*kind)), &[text]);
            (title, did_you_mean(closest))
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
        // Said in the bar (`not_understood_line`): speech not meant as a
        // command would fill the toasts.
        Failure::NotUnderstood(_) => return None,
    };
    Some(Notice::new(Severity::Warning, title, detail))
}

/// The template saying no name of `kind` matches.
fn not_found_key(kind: NameKind) -> PlaybackKey {
    match kind {
        NameKind::Source => PlaybackKey::VoiceNoSource,
        NameKind::Layer => PlaybackKey::VoiceNoLayer,
        NameKind::Format => PlaybackKey::VoiceNoFormat,
        NameKind::Well => PlaybackKey::VoiceNoWell,
        NameKind::Group => PlaybackKey::VoiceNoGroup,
        NameKind::Dimension => PlaybackKey::VoiceNoDimension,
        NameKind::Variable => PlaybackKey::VoiceNoVariable,
        NameKind::PlotType => PlaybackKey::VoiceNoPlotType,
        NameKind::Colormap => PlaybackKey::VoiceNoColormap,
    }
}

/// The bar's line for text no command was read in.
pub(super) fn not_understood_line(text: &str) -> String {
    filled(tr(PlaybackKey::VoiceNotUnderstood), &[&text])
}

fn did_you_mean(names: &[String]) -> String {
    if names.is_empty() {
        return String::new();
    }
    filled(tr(PlaybackKey::VoiceDidYouMean), &[&names.join(", ")])
}

/// A role as the Dimensions panel's selects name it.
fn role_name(role: DimRole) -> &'static str {
    use crate::app::{AnimationRole, SpatialRole};
    use crate::ui::variables_panel::dimension_slider::{anim_name, spatial_name};
    match role {
        DimRole::X => spatial_name(SpatialRole::X),
        DimRole::Y => spatial_name(SpatialRole::Y),
        DimRole::Z => spatial_name(SpatialRole::Z),
        DimRole::Animate => anim_name(AnimationRole::Animated),
        DimRole::None => anim_name(AnimationRole::None),
    }
}
