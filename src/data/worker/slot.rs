//! One data worker as the pool holds it: started, listened to, timed until
//! its hello, and what is sent to it before that.

use std::collections::HashSet;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Event, MessageEvent, Worker};

use super::js::{self, Part};
use super::pool::POOL;
use super::wire::{FromWorker, LocalRead, ToWorker};

/// Trunk's loader for the worker, next to the page.
const SCRIPT: &str = "data_worker_loader.js";
/// How long a worker may take to say hello before the page loads alone.
const START_TIMEOUT_MS: i32 = 20_000;

pub(super) struct Slot {
    pub(super) worker: Worker,
    /// Tells this start apart from a later one in the same place.
    pub(super) serial: u64,
    pub(super) ready: bool,
    pub(super) dead: bool,
    /// What is sent before hello.
    pub(super) early: Vec<ToWorker>,
    pub(super) jobs: HashSet<u64>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_error: Closure<dyn FnMut(Event)>,
    /// The start timer, cleared on hello or drop so it never calls its
    /// dropped closure.
    pub(super) timer: i32,
    _timeout: Closure<dyn FnMut()>,
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.worker.terminate();
        clear_timer(self.timer);
    }
}

pub(super) fn clear_timer(handle: i32) {
    if let Some(window) = web_sys::window() {
        window.clear_timeout_with_handle(handle);
    }
}

impl Slot {
    pub(super) fn start(index: usize, serial: u64) -> Result<Self, String> {
        let worker = Worker::new(SCRIPT).map_err(|e| format!("{e:?}"))?;
        let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            if let Some((message, parts)) = js::receive::<FromWorker>(&e.data()) {
                POOL.with_borrow_mut(|pool| pool.receive(index, message, &parts));
            }
        });
        // A script that fails to load may raise a plain `Event`, without a message.
        let on_error = Closure::<dyn FnMut(Event)>::new(move |e: Event| {
            let message = js_sys::Reflect::get(&e, &"message".into()).ok();
            let why = message.and_then(|m| m.as_string()).unwrap_or_default();
            POOL.with_borrow_mut(|pool| pool.stopped(index, serial, &why));
        });
        let timeout = Closure::<dyn FnMut()>::new(move || {
            POOL.with_borrow_mut(|pool| {
                if matches!(&pool.slots[index], Some(s) if s.serial == serial && !s.ready) {
                    pool.stopped(index, serial, "no answer");
                }
            });
        });
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        worker.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        let window = web_sys::window().ok_or("No window")?;
        let timer = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout.as_ref().unchecked_ref(),
                START_TIMEOUT_MS,
            )
            .map_err(|e| format!("{e:?}"))?;
        Ok(Self {
            worker,
            serial,
            ready: false,
            dead: false,
            early: Vec::new(),
            jobs: HashSet::new(),
            _on_message: on_message,
            _on_error: on_error,
            timer,
            _timeout: timeout,
        })
    }

    pub(super) fn send(&mut self, message: ToWorker) {
        if self.dead {
            return;
        }
        if self.ready {
            js::post(&self.worker, &message, &[]);
        } else {
            self.early.push(message);
        }
    }
}

/// Reads what a worker asked of a local file, and answers it.
pub(super) async fn answer_read(worker: Worker, id: u64, url: String, read: LocalRead) {
    use crate::data::backends::local;
    let result = match read {
        LocalRead::All => local::read_all(&url).await,
        LocalRead::Range { offset, length } => local::read_range(&url, offset, length).await,
        LocalRead::Size => local::size(&url).await.map(|n| n.to_le_bytes().to_vec()),
    };
    let error = match result {
        Ok(bytes) => {
            let done = ToWorker::ReadDone { id, error: None };
            if js::post(&worker, &done, &[Part::Bytes(&bytes)]) {
                return;
            }
            format!("Failed to hand '{url}' to the data worker")
        }
        Err(error) => error,
    };
    let error = Some(error);
    js::post(&worker, &ToWorker::ReadDone { id, error }, &[]);
}
