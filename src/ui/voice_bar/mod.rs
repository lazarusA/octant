//! The voice bar: drawn in the playback bar's place while open (the top
//! bar's Voice button), with the microphone toggle, a disk pulsing with its
//! frequencies and the live transcript (or the listening state) from the
//! chosen speech to text provider (`voice::stt`), and a field to type
//! commands in. It reads nothing from the app: each final utterance, said
//! or typed, is handed out (`take_utterances`) for the app to run as
//! commands (`app::actions`), what the last one did is shown beside the
//! transcript (`set_last_action`), and the app gives it the names the
//! speech model is prompted with (`follow_prompt`).

mod commands;
mod listening;
mod model_menu;
mod pulsar;
#[cfg(test)]
mod tests;
mod transcript;

use crate::i18n::{PlaybackKey, tr};
use crate::ui::icons::{Icon, ToolbarButton, UiIconExt};
use crate::ui::keyboard::regions::Region;
use crate::ui::toast::{Severity, report};
use crate::ui::toolbar::roving::{BarNav, Stop};
use crate::voice::Capture;
use crate::voice::resample::Resampler;
use crate::voice::spectrum::Spectrum;
use crate::voice::stt::{self, Provider, SpeechToText};
pub use commands::field_id;
use listening::Listening;
use model_menu::model_menu;
use pulsar::Pulsar;
use transcript::Transcript;

const BAR_HEIGHT: f32 = 42.0;
/// Seconds the provider is still polled after the microphone stops (or the
/// model it waited for gets ready), for the last final text.
// shortcut: a fixed wait; a provider saying it has nothing left would end it
// sooner, worth it if the idle repaints show up.
const FINAL_WAIT: f64 = 30.0;
/// How often the provider is polled while nothing else repaints.
const POLL_EVERY: std::time::Duration = std::time::Duration::from_millis(100);

/// Whether the bar is open and, while it listens, the microphone. Session
/// only: the microphone never opens by itself on launch.
#[derive(Default)]
pub struct VoiceBar {
    open: bool,
    listening: Option<Listening>,
    pulsar: Pulsar,
    /// Kept after the microphone stops, until the bar closes, so the last
    /// utterance's final text still arrives.
    stt: Option<Box<dyn SpeechToText>>,
    provider: Provider,
    transcript: Transcript,
    /// Until when (egui time) the provider is polled for a last final text.
    final_until: f64,
    /// Utterances (said or typed) not yet run as commands.
    utterances: Vec<String>,
    /// The typed command field's text.
    typed: String,
    /// Names given to the speech model, and the metadata generation they
    /// were listed for.
    prompt: String,
    prompt_key: Option<u64>,
}

impl VoiceBar {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_listening(&self) -> bool {
        self.listening.is_some()
    }

    /// Opens the bar listening, or closes it and releases the microphone.
    pub fn toggle(&mut self) {
        if self.open {
            self.open = false;
            self.release();
        } else {
            self.open = true;
            self.listen();
        }
    }

    /// Stops the microphone at `now`; the utterance under way still ends
    /// with its final text.
    fn stop(&mut self, now: f64) {
        if self.listening.take().is_some()
            && let Some(stt) = &mut self.stt
        {
            stt.finish();
            self.final_until = now + FINAL_WAIT;
        }
        self.pulsar.reset();
    }

    /// Stops the microphone and drops the provider.
    fn release(&mut self) {
        self.listening = None;
        self.set_provider(None);
        self.pulsar.reset();
    }

    /// Replaces the provider; what the old one was getting ready is forgotten.
    fn set_provider(&mut self, stt: Option<Box<dyn SpeechToText>>) {
        self.stt = stt;
        self.transcript.provider_dropped();
        self.give_prompt();
    }

    fn listen(&mut self) {
        match Capture::start() {
            Ok(capture) => {
                self.listening = Some(Listening::new(capture));
                if self.stt.is_none() {
                    self.set_provider(stt::open(self.provider));
                }
                self.transcript.clear();
            }
            Err(e) => report(Severity::Error, tr(PlaybackKey::VoiceMicError), e),
        }
    }

    /// Feeds the samples recorded since the last frame to the disk and the
    /// provider, and takes the provider's text.
    fn update(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if let Some(l) = &self.listening
            && l.capture.failed()
        {
            if l.capture.sample_rate() > 0 {
                // A microphone that stopped after it opened.
                self.stop(now);
            } else {
                // Refused (the web's prompt): no model downloads for audio
                // that never comes.
                self.release();
            }
        }
        self.poll(ctx, now);
        let Some(l) = &mut self.listening else {
            return;
        };
        l.samples.clear();
        l.capture.drain(&mut l.samples);
        let rate = l.capture.sample_rate();
        l.check_silence(rate);
        if l.spectrum.is_none() && rate > 0 {
            l.spectrum = Some(Spectrum::new(rate));
        }
        if let Some(spectrum) = &mut l.spectrum {
            spectrum.push(&l.samples, |column| self.pulsar.push(column));
        }
        if let Some(stt) = &mut self.stt
            && rate > 0
        {
            let resampler = l.resampler.get_or_insert_with(|| Resampler::new(rate));
            l.resampled.clear();
            resampler.push(&l.samples, &mut l.resampled);
            stt.push(&l.resampled);
        }
        ctx.request_repaint();
    }

    /// Takes the provider's events, polling again soon while it gets ready
    /// or a final text may still come.
    fn poll(&mut self, ctx: &egui::Context, now: f64) {
        let Some(stt) = &mut self.stt else {
            return;
        };
        let was_preparing = self.transcript.is_preparing();
        let mut gave_up = false;
        while let Some(event) = stt.poll() {
            if let stt::SttEvent::Final(text) = &event {
                self.utterances.push(text.clone());
            }
            gave_up |= self.transcript.apply(event);
        }
        if gave_up {
            self.set_provider(None);
            return;
        }
        if was_preparing && !self.transcript.is_preparing() {
            // The audio held while it loaded is transcribed now.
            self.final_until = now + FINAL_WAIT;
        }
        if self.transcript.is_preparing() || now < self.final_until {
            ctx.request_repaint_after(POLL_EVERY);
        }
    }
}

pub fn show_voice_bar(voice: &mut VoiceBar, ui: &mut egui::Ui) {
    egui::Panel::bottom("octant_voice_bar")
        .exact_size(BAR_HEIGHT)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| show_contents(voice, ui));
        });
}

fn show_contents(voice: &mut VoiceBar, ui: &mut egui::Ui) {
    voice.update(ui.ctx());
    let mut nav = BarNav::default();

    let listening = voice.is_listening();
    let hover = tr(if listening {
        PlaybackKey::VoiceStop
    } else {
        PlaybackKey::VoiceListen
    });
    let mic = ui.add(
        ToolbarButton::new(Icon::Microphone, hover)
            .compact(true)
            .toggled(listening),
    );
    if mic.clicked() {
        if listening {
            voice.stop(ui.input(|i| i.time));
        } else {
            voice.listen();
        }
    }
    nav.push(&mic, Stop::Button);

    voice
        .pulsar
        .show(ui, ui.available_height() - 4.0, listening);
    voice.command_field(ui);

    let state = tr(if listening {
        PlaybackKey::VoiceListening
    } else {
        PlaybackKey::VoiceMicOff
    });
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let close = ui.close_button(tr(PlaybackKey::VoiceCloseHover));
        if close.clicked() {
            voice.toggle();
        }
        nav.push(&close, Stop::Button);
        nav.push(&model_menu(voice, ui), Stop::Button);
        voice.transcript.show_action(ui);
        let rect = ui.available_rect_before_wrap();
        voice.transcript.show(ui, rect, state);
    });
    nav.finish(ui, Region::BottomBar);
}
