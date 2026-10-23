//! The voice bar's Whisper model select.

use super::VoiceBar;
use crate::i18n::{PlaybackKey, tr};
use crate::voice::stt::{self, Provider};

/// The Whisper model select; a new choice takes over while listening.
pub(super) fn model_menu(voice: &mut VoiceBar, ui: &mut egui::Ui) -> egui::Response {
    use crate::voice::stt::WhisperSize;
    let Provider::Whisper(current) = voice.provider;
    let mut chosen = current;
    let response = egui::ComboBox::from_id_salt("voice_model")
        .selected_text(current.name())
        .show_ui(ui, |ui| {
            for &size in WhisperSize::OFFERED {
                let mut buf = [0u8; 64];
                let mb = crate::voice::whisper::megabytes(size);
                let label = crate::i18n::fill(
                    &mut buf,
                    tr(PlaybackKey::VoiceModelSize),
                    &[&size.name(), &mb],
                );
                ui.selectable_value(&mut chosen, size, label);
            }
        })
        .response
        .on_hover_text(tr(PlaybackKey::VoiceModelHover));
    if chosen != current {
        voice.provider = Provider::Whisper(chosen);
        voice.stt = voice
            .is_listening()
            .then(|| stt::open(voice.provider))
            .flatten();
        voice.transcript.clear();
    }
    response
}
