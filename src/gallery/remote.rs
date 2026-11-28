//! The docs gallery's entries (`gallery.json`, built by the octant-docs
//! site), fetched once at startup and again on Retry. One shared slot holds
//! them, read by the gallery window and the names voice commands open.

use super::types::GalleryEntry;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

macro_rules! site {
    () => {
        "https://ndvis.github.io/octant-docs"
    };
}
/// The docs site; entries' thumbnails are paths on it.
pub const SITE: &str = site!();
/// The gallery's page on the docs site, for people.
pub const GALLERY_PAGE: &str = concat!(site!(), "/gallery");
/// The gallery as data; its `version` is [`VERSION`].
pub const GALLERY_URL: &str = concat!(site!(), "/gallery.json");
const VERSION: u64 = 1;
/// Far above the gallery's ~100 KB; a larger answer is not the gallery.
const MAX_BYTES: usize = 4 << 20;
/// Addresses an entry may hold: its store, provider's page and license
/// (`app::share` `SCHEMES`, minus procedural ones); never `javascript:`,
/// `file:` or the like.
const SCHEMES: [&str; 4] = ["http://", "https://", "s3://", "gs://"];

fn remote(address: &str) -> bool {
    SCHEMES.iter().any(|s| address.trim().starts_with(s))
}

/// Where the gallery's entries are.
#[derive(Debug, Clone)]
pub enum Gallery {
    Loading,
    Ready(Arc<[GalleryEntry]>),
    Failed(Arc<str>),
}

impl Gallery {
    /// The loaded entries; none while loading or after a failure.
    pub fn entries(&self) -> &[GalleryEntry] {
        match self {
            Gallery::Ready(entries) => entries,
            _ => &[],
        }
    }
}

/// The gallery and a count of its changes (the voice vocabulary's key).
static STATE: Mutex<(u64, Gallery)> = Mutex::new((0, Gallery::Loading));

/// The gallery now.
pub fn current() -> Gallery {
    STATE.lock().unwrap_or_else(|p| p.into_inner()).1.clone()
}

/// Changes since startup: entries arrived, or a fetch failed or restarted.
pub fn generation() -> u64 {
    STATE.lock().unwrap_or_else(|p| p.into_inner()).0
}

fn set(gallery: Gallery) {
    let mut state = STATE.lock().unwrap_or_else(|p| p.into_inner());
    *state = (state.0 + 1, gallery);
}

/// Fetches the gallery in the background, repainting `ctx` when it arrives,
/// once: a failure (the desktop may run offline) is logged and shown in the
/// gallery window, which offers [`retry`]; nothing fetches again by itself.
pub fn fetch(ctx: &egui::Context) {
    start(ctx, false);
}

/// [`fetch`] asked for by the user: a failure also shows a toast.
pub fn retry(ctx: &egui::Context) {
    start(ctx, true);
}

fn start(ctx: &egui::Context, loud: bool) {
    set(Gallery::Loading);
    let ctx = ctx.clone();
    let url = GALLERY_URL;
    #[cfg(not(target_arch = "wasm32"))]
    crate::utils::executor::TaskExecutor::spawn_background(move || {
        let rt = crate::utils::executor::get_shared_tokio_rt();
        let bytes = rt.block_on(crate::data::backends::http::fetch_url_bytes(url));
        finish(bytes, &ctx, loud);
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let bytes = crate::data::backends::http::fetch_url_bytes(url).await;
        finish(bytes, &ctx, loud);
    });
}

fn finish(bytes: Result<Vec<u8>, String>, ctx: &egui::Context, loud: bool) {
    match bytes.and_then(|b| parse(&b)) {
        Ok(entries) => set(Gallery::Ready(entries.into())),
        Err(e) if loud => {
            let title = crate::i18n::tr(crate::i18n::DatasetKey::GalleryUnavailable);
            crate::ui::toast::report(crate::ui::toast::Severity::Error, title, e.clone());
            set(Gallery::Failed(e.into()));
        }
        Err(e) => {
            log::warn!("{GALLERY_URL}: {e}");
            set(Gallery::Failed(e.into()));
        }
    }
    ctx.request_repaint();
}

/// The entries of a `gallery.json`: one that fails to read, opens no remote
/// store or repeats a key is left out (and logged); a thumbnail that is no
/// path on the docs site, or a page or license address of another scheme,
/// is dropped; a whole answer of another version or size
/// is refused.
pub fn parse(bytes: &[u8]) -> Result<Vec<GalleryEntry>, String> {
    #[derive(serde::Deserialize)]
    struct Manifest {
        version: u64,
        entries: Vec<serde_json::Value>,
    }
    if bytes.len() > MAX_BYTES {
        return Err(format!("gallery.json is {} bytes, too large", bytes.len()));
    }
    let manifest: Manifest =
        serde_json::from_slice(bytes).map_err(|e| format!("gallery.json: {e}"))?;
    if manifest.version != VERSION {
        return Err(format!(
            "gallery.json version {}, this Octant reads {VERSION}: update Octant",
            manifest.version
        ));
    }
    let mut keys = HashSet::new();
    let mut entries = Vec::with_capacity(manifest.entries.len());
    for value in manifest.entries {
        match serde_json::from_value::<GalleryEntry>(value) {
            Ok(e) if !remote(&e.store) => {
                log::warn!("gallery entry {}: no remote store", e.key);
            }
            Ok(e) if !keys.insert(e.key.clone()) => {
                log::warn!("gallery entry {}: repeated key", e.key);
            }
            Ok(mut e) => {
                // A picture only from the docs site: a path on it.
                let local = |t: &String| t.starts_with('/') && !t.starts_with("//");
                e.thumbnail = e.thumbnail.filter(local);
                e.source_page = e.source_page.filter(|p| remote(p));
                e.license_url = e.license_url.filter(|u| remote(u));
                entries.push(e);
            }
            Err(err) => log::warn!("gallery entry left out: {err}"),
        }
    }
    Ok(entries)
}

/// Fills the gallery from `fixture.json` (four real entries) and an entry
/// with a view over a procedural store (opened without the network), for
/// tests that open or list entries; every test installs the same list.
#[cfg(test)]
pub fn install_fixture() {
    let parsed = parse(include_bytes!("fixture.json"));
    let Ok(mut entries) = parsed else {
        panic!("fixture: {parsed:?}");
    };
    entries.push(GalleryEntry {
        key: "fixture-matrix".into(),
        label: "Fixture Matrix".into(),
        store: "procedural://matrix2d".into(),
        open: [("cmap".to_string(), "viridis".to_string())].into(),
        ..Default::default()
    });
    let mut state = STATE.lock().unwrap_or_else(|p| p.into_inner());
    if state.1.entries().is_empty() {
        *state = (state.0 + 1, Gallery::Ready(entries.into()));
    }
}

/// The live gallery, fetched and read now (network tests).
#[cfg(all(test, not(target_arch = "wasm32")))]
pub fn fetch_blocking() -> Vec<GalleryEntry> {
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let fetched = rt.block_on(crate::data::backends::http::fetch_url_bytes(GALLERY_URL));
    match fetched.and_then(|b| parse(&b)) {
        Ok(entries) => entries,
        Err(e) => panic!("{GALLERY_URL}: {e}"),
    }
}
