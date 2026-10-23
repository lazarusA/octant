use crate::voice::js_text;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    AudioContext, AudioProcessingEvent, MediaStream, MediaStreamAudioSourceNode,
    MediaStreamConstraints, ScriptProcessorNode,
};

/// Samples per `audioprocess` callback (about 43 ms at 48 kHz).
const BLOCK: u32 = 2048;

pub struct Capture {
    shared: Rc<Shared>,
}

#[derive(Default)]
struct Shared {
    samples: RefCell<Vec<f32>>,
    rate: Cell<u32>,
    stopped: Cell<bool>,
    live: RefCell<Option<Live>>,
}

/// The audio graph while the microphone is open.
struct Live {
    context: AudioContext,
    stream: MediaStream,
    _source: MediaStreamAudioSourceNode,
    processor: ScriptProcessorNode,
    _on_audio: Closure<dyn FnMut(AudioProcessingEvent)>,
}

impl Capture {
    /// Asks the browser for the microphone; recording starts once the user
    /// allows it, and a refusal is reported as a toast.
    pub fn start() -> Result<Self, String> {
        let devices = web_sys::window()
            .ok_or_else(|| "no window".to_string())?
            .navigator()
            .media_devices()
            .map_err(js_text)?;
        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&JsValue::TRUE);
        let promise = devices
            .get_user_media_with_constraints(&constraints)
            .map_err(js_text)?;
        let shared = Rc::new(Shared::default());
        let pending = Rc::clone(&shared);
        wasm_bindgen_futures::spawn_local(async move {
            let opened = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .and_then(|stream| connect(&pending, stream.unchecked_into()));
            if let Err(e) = opened {
                crate::ui::toast::report(
                    crate::ui::toast::Severity::Error,
                    crate::i18n::tr(crate::i18n::PlaybackKey::VoiceMicError),
                    js_text(e),
                );
            }
        });
        Ok(Self { shared })
    }

    /// Samples per second; 0 until the microphone opens.
    pub fn sample_rate(&self) -> u32 {
        self.shared.rate.get()
    }

    /// Moves the samples recorded since the last call into `out`.
    pub fn drain(&self, out: &mut Vec<f32>) {
        if let Ok(mut pending) = self.shared.samples.try_borrow_mut() {
            out.append(&mut pending);
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.shared.stopped.set(true);
        if let Some(live) = self.shared.live.take() {
            live.processor.set_onaudioprocess(None);
            stop_tracks(&live.stream);
            let _ = live.context.close();
        }
    }
}

/// Routes `stream` through a script processor that copies its samples,
/// unless the capture was dropped while the browser asked.
// shortcut: ScriptProcessorNode is deprecated but runs everywhere without a
// worklet module; move to an AudioWorklet when speech to text needs it.
fn connect(shared: &Rc<Shared>, stream: MediaStream) -> Result<(), JsValue> {
    if shared.stopped.get() {
        stop_tracks(&stream);
        return Ok(());
    }
    let context = AudioContext::new()?;
    let source = context.create_media_stream_source(&stream)?;
    let processor = context
        .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
            BLOCK, 1, 1,
        )?;
    let sink: Weak<Shared> = Rc::downgrade(shared);
    let on_audio =
        Closure::<dyn FnMut(AudioProcessingEvent)>::new(move |e: AudioProcessingEvent| {
            let Some(shared) = sink.upgrade() else { return };
            let (Ok(buffer), Ok(mut pending)) = (e.input_buffer(), shared.samples.try_borrow_mut())
            else {
                return;
            };
            if let Ok(data) = buffer.get_channel_data(0) {
                super::keep_recent(&mut pending, data);
            }
        });
    processor.set_onaudioprocess(Some(on_audio.as_ref().unchecked_ref()));
    source.connect_with_audio_node(&processor)?;
    // The processor only runs while connected; its output stays silent.
    processor.connect_with_audio_node(&context.destination())?;
    shared.rate.set(context.sample_rate() as u32);
    *shared.live.borrow_mut() = Some(Live {
        context,
        stream,
        _source: source,
        processor,
        _on_audio: on_audio,
    });
    Ok(())
}

fn stop_tracks(stream: &MediaStream) {
    for track in stream.get_tracks().iter() {
        if let Ok(track) = track.dyn_into::<web_sys::MediaStreamTrack>() {
            track.stop();
        }
    }
}
