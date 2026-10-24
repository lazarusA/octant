//! The worker's state and messages: what arrives before the model is ready
//! is kept as a [`Tail`] (its last seconds and stops); afterwards it goes to
//! a [`Session`],
//! whose partial text runs once the messages waiting have been taken (a
//! zero-delay timer), so a slow model skips stale partials.

use js_sys::Float32Array;
use octant::voice::stt::{Input, Session, SttEvent, Tail, WhisperSize, wire};
use octant::voice::whisper::{Whisper, models};
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

#[derive(Default)]
struct State {
    session: Option<Session<Whisper>>,
    /// What arrives while the model loads.
    tail: Tail,
    partial_scheduled: bool,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

fn scope() -> DedicatedWorkerGlobalScope {
    js_sys::global().unchecked_into()
}

fn emit(event: SttEvent) {
    let _ = scope().post_message(&JsValue::from_str(&wire::encode(&event)));
}

pub fn start() {
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(|e: MessageEvent| receive(&e.data()));
    scope().set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    // The worker lives as long as the page keeps it.
    on_message.forget();
    let _ = scope().post_message(&JsValue::from_str(wire::HELLO));
}

fn receive(data: &JsValue) {
    if let Some(command) = data.as_string() {
        if let Some(size) = wire::loaded(&command) {
            wasm_bindgen_futures::spawn_local(load(size));
        } else if command == wire::FINISH {
            take(Input::Finish);
        } else if let Some(prompt) = wire::prompted(&command) {
            take(Input::Prompt(prompt.to_string()));
        }
    } else if let Some(samples) = data.dyn_ref::<Float32Array>() {
        take(Input::Audio(samples.to_vec()));
    }
}

fn take(input: Input) {
    STATE.with_borrow_mut(|state| match (&mut state.session, input) {
        (Some(session), input) => session.take(input, &mut emit),
        (None, input) => state.tail.push(input),
    });
    schedule_partial();
}

/// Makes the partial text due once the messages waiting have run.
fn schedule_partial() {
    let first = STATE.with_borrow_mut(|s| !std::mem::replace(&mut s.partial_scheduled, true));
    if !first {
        return;
    }
    let run = Closure::once_into_js(|| {
        STATE.with_borrow_mut(|state| {
            state.partial_scheduled = false;
            if let Some(session) = &mut state.session {
                session.partial(&mut emit);
            }
        });
    });
    let _ = scope().set_timeout_with_callback_and_timeout_and_arguments_0(run.unchecked_ref(), 0);
}

async fn load(size: WhisperSize) {
    if let Err(e) = try_load(size).await {
        emit(SttEvent::Failed(e));
    }
}

async fn try_load(size: WhisperSize) -> Result<(), String> {
    emit(SttEvent::Preparing(Some(0.0)));
    let [config, tokenizer, weights] = models::files(size);
    let config = crate::files::fetch(size, config, &|_| {}).await?;
    let tokenizer = crate::files::fetch(size, tokenizer, &|_| {}).await?;
    let progress = |done| emit(SttEvent::Preparing(Some(done)));
    let weights = crate::files::fetch(size, weights, &progress).await?;
    emit(SttEvent::Preparing(None));
    let whisper = Whisper::from_bytes(&config, &tokenizer, weights)?;
    let tail: Vec<Input> = STATE.with_borrow_mut(|state| {
        state.session = Some(Session::finals_first(whisper));
        state.tail.take().collect()
    });
    emit(SttEvent::Ready);
    for input in tail {
        take(input);
    }
    Ok(())
}
