//! The worker's side: loads blocks as the page would (`BlockLoader::load_one_wasm`),
//! from stores it opens itself, once each, by inspecting them as the page
//! does (`backends::inspect_web`), and keeps until their dataset closes. Every `REPORT_MS` it sends the
//! page what its fetches counted and the toasts reported here. Local files
//! are read through the page ([`read_from_page`]).

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use futures::channel::oneshot;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

mod page_reads;

pub use page_reads::{file_from_page, read_from_page};

use super::js::{self, Part};
use super::lifetimes::Lifetimes;
use super::wire::{BlockHeader, FromWorker, MetadataHeader, ToWorker};
use crate::data::blocks::memory_budget::MemoryBudget;
use crate::data::blocks::{BlockLoader, BlockRequest, progress};
use crate::data::slice_request::SliceRequest;
use crate::data::{DataSource, SourceFactory, StoreHandle};

/// How often the worker reports its counts and toasts.
const REPORT_MS: i32 = 100;

type Opened = Result<StoreHandle, String>;

thread_local! {
    static STORES: RefCell<HashMap<DataSource, StoreHandle>> = RefCell::default();
    /// Stores being opened, with the loads waiting for them.
    static OPENING: RefCell<HashMap<DataSource, Vec<oneshot::Sender<Opened>>>> =
        RefCell::default();
    /// When stores are let go.
    static LIFETIMES: RefCell<Lifetimes> = RefCell::default();
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
            mut source,
            slice,
            language,
        } => {
            crate::i18n::set_language(language);
            // URLs compared without a trailing slash, as the stores' registries keep them.
            source.uri.truncate(source.uri.trim_end_matches('/').len());
            LIFETIMES.with_borrow_mut(|l| l.begin(&source.uri));
            wasm_bindgen_futures::spawn_local(load(id, source, slice));
        }
        ToWorker::Inspect {
            id,
            mut source,
            language,
        } => {
            crate::i18n::set_language(language);
            source.uri.truncate(source.uri.trim_end_matches('/').len());
            LIFETIMES.with_borrow_mut(|l| l.begin(&source.uri));
            wasm_bindgen_futures::spawn_local(inspect(id, source));
        }
        ToWorker::Release { uri } => {
            let uri = uri.trim_end_matches('/');
            STORES.with_borrow_mut(|s| s.retain(|source, _| source.uri != uri));
            if LIFETIMES.with_borrow_mut(|l| l.release(uri)) {
                release(uri);
            }
        }
        ToWorker::ReadDone { id, error } => page_reads::answered(id, error, &parts),
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
    if LIFETIMES.with_borrow_mut(|l| l.end(&source.uri)) {
        release(&source.uri);
    }
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

/// Inspects `source`, keeping its store open for the loads that follow.
async fn inspect(id: u64, source: DataSource) {
    let result = match open(&source).await {
        Ok(store) => store.inspect().map_err(|e| e.to_string()),
        Err(e) => Err(e),
    };
    if LIFETIMES.with_borrow_mut(|l| l.end(&source.uri)) {
        release(&source.uri);
    }
    report();
    match result {
        Ok(metadata) => {
            let (header, numbers) = MetadataHeader::split(metadata);
            let header = Box::new(header);
            let message = FromWorker::Inspected { id, header };
            if !js::post(&scope(), &message, &[Part::F64(&numbers)]) {
                let error = "The inspection could not be sent".to_owned();
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
    // NetCDF files are opened here, never on the page, which keeps `hdf5-pure` out of the
    // app's `.wasm`; their store inspects them.
    let inspected = match source.kind {
        crate::data::DataSourceKind::NetCdf => crate::data::backends::netcdf::wasm::open(source)
            .await
            .err()
            .map(Err),
        _ => crate::data::backends::inspect_web(source).await,
    };
    let opened = match inspected {
        Some(Err(e)) => Err(e),
        _ => SourceFactory::open(source.clone()).map_err(|e| e.to_string()),
    };
    // A store released while it opened is not kept: the last load lets it go.
    let released = LIFETIMES.with_borrow(|l| l.released(&source.uri));
    if let (Ok(store), false) = (&opened, released) {
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
