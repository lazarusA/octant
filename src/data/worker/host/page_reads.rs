//! Reads the worker asks of the page: local files live only on the page (dropped folders
//! cannot be sent to a worker), so their bytes, sizes or `File` come back by message.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use futures::channel::oneshot;
use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use super::super::js;
use super::super::wire::{FromWorker, LocalRead};
use super::scope;

type ReadAnswer = Result<Vec<u8>, String>;
/// A page's answer to a `Read`: its first part, bytes or a `File`.
type PartAnswer = Result<JsValue, String>;

thread_local! {
    static READS: RefCell<HashMap<u64, oneshot::Sender<PartAnswer>>> = RefCell::default();
    static NEXT_READ: Cell<u64> = const { Cell::new(0) };
}

/// The page answered the read `id`: its first part, unless `error`.
pub(super) fn answered(id: u64, error: Option<String>, parts: &Array) {
    let answer = match error {
        Some(error) => Err(error),
        None => Ok(parts.get(0)),
    };
    if let Some(tx) = READS.with_borrow_mut(|r| r.remove(&id)) {
        let _ = tx.send(answer);
    }
}

/// Asks the page for `read` of the local file at `url`.
pub async fn read_from_page(url: &str, read: LocalRead) -> ReadAnswer {
    let part = ask_page(url, read).await?;
    let bytes = part.dyn_into::<js_sys::Uint8Array>();
    bytes
        .map(|b| b.to_vec())
        .map_err(|_| "A read without its bytes".to_owned())
}

/// The local file at `url` itself, for synchronous reads (`netcdf/wasm.rs`).
pub async fn file_from_page(url: &str) -> Result<web_sys::File, String> {
    let part = ask_page(url, LocalRead::File).await?;
    part.dyn_into()
        .map_err(|_| format!("'{url}' is not one file"))
}

async fn ask_page(url: &str, read: LocalRead) -> PartAnswer {
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
