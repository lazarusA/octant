//! The browser's local sources by id, and the ones opened since the app
//! last looked (`take_opened`).

use super::path;
use crate::data::DataSourceKind;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;

/// What a local source reads its files from.
#[derive(Clone)]
pub(super) enum Source {
    /// One file: its URL names it, with no key under it.
    File(web_sys::File),
    /// A folder listed up front (`webkitdirectory`): files by path.
    Listed(Rc<HashMap<String, web_sys::File>>),
    /// A `FileSystemDirectoryHandle` (`showDirectoryPicker`), looked up per key.
    Handle(JsValue),
    /// A dropped `FileSystemDirectoryEntry`, looked up per key.
    Entry(JsValue),
}

struct Registered {
    name: String,
    source: Source,
    kind: DataSourceKind,
}

/// A source opened (its URL and store kind) or why it could not be.
pub type Opened = Result<(String, DataSourceKind), String>;

thread_local! {
    static SOURCES: RefCell<HashMap<u32, Registered>> = RefCell::new(HashMap::new());
    static NEXT_ID: Cell<u32> = const { Cell::new(0) };
    static OPENED: RefCell<Vec<Opened>> = const { RefCell::new(Vec::new()) };
}

/// Registers `source`, named `name`, as a store of `kind`; its URL.
fn register(name: &str, source: Source, kind: DataSourceKind) -> String {
    let id = NEXT_ID.with(|n| n.replace(n.get() + 1));
    let registered = Registered {
        name: name.to_owned(),
        source,
        kind,
    };
    SOURCES.with(|s| s.borrow_mut().insert(id, registered));
    path::url_of(id, name)
}

/// The source of local URL `url`.
pub(super) fn source_of(url: &str) -> Option<(Source, &str)> {
    let (id, key) = path::parse(url)?;
    let source = SOURCES.with(|s| s.borrow().get(&id).map(|r| r.source.clone()))?;
    Some((source, key))
}

/// The store kind of the local source at `url`.
pub fn kind_of(url: &str) -> Option<DataSourceKind> {
    let (id, _) = path::parse(url)?;
    SOURCES.with(|s| s.borrow().get(&id).map(|r| r.kind.clone()))
}

/// Forgets the local source at `url` (its dataset closed).
pub fn release(url: &str) {
    if let Some((id, _)) = path::parse(url) {
        SOURCES.with(|s| s.borrow_mut().remove(&id));
    }
}

/// Decides what `source` holds, registers it (or finds it registered) and
/// queues it for the app (`take_opened`), repainting `ctx` so the app looks.
pub(super) async fn open(name: String, source: Source, ctx: egui::Context) {
    let opened = match registered(&name, &source).await {
        Some(found) => Ok(found),
        None => match super::read::kind_in(&source, &name).await {
            Some(kind) => Ok((register(&name, source, kind.clone()), kind)),
            None => Err(name),
        },
    };
    OPENED.with(|o| o.borrow_mut().push(opened));
    ctx.request_repaint();
}

/// The URL and kind of `source`, named `name`, when it is registered
/// already, so opening it again activates its dataset. Only sources the
/// browser can tell apart: picked folders (`isSameEntry`) and files (size
/// and modification time); a listed or dropped folder of the same name may
/// be another one.
async fn registered(name: &str, source: &Source) -> Option<(String, DataSourceKind)> {
    let same_name: Vec<(u32, Source, DataSourceKind)> = SOURCES.with(|s| {
        let sources = s.borrow();
        let found = sources.iter().filter(|(_, r)| r.name == name);
        found
            .map(|(id, r)| (*id, r.source.clone(), r.kind.clone()))
            .collect()
    });
    for (id, other, kind) in same_name {
        if same_source(source, &other).await {
            return Some((path::url_of(id, name), kind));
        }
    }
    None
}

async fn same_source(a: &Source, b: &Source) -> bool {
    match (a, b) {
        (Source::Handle(a), Source::Handle(b)) => {
            let args = js_sys::Array::of1(b);
            let same = super::read::call(a, "isSameEntry", &args).await;
            same.ok().and_then(|s| s.as_bool()) == Some(true)
        }
        (Source::File(a), Source::File(b)) => {
            a.size() == b.size() && a.last_modified() == b.last_modified()
        }
        _ => false,
    }
}

/// The sources opened (or refused, by name) since the last call.
pub fn take_opened() -> Vec<Opened> {
    OPENED.with(|o| std::mem::take(&mut *o.borrow_mut()))
}
