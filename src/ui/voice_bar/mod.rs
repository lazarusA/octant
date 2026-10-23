//! The voice bar: drawn in the playback bar's place while open (the top
//! bar's Voice button), with the microphone toggle, its live spectrogram and
//! the listening state. It reads nothing from the app; speech to text and
//! the action layer come later (`VOICE_PLAN.md`).

mod spectrogram;
#[cfg(test)]
mod tests;

use crate::i18n::{PlaybackKey, tr};
use crate::ui::icons::{Icon, ToolbarButton, UiIconExt};
use crate::ui::keyboard::regions::Region;
use crate::ui::toast::{Severity, report};
use crate::ui::toolbar::roving::{BarNav, Stop};
use crate::voice::Capture;
use crate::voice::spectrum::Spectrum;
use spectrogram::Spectrogram;

const BAR_HEIGHT: f32 = 42.0;
/// The spectrogram's widest; it takes at most half of a narrow bar.
const SPECTROGRAM_W: f32 = 400.0;

/// Whether the bar is open and, while it listens, the microphone. Session
/// only: the microphone never opens by itself on launch.
#[derive(Default)]
pub struct VoiceBar {
    open: bool,
    listening: Option<Listening>,
    spectrogram: Spectrogram,
}

struct Listening {
    capture: Capture,
    /// Made once the microphone's sample rate is known.
    spectrum: Option<Spectrum>,
    samples: Vec<f32>,
    /// Exact zeros heard so far, until a sound or the warning ends the check.
    silent: Option<usize>,
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
            self.listening = None;
        } else {
            self.open = true;
            self.listen();
        }
    }

    fn listen(&mut self) {
        match Capture::start() {
            Ok(capture) => {
                self.listening = Some(Listening {
                    capture,
                    spectrum: None,
                    samples: Vec::new(),
                    silent: Some(0),
                });
            }
            Err(e) => report(Severity::Error, tr(PlaybackKey::VoiceMicError), e),
        }
    }

    /// Turns the samples recorded since the last frame into columns.
    fn update(&mut self, ctx: &egui::Context) {
        let Some(l) = &mut self.listening else {
            return;
        };
        l.samples.clear();
        l.capture.drain(&mut l.samples);
        let rate = l.capture.sample_rate();
        if let Some(silent) = &mut l.silent {
            *silent += l.samples.len();
            if l.samples.iter().any(|&s| s != 0.0) {
                l.silent = None;
            } else if rate > 0 && *silent >= rate as usize {
                // A microphone without permission (macOS) records exact zeros.
                report(
                    Severity::Warning,
                    tr(PlaybackKey::VoiceSilent),
                    tr(PlaybackKey::VoiceSilentHint),
                );
                l.silent = None;
            }
        }
        if l.spectrum.is_none() && rate > 0 {
            l.spectrum = Some(Spectrum::new(rate));
        }
        if let Some(spectrum) = &mut l.spectrum {
            let spectrogram = &mut self.spectrogram;
            spectrum.push(&l.samples, |column| spectrogram.push(ctx, column));
        }
        ctx.request_repaint();
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
            voice.listening = None;
        } else {
            voice.listen();
        }
    }
    nav.push(&mic, Stop::Button);

    let height = ui.available_height() - 8.0;
    let width = SPECTROGRAM_W.min(ui.available_width() * 0.5);
    voice.spectrogram.show(ui, egui::vec2(width, height));

    let state = tr(if listening {
        PlaybackKey::VoiceListening
    } else {
        PlaybackKey::VoiceMicOff
    });
    ui.label(egui::RichText::new(state).color(ui.visuals().weak_text_color()));

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let close = ui.close_button(tr(PlaybackKey::VoiceCloseHover));
        if close.clicked() {
            voice.toggle();
        }
        nav.push(&close, Stop::Button);
    });
    nav.finish(ui, Region::BottomBar);
}
