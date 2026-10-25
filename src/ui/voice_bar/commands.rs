//! The voice bar's commands: the utterances it hands the app (spoken, or
//! typed in its field) and the last command's action.

use super::VoiceBar;
use crate::i18n::{PlaybackKey, tr};
use crate::voice::stt::SttEvent;

/// Width of the typed command field.
const FIELD_W: f32 = 200.0;

/// The typed command field, which the `M` shortcut focuses.
pub fn field_id() -> egui::Id {
    egui::Id::unique("voice_command_field")
}

impl VoiceBar {
    /// The utterances said or typed since the last call.
    pub fn take_utterances(&mut self) -> Vec<String> {
        std::mem::take(&mut self.utterances)
    }

    /// What the last command did, in the UI language.
    pub fn set_last_action(&mut self, text: String) {
        self.transcript.action = text;
    }

    /// Opens the bar for typing, the microphone off.
    pub fn open_for_typing(&mut self) {
        self.open = true;
    }

    /// The typed command field: Enter runs its text as an utterance.
    pub(super) fn command_field(&mut self, ui: &mut egui::Ui) {
        let field = egui::TextEdit::singleline(&mut self.typed)
            .id(field_id())
            .hint_text(tr(PlaybackKey::VoiceTypeHint))
            .desired_width(FIELD_W);
        let response = ui.add(field);
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let text = std::mem::take(&mut self.typed).trim().to_string();
            if !text.is_empty() {
                self.transcript.apply(SttEvent::Final(text.clone()));
                self.utterances.push(text);
            }
            response.request_focus();
        }
    }
}
