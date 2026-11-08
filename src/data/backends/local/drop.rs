//! Dropping a local folder or file on the page. eframe's own drop handling
//! hands over files only, never folders, so the page listens itself (in the
//! capture phase, before eframe's canvas does) and takes the first dropped
//! item's entry: a folder's is looked up per key when read, never listed.

use super::read::entry_as_file;
use super::registry::{Source, open};
use js_sys::{Function, Reflect};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;

/// Listens for drops on the page for as long as it is open; each dropped
/// folder or file opens as a dataset (`take_opened`).
pub fn install_drop_listener(ctx: &egui::Context) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let ctx = ctx.clone();
    let on_drop =
        Closure::<dyn FnMut(web_sys::DragEvent)>::new(move |event: web_sys::DragEvent| {
            if let Some(entry) = first_entry(&event) {
                spawn_local(open_entry(entry, ctx.clone()));
            }
        });
    let listener = on_drop.as_ref().unchecked_ref();
    let _ = document.add_event_listener_with_callback_and_bool("drop", listener, true);
    // The page keeps listening until it closes.
    on_drop.forget();
}

/// The first dropped item's `FileSystemEntry`, read while the drop event
/// runs: its data transfer is empty afterwards.
fn first_entry(event: &web_sys::DragEvent) -> Option<JsValue> {
    let item = event.data_transfer()?.items().get(0)?;
    let get_entry = Reflect::get(&item, &"webkitGetAsEntry".into()).ok()?;
    let entry = get_entry.dyn_into::<Function>().ok()?.call0(&item).ok()?;
    (!entry.is_null()).then_some(entry)
}

async fn open_entry(entry: JsValue, ctx: egui::Context) {
    let read = |key: &str| Reflect::get(&entry, &key.into()).ok();
    let name = read("name").and_then(|n| n.as_string()).unwrap_or_default();
    let source = if read("isDirectory").and_then(|d| d.as_bool()) == Some(true) {
        Source::Entry(entry)
    } else {
        match entry_as_file(&entry).await {
            Ok(file) => Source::File(file),
            Err(e) => {
                let title = crate::i18n::tr(crate::i18n::NoticesKey::CouldntOpenDataset);
                let severity = crate::ui::toast::Severity::Warning;
                crate::ui::toast::report(severity, title, format!("{name}: {e:?}"));
                ctx.request_repaint();
                return;
            }
        }
    };
    open(name, source, ctx).await;
}
