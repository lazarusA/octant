//! The worker's side: loads blocks as the page would (`BlockLoader::load_one_wasm`),
//! from stores it opens itself, once each, by inspecting them as the page
//! does (`backends::inspect_web`), and keeps until their dataset closes. Every `REPORT_MS` it sends the
//! page what its fetches counted and the toasts reported here. Local files
//! are read through the page ([`read_from_page`]).

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use futures::channel::oneshot;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

use super::js::{self, Part};
use super::wire::{BlockHeader, FromWorker, LocalRead, ToWorker};
use crate::data::blocks::memory_budget::MemoryBudget;
use crate::data::blocks::{BlockLoader, BlockRequest, progress};
use crate::data::slice_request::SliceRequest;
use crate::data::{DataSource, SourceFactory, StoreHandle};

/// How often the worker reports its counts and toasts.
const REPORT_MS: i32 = 100;

type Opened = Result<StoreHandle, String>;
type ReadAnswer = Result<Vec<u8>, String>;

thread_local! {
    static STORES: RefCell<HashMap<DataSource, StoreHandle>> = RefCell::default();
    /// Stores being opened, with the loads waiting for them.
    static OPENING: RefCell<HashMap<DataSource, Vec<oneshot::Sender<Opened>>>> =
        RefCell::default();
    /// Stores released while they opened: let go again once open.
    static RELEASED: RefCell<HashSet<String>> = RefCell::default();
    static READS: RefCell<HashMap<u64, oneshot::Sender<ReadAnswer>>> = RefCell::default();
    static NEXT_READ: Cell<u64> = const { Cell::new(0) };
    /// The progress counts last sent.
    static REPORTED: Cell<[u64; 3]> = const { Cell::new([0; 3]) };
}

fn scope() -> DedicatedWorkerGlobalScope {
    js_sys::global().unchecked_into()
}

/// Listens to the page, reports every `REPORT_MS`, and says hello.
pub fn start() {
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(|e: MessageEvent| receive(&e.data()));
    scope().set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    // The worker lives as long as the page keeps it.
    on_message.forget();
    let tick = Closure::<dyn FnMut()>::new(report);
    let ticking = scope().set_interval_with_callback_and_timeout_and_arguments_0(
        tick.as_ref().unchecked_ref(),
        REPORT_MS,
    );
    if let Err(e) = ticking {
        log::error!("[data worker] No report timer: {e:?}");
    }
    tick.forget();
    js::post(&scope(), &FromWorker::Hello, &[]);
}

fn receive(data: &JsValue) {
    let Some((message, parts)) = js::receive::<ToWorker>(data) else {
        return;
    };
    match message {
        ToWorker::Budget { bytes } => MemoryBudget::set_shared_limit(bytes),
        ToWorker::Load {
            id,
            source,
            slice,
            language,
        } => {
            crate::i18n::set_language(language);
            wasm_bindgen_futures::spawn_local(load(id, source, slice));
        }
        ToWorker::Release { uri } => {
            let uri = uri.trim_end_matches('/');
            let opening = OPENING.with_borrow(|o| o.keys().any(|s| same_uri(s, uri)));
            if opening {
                RELEASED.with_borrow_mut(|r| r.insert(uri.to_owned()));
            }
            STORES.with_borrow_mut(|s| s.retain(|source, _| !same_uri(source, uri)));
            release(uri);
        }
        ToWorker::ReadDone { id, error } => {
            let answer = match error {
                Some(error) => Err(error),
                None => js::bytes(&parts, 0).ok_or_else(|| "A read without its bytes".to_owned()),
            };
            if let Some(tx) = READS.with_borrow_mut(|r| r.remove(&id)) {
                let _ = tx.send(answer);
            }
        }
    }
}

async fn load(id: u64, source: DataSource, slice: SliceRequest) {
    let result = match open(&source).await {
        Ok(store) => {
            let request = BlockRequest::new(store, slice);
            let mut counted = |bytes| progress::bytes(bytes);
            BlockLoader::load_one_wasm(&request, Some(&mut counted))
                .await
                .map_err(|e| e.to_string())
        }
        Err(e) => Err(e),
    };
    // Counts and toasts reach the page before the block they came with.
    report();
    match result {
        Ok(block) => {
            let (header, values, numbers) = BlockHeader::split(block);
            let parts = [Part::F32(&values), Part::F64(&numbers)];
            if !js::post(&scope(), &FromWorker::Block { id, header }, &parts) {
                let error = "The loaded block could not be sent".to_owned();
                js::post(&scope(), &FromWorker::Failed { id, error }, &[]);
            }
        }
        Err(error) => {
            js::post(&scope(), &FromWorker::Failed { id, error }, &[]);
        }
    }
}

/// The store of `source`, opened once: loads asking while it opens wait for it.
async fn open(source: &DataSource) -> Opened {
    if let Some(store) = STORES.with_borrow(|s| s.get(source).cloned()) {
        return Ok(store);
    }
    let joined = OPENING.with_borrow_mut(|o| {
        o.get_mut(source).map(|waiting| {
            let (tx, rx) = oneshot::channel();
            waiting.push(tx);
            rx
        })
    });
    if let Some(rx) = joined {
        return rx
            .await
            .unwrap_or_else(|_| Err("The store did not open".to_owned()));
    }
    OPENING.with_borrow_mut(|o| o.insert(source.clone(), Vec::new()));
    let inspected = crate::data::backends::inspect_web(source).await;
    let opened = match inspected {
        Some(Err(e)) => Err(e),
        _ => SourceFactory::open(source.clone()).map_err(|e| e.to_string()),
    };
    let uri = source.uri.trim_end_matches('/');
    if RELEASED.with_borrow_mut(|r| r.remove(uri)) {
        // Its dataset closed meanwhile: this load still runs, nothing is kept.
        release(uri);
    } else if let Ok(store) = &opened {
        STORES.with_borrow_mut(|s| s.insert(source.clone(), store.clone()));
    }
    let waiting = OPENING
        .with_borrow_mut(|o| o.remove(source))
        .unwrap_or_default();
    for tx in waiting {
        let _ = tx.send(opened.clone());
    }
    opened
}

fn same_uri(source: &DataSource, uri: &str) -> bool {
    source.uri.trim_end_matches('/') == uri
}

/// Lets go of what the worker keeps for `uri`: its stores and coordinates.
fn release(uri: &str) {
    crate::data::backends::release_stores(uri);
    crate::data::backends::coord_bounds::evict_coord_values(Some(uri));
}

/// Sends the page what was counted since the last report, and the toasts.
fn report() {
    let totals = progress::totals();
    let last = REPORTED.replace(totals);
    let counts = [0, 1, 2].map(|i| totals[i].saturating_sub(last[i]));
    if counts != [0; 3] {
        js::post(&scope(), &FromWorker::Progress { counts }, &[]);
    }
    for notice in crate::ui::toast::drain_reports() {
        let message = FromWorker::Notice {
            severity: notice.severity,
            title: notice.title,
            detail: notice.detail,
        };
        js::post(&scope(), &message, &[]);
    }
}

/// Asks the page for `read` of the local file at `url`.
pub async fn read_from_page(url: &str, read: LocalRead) -> ReadAnswer {
    let id = NEXT_READ.replace(NEXT_READ.get() + 1);
    let (tx, rx) = oneshot::channel();
    READS.with_borrow_mut(|r| r.insert(id, tx));
    let message = FromWorker::Read {
        id,
        url: url.to_owned(),
        read,
    };
    js::post(&scope(), &message, &[]);
    rx.await
        .unwrap_or_else(|_| Err(format!("No answer reading '{url}'")))
}
