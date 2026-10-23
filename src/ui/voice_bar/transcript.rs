//! The words heard: the final texts of past utterances, then the partial
//! text of the one under way, muted. One line, which once too long ends at
//! the right edge so the newest words stay in view.

use crate::i18n::{PlaybackKey, tr};
use crate::ui::toast::{Severity, report};
use crate::voice::stt::SttEvent;

/// Characters of final text kept.
const KEEP_CHARS: usize = 400;

#[derive(Default)]
pub(super) struct Transcript {
    finals: String,
    partial: String,
    /// While the provider gets ready: the download's share done, or `None`
    /// while loading.
    preparing: Option<Option<f32>>,
}

impl Transcript {
    pub(super) fn clear(&mut self) {
        self.finals.clear();
        self.partial.clear();
        self.preparing = None;
    }

    pub(super) fn apply(&mut self, event: SttEvent) {
        match event {
            SttEvent::Preparing(done) => self.preparing = Some(done),
            SttEvent::Ready => self.preparing = None,
            SttEvent::Partial(text) => self.partial = text,
            SttEvent::Final(text) => {
                if !self.finals.is_empty() {
                    self.finals.push(' ');
                }
                self.finals.push_str(&text);
                self.partial.clear();
                if let Some((cut, _)) = self.finals.char_indices().rev().nth(KEEP_CHARS) {
                    self.finals.drain(..=cut);
                }
            }
            SttEvent::Failed(e) => {
                self.preparing = None;
                report(Severity::Error, tr(PlaybackKey::VoiceSttFailed), e);
            }
        }
    }

    /// Draws the text in `rect`; while there is none, the provider getting
    /// ready, else `placeholder`.
    pub(super) fn show(&self, ui: &egui::Ui, rect: egui::Rect, placeholder: &str) {
        let visuals = ui.visuals();
        let font = egui::TextStyle::Body.resolve(ui.style());
        let weak = visuals.weak_text_color();
        let mut job = egui::text::LayoutJob::default();
        let format = |color| egui::TextFormat::simple(font.clone(), color);
        let mut buf = [0u8; 128];
        if self.finals.is_empty() && self.partial.is_empty() {
            let note = match self.preparing {
                Some(Some(done)) => {
                    let percent = (done * 100.0).round() as u32;
                    crate::i18n::fill(&mut buf, tr(PlaybackKey::VoiceDownloading), &[&percent])
                }
                Some(None) => tr(PlaybackKey::VoiceLoading),
                None => placeholder,
            };
            job.append(note, 0.0, format(weak));
        } else {
            job.append(&self.finals, 0.0, format(visuals.strong_text_color()));
            let gap = if self.finals.is_empty() {
                0.0
            } else {
                font.size * 0.3
            };
            job.append(&self.partial, gap, format(weak));
        }
        let galley = ui.painter().layout_job(job);
        let width = galley.size().x;
        let x = if width > rect.width() {
            rect.right() - width
        } else {
            rect.left()
        };
        let pos = egui::pos2(x, rect.center().y - galley.size().y * 0.5);
        ui.painter().with_clip_rect(rect).galley(pos, galley, weak);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partials_replace_each_other_and_finals_add_up() {
        let mut t = Transcript::default();
        t.apply(SttEvent::Partial("plot".into()));
        t.apply(SttEvent::Partial("plot the".into()));
        assert_eq!((t.finals.as_str(), t.partial.as_str()), ("", "plot the"));
        t.apply(SttEvent::Final("plot the temperature".into()));
        t.apply(SttEvent::Final("as a volume".into()));
        assert_eq!(t.finals, "plot the temperature as a volume");
        assert!(t.partial.is_empty());
    }

    #[test]
    fn only_the_newest_characters_are_kept() {
        let mut t = Transcript::default();
        for _ in 0..100 {
            t.apply(SttEvent::Final("气温 temperature".into()));
        }
        assert_eq!(t.finals.chars().count(), KEEP_CHARS);
        assert!(t.finals.ends_with("气温 temperature"));
    }
}
