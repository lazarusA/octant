//! Outcomes in the UI language: what was done as a short line, why nothing
//! was as a notice. Names from the dataset or the user stay as they are.

use super::{Done, Failure, NameKind};
use crate::i18n::{KeymapKey, PlaybackKey, filled, tr};
use crate::ui::toast::{Notice, Severity};

pub(super) fn done_text(done: &Done) -> String {
    match done {
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

pub(super) fn failure_notice(failure: &Failure) -> Notice {
    let (title, detail) = match failure {
        Failure::NoDataset => (tr(PlaybackKey::VoiceNoDataset).to_string(), String::new()),
        Failure::NotFound {
            kind,
            text,
            closest,
        } => {
            let key = match kind {
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
    Notice::new(Severity::Warning, title, detail)
}

fn did_you_mean(names: &[String]) -> String {
    if names.is_empty() {
        return String::new();
    }
    filled(tr(PlaybackKey::VoiceDidYouMean), &[&names.join(", ")])
}
