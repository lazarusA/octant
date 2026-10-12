//! Web only: the slices of the Noto CJK fonts that a dataset's text needs,
//! fetched from `fonts/cjk/<code>/` next to the page (built by
//! `tools/cjk_slices` in `pages.yml`). Each font's index is fetched once
//! (requests made while it loads join it); then only the slices holding
//! characters not requested before, and only when the dataset's characters
//! changed.

use super::dataset_text::DatasetText;
use super::slice_index::{INDEX_FILE, SliceIndex, slice_file};
use super::{CjkFont, store};
use crate::i18n::{Language, NoticesKey, filled, tr};
use std::collections::{BTreeSet, HashSet};
use std::sync::{Arc, Mutex};

/// Each font's slice index, by `CjkFont::ALL` index.
static INDEXES: Mutex<[Option<Arc<SliceIndex>>; 3]> = Mutex::new([None, None, None]);

/// What this session has asked for.
#[derive(Default)]
struct Requested {
    /// Slices fetched or on their way.
    slices: HashSet<(CjkFont, u16)>,
    /// Fonts whose index failed to load: not asked again.
    failed: HashSet<CjkFont>,
}

static REQUESTED: Mutex<Option<Requested>> = Mutex::new(None);

fn with_requested<T>(
    f: impl FnOnce(&mut HashSet<(CjkFont, u16)>, &mut HashSet<CjkFont>) -> T,
) -> T {
    let mut requested = REQUESTED.lock().unwrap_or_else(|p| p.into_inner());
    let requested = requested.get_or_insert_with(Requested::default);
    f(&mut requested.slices, &mut requested.failed)
}

/// Requests the slices a dataset's text needs, unless nothing changed since
/// the last request: same dataset (`generation`), same number of characters
/// and the same font for its Han text.
pub fn request_dataset(
    ctx: &egui::Context,
    text: &DatasetText,
    generation: u64,
    han: CjkFont,
    lang: Language,
) {
    static LAST: Mutex<Option<(u64, usize, CjkFont)>> = Mutex::new(None);
    let key = (generation, text.char_count(), han);
    if LAST.lock().unwrap_or_else(|p| p.into_inner()).replace(key) == Some(key) {
        return;
    }
    for (font, chars) in CjkFont::ALL.into_iter().zip(text.by_font(han)) {
        if !chars.is_empty() {
            // Characters a font lacks go on to the next one.
            let rest = CjkFont::fallback_order(lang, Some(han))
                .into_iter()
                .filter(|&f| f != font)
                .collect();
            request(ctx.clone(), font, chars, rest);
        }
    }
}

/// Characters waiting for a font's index while it is being fetched, with
/// the fonts their misses go on to, by `CjkFont::ALL` index: requests made
/// meanwhile join them instead of fetching the index again.
static WAITING: Mutex<[Option<Waiting>; 3]> = Mutex::new([None, None, None]);

/// Characters waiting for an index, and the fonts their misses go on to.
type Waiting = (BTreeSet<char>, Vec<CjkFont>);

/// Fetches the slices of `font` holding `chars`, then reinstalls the fonts.
/// Characters `font` lacks (or all of them, when its index is unavailable)
/// go on to the first of `rest`.
pub fn request(ctx: egui::Context, font: CjkFont, chars: BTreeSet<char>, rest: Vec<CjkFont>) {
    let slot = store::index(font);
    if cached_index(font).is_none() {
        let mut waiting = WAITING.lock().unwrap_or_else(|p| p.into_inner());
        let Some(entry) = waiting.get_mut(slot) else {
            return;
        };
        if let Some((pending, _)) = entry {
            // The index is on its way: its request handles these too.
            pending.extend(chars);
            return;
        }
        *entry = Some((chars, rest));
        drop(waiting);
        wasm_bindgen_futures::spawn_local(async move {
            let index = usable_index(font).await;
            let taken = WAITING
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .get_mut(slot)
                .and_then(Option::take);
            if let Some((chars, rest)) = taken {
                serve(ctx, font, index, chars, rest).await;
            }
        });
        return;
    }
    wasm_bindgen_futures::spawn_local(async move {
        let index = usable_index(font).await;
        serve(ctx, font, index, chars, rest).await;
    });
}

/// Sends the characters `index` lacks on to the next font, and fetches the
/// slices holding the others.
async fn serve(
    ctx: egui::Context,
    font: CjkFont,
    index: Option<Arc<SliceIndex>>,
    chars: BTreeSet<char>,
    rest: Vec<CjkFont>,
) {
    let (held, missing): (BTreeSet<char>, BTreeSet<char>) = chars
        .into_iter()
        .partition(|&c| index.as_ref().is_some_and(|i| i.slice_of(c).is_some()));
    if let Some((&next, more)) = rest.split_first()
        && !missing.is_empty()
    {
        request(ctx.clone(), next, missing, more.to_vec());
    }
    if let Some(index) = index
        && fetch_slices(font, &index, &held).await
    {
        super::fonts_arrived(&ctx);
    }
}

/// `font`'s slice index if it was fetched already.
fn cached_index(font: CjkFont) -> Option<Arc<SliceIndex>> {
    INDEXES
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(store::index(font))
        .cloned()
        .flatten()
}

/// `font`'s slice index, `None` when it failed to load (reported once; the
/// font is not asked again this session).
async fn usable_index(font: CjkFont) -> Option<Arc<SliceIndex>> {
    if with_requested(|_, failed| failed.contains(&font)) {
        return None;
    }
    match index(font).await {
        Ok(index) => Some(index),
        Err(error) => {
            with_requested(|_, failed| failed.insert(font));
            report(font, &error);
            None
        }
    }
}

/// Fetches the slices of `font` holding `chars` that were not requested
/// before; `true` when at least one arrived.
async fn fetch_slices(font: CjkFont, index: &SliceIndex, chars: &BTreeSet<char>) -> bool {
    let wanted: BTreeSet<u16> = chars.iter().filter_map(|&c| index.slice_of(c)).collect();
    let new: Vec<u16> = with_requested(|slices, _| {
        wanted
            .into_iter()
            .filter(|&slice| slices.insert((font, slice)))
            .collect()
    });
    let fetches = new.iter().map(|&slice| fetch(font, slice_file(slice)));
    let mut arrived = false;
    for (&slice, result) in new.iter().zip(futures::future::join_all(fetches).await) {
        match result {
            Ok(bytes) => {
                store::add_slice(font, slice, bytes);
                arrived = true;
            }
            Err(error) => {
                with_requested(|slices, _| slices.remove(&(font, slice)));
                report(font, &error);
            }
        }
    }
    arrived
}

/// `font`'s slice index, fetched on first use.
async fn index(font: CjkFont) -> Result<Arc<SliceIndex>, String> {
    if let Some(index) = cached_index(font) {
        return Ok(index);
    }
    let slot = store::index(font);
    let bytes = fetch(font, INDEX_FILE.to_owned()).await?;
    let index = SliceIndex::decode(&bytes)
        .map(Arc::new)
        .ok_or_else(|| format!("fonts/cjk/{}/{INDEX_FILE}: unreadable", font.code()))?;
    if let Some(entry) = INDEXES
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get_mut(slot)
    {
        *entry = Some(Arc::clone(&index));
    }
    Ok(index)
}

async fn fetch(font: CjkFont, file: String) -> Result<Vec<u8>, String> {
    let url = format!("fonts/cjk/{}/{file}", font.code());
    crate::data::backends::http::fetch_url_bytes(&url).await
}

/// One toast for a font that could not be fetched.
fn report(font: CjkFont, error: &str) {
    let language = font.language().display_name();
    crate::ui::toast::report(
        crate::ui::toast::Severity::Warning,
        tr(NoticesKey::FontLoadFailed),
        filled(tr(NoticesKey::FontLoadFailedDetail), &[&language, &error]),
    );
}
