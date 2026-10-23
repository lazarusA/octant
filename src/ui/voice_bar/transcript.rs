//! The words heard: the final texts of past utterances, then the partial
//! text of the one under way, muted. One line, which once too long glides
//! left as words come (and back as a partial is revised shorter) so the
//! newest stay in view, the oldest fading out at the left edge.

use crate::i18n::{PlaybackKey, tr};
use crate::ui::toast::{Severity, report};
use crate::voice::stt::SttEvent;

/// Characters of final text kept.
const KEEP_CHARS: usize = 400;
/// Seconds the line takes to glide to a new place.
const GLIDE: f32 = 0.35;
/// Width of the fade at the left edge.
const FADE: f32 = 56.0;
/// Strips the fade is drawn in, eased between.
const FADE_STEPS: usize = 8;

#[derive(Default)]
pub(super) struct Transcript {
    finals: String,
    partial: String,
    /// While the provider gets ready: the download's share done, or `None`
    /// while loading.
    preparing: Option<Option<f32>>,
    /// Bumped when the text is cut or cleared, so the line takes its new
    /// place at once instead of gliding there.
    jumps: u32,
}

impl Transcript {
    pub(super) fn clear(&mut self) {
        self.finals.clear();
        self.partial.clear();
        self.preparing = None;
        self.jumps += 1;
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
                    self.jumps += 1;
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
        let overflow = (galley.size().x - rect.width()).max(0.0);
        let id = ui.scope_id().with(("voice_transcript", self.jumps));
        let scroll = ui.ctx().animate_value_with_time(id, overflow, GLIDE);
        let pos = egui::pos2(
            rect.left() - scroll,
            rect.center().y - galley.size().y * 0.5,
        );
        let painter = ui.painter().with_clip_rect(rect);
        painter.galley(pos, galley, weak);
        if scroll > 0.5 {
            // Fade in as the line starts to scroll, not all at once.
            let width = FADE.min(scroll).min(rect.width() * 0.5);
            fade_left(&painter, rect, width, visuals.panel_fill);
        }
    }
}

/// Covers the left `width` of `rect` with `ground`, opaque at the edge and
/// clear on the inside, eased so the text fades out without a visible line.
fn fade_left(painter: &egui::Painter, rect: egui::Rect, width: f32, ground: egui::Color32) {
    let mut mesh = egui::Mesh::default();
    for i in 0..=FADE_STEPS {
        let t = i as f32 / FADE_STEPS as f32;
        let x = rect.left() + width * t;
        let cover = ground.gamma_multiply(1.0 - t * t * (3.0 - 2.0 * t));
        mesh.colored_vertex(egui::pos2(x, rect.top()), cover);
        mesh.colored_vertex(egui::pos2(x, rect.bottom()), cover);
    }
    for i in 0..FADE_STEPS as u32 {
        let k = i * 2;
        mesh.add_triangle(k, k + 1, k + 2);
        mesh.add_triangle(k + 1, k + 3, k + 2);
    }
    painter.add(mesh);
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
