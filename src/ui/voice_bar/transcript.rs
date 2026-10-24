//! The words heard: the final texts of past utterances, then the partial
//! text of the one under way. Only the newest words are in full color; older
//! ones ease to the muted grey and the oldest further toward the background.
//! One line, which once too long glides
//! left as words come (and back as a partial is revised shorter) so the
//! newest stay in view, the oldest fading out at the left edge.

use crate::i18n::{PlaybackKey, tr};
use crate::ui::toast::{Severity, report};
use crate::voice::stt::SttEvent;

/// Characters of final text kept.
const KEEP_CHARS: usize = 400;
/// Seconds the line takes to glide to a new place.
const GLIDE: f32 = 0.35;
/// Words, counted from the newest, kept in full color, and over how many
/// more the older ones fade to the faintest grey.
const FRESH_WORDS: usize = 6;
const FADING_WORDS: usize = 18;
/// How far the oldest words go from the muted text color to the background.
const OLDEST: f32 = 0.55;
/// Width of the fade at the left edge.
const FADE: f32 = 56.0;
/// Strips the fade is drawn in, eased between.
const FADE_STEPS: usize = 8;
/// Share of the line the last command's action may take.
const ACTION_SHARE: f32 = 0.4;

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
    /// What the last command did, shown beside the text.
    pub(super) action: String,
}

impl Transcript {
    /// Clears the text; the provider's state stays, as the provider does.
    pub(super) fn clear(&mut self) {
        self.finals.clear();
        self.partial.clear();
        self.action.clear();
        self.jumps += 1;
    }

    /// The provider was dropped: nothing gets ready any more.
    pub(super) fn provider_dropped(&mut self) {
        self.preparing = None;
    }

    pub(super) fn is_preparing(&self) -> bool {
        self.preparing.is_some()
    }

    /// Takes `event`; true when the provider failed while getting ready, so
    /// it has stopped and listening again needs a new one.
    pub(super) fn apply(&mut self, event: SttEvent) -> bool {
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
                report(Severity::Error, tr(PlaybackKey::VoiceSttFailed), e);
                return self.preparing.take().is_some();
            }
        }
        false
    }

    /// What the last command did, at the right of the line.
    pub(super) fn show_action(&self, ui: &mut egui::Ui) {
        if !self.action.is_empty() {
            let text = egui::RichText::new(&self.action).strong();
            // Cut long names rather than the words heard.
            let width = ui.available_width() * ACTION_SHARE;
            ui.scope(|ui| {
                ui.set_max_width(width);
                ui.add(egui::Label::new(text).truncate());
            });
        }
    }

    /// Draws the text in `rect`; while there is none, the provider getting
    /// ready, else `placeholder`.
    pub(super) fn show(&self, ui: &egui::Ui, rect: egui::Rect, placeholder: &str) {
        let mut buf = [0u8; 128];
        let galley = ui
            .painter()
            .layout_job(self.line(ui, placeholder, &mut buf));
        let overflow = (galley.size().x - rect.width()).max(0.0);
        let id = ui.scope_id().with(("voice_transcript", self.jumps));
        let scroll = ui.ctx().animate_value_with_time(id, overflow, GLIDE);
        let pos = egui::pos2(
            rect.left() - scroll,
            rect.center().y - galley.size().y * 0.5,
        );
        let painter = ui.painter().with_clip_rect(rect);
        painter.galley(pos, galley, ui.visuals().weak_text_color());
        if scroll > 0.5 {
            // Fade in as the line starts to scroll, not all at once.
            let width = FADE.min(scroll).min(rect.width() * 0.5);
            fade_left(&painter, rect, width, ui.visuals().panel_fill);
        }
    }

    /// The line to draw: the words colored by age, or a note while there are
    /// none (formatted into `buf`).
    fn line(&self, ui: &egui::Ui, placeholder: &str, buf: &mut [u8]) -> egui::text::LayoutJob {
        let visuals = ui.visuals();
        let font = egui::TextStyle::Body.resolve(ui.style());
        let weak = visuals.weak_text_color();
        let format = |color| egui::TextFormat::simple(font.clone(), color);
        let mut job = egui::text::LayoutJob::default();
        if self.finals.is_empty() && self.partial.is_empty() {
            let note = match self.preparing {
                Some(Some(done)) => {
                    let percent = (done * 100.0).round() as u32;
                    crate::i18n::fill(buf, tr(PlaybackKey::VoiceDownloading), &[&percent])
                }
                Some(None) => tr(PlaybackKey::VoiceLoading),
                None => placeholder,
            };
            job.append(note, 0.0, format(weak));
            return job;
        }
        let (strong, faint) = (
            visuals.strong_text_color(),
            weak.lerp_to_gamma(visuals.panel_fill, OLDEST),
        );
        let finals = self.finals.split_inclusive(' ').count();
        let words = || {
            self.finals
                .split_inclusive(' ')
                .chain(self.partial.split_inclusive(' '))
        };
        let count = words().count();
        for (i, word) in words().enumerate() {
            let t = fading(count - 1 - i);
            let color = if t < 0.5 {
                strong.lerp_to_gamma(weak, t * 2.0)
            } else {
                weak.lerp_to_gamma(faint, t * 2.0 - 1.0)
            };
            // The partial text follows the last final text after a space.
            let gap = if i == finals && finals > 0 {
                font.size * 0.3
            } else {
                0.0
            };
            job.append(word, gap, format(color));
        }
        job
    }
}

/// How far a word `age` words before the newest has faded: 0 in full
/// color, 1 at the faintest.
fn fading(age: usize) -> f32 {
    (age.saturating_sub(FRESH_WORDS) as f32 / FADING_WORDS as f32).min(1.0)
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
    fn only_the_newest_words_are_in_full_color() {
        assert_eq!(fading(0), 0.0);
        assert_eq!(fading(FRESH_WORDS - 1), 0.0);
        assert!(fading(FRESH_WORDS + 1) > 0.0 && fading(FRESH_WORDS + 1) < 1.0);
        assert_eq!(fading(FRESH_WORDS + FADING_WORDS), 1.0);
        assert_eq!(fading(1000), 1.0);
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
