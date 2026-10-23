//! [`WorkerStt`]: Whisper in the browser's voice worker
//! (`src/bin/voice_worker/`), started when the provider opens and ended
//! when it drops, which also stops a download under way. Audio goes to the
//! worker as transferred `Float32Array`s, commands and events as text
//! ([`super::wire`]); the model is asked for once the worker says hello.
//! Until then the worker has no handler and would drop what it is sent, so
//! audio and stops wait in a [`Tail`].

use super::{Input, SpeechToText, SttEvent, Tail, WhisperSize, wire};
use js_sys::{Array, Float32Array};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, Worker};

/// Trunk's loader for the worker, next to the page.
const SCRIPT: &str = "voice_worker_loader.js";

pub struct WorkerStt {
    worker: Worker,
    events: Rc<RefCell<VecDeque<SttEvent>>>,
    /// What is sent before the worker says hello; `None` after.
    early: Rc<RefCell<Option<Tail>>>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_error: Closure<dyn FnMut(ErrorEvent)>,
}

impl WorkerStt {
    /// Starts the worker and has it load `size` once it says it listens.
    pub fn start(size: WhisperSize) -> Result<Self, String> {
        let worker = Worker::new(SCRIPT).map_err(|e| format!("{e:?}"))?;
        let events = Rc::new(RefCell::new(VecDeque::new()));
        let inbox = Rc::clone(&events);
        let early = Rc::new(RefCell::new(Some(Tail::default())));
        let held = Rc::clone(&early);
        let to_worker = worker.clone();
        let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            let Some(message) = e.data().as_string() else {
                return;
            };
            if message == wire::HELLO {
                let _ = to_worker.post_message(&JsValue::from_str(&wire::load(size)));
                let early = held.try_borrow_mut().ok().and_then(|mut t| t.take());
                if let Some(mut tail) = early {
                    for input in tail.take() {
                        post(&to_worker, &input);
                    }
                }
            } else if let (Some(event), Ok(mut queue)) =
                (wire::decode(&message), inbox.try_borrow_mut())
            {
                queue.push_back(event);
            }
        });
        let inbox = Rc::clone(&events);
        let on_error = Closure::<dyn FnMut(ErrorEvent)>::new(move |e: ErrorEvent| {
            if let Ok(mut queue) = inbox.try_borrow_mut() {
                // A worker error ends the worker: reported as a failure while
                // getting ready, so the voice bar drops it and starts afresh.
                queue.push_back(SttEvent::Preparing(None));
                queue.push_back(SttEvent::Failed(e.message()));
            }
        });
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        worker.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        Ok(Self {
            worker,
            events,
            early,
            _on_message: on_message,
            _on_error: on_error,
        })
    }
}

impl WorkerStt {
    fn send(&mut self, input: Input) {
        match self.early.try_borrow_mut().as_deref_mut() {
            Ok(Some(tail)) => tail.push(input),
            _ => post(&self.worker, &input),
        }
    }
}

/// Sends `input` to the worker, audio as a transferred `Float32Array`.
fn post(worker: &Worker, input: &Input) {
    let _ = match input {
        Input::Audio(samples) => {
            let audio = Float32Array::from(samples.as_slice());
            worker.post_message_with_transfer(&audio, &Array::of1(&audio.buffer()))
        }
        Input::Finish => worker.post_message(&JsValue::from_str(wire::FINISH)),
    };
}

impl SpeechToText for WorkerStt {
    fn push(&mut self, samples: &[f32]) {
        if !samples.is_empty() {
            self.send(Input::Audio(samples.to_vec()));
        }
    }

    fn finish(&mut self) {
        self.send(Input::Finish);
    }

    fn poll(&mut self) -> Option<SttEvent> {
        self.events.try_borrow_mut().ok()?.pop_front()
    }
}

impl Drop for WorkerStt {
    fn drop(&mut self) {
        self.worker.terminate();
    }
}
