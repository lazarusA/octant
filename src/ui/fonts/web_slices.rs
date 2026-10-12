//! Web only: the slices of the Noto CJK fonts that a dataset's text needs,
//! fetched from `fonts/cjk/<code>/` next to the page (built by
//! `tools/cjk_slices` in `pages.yml`). Each font's index is fetched once;
//! then only the slices holding characters not requested before.

use super::slice_index::{INDEX_FILE, SliceIndex, slice_file};
use super::{CjkFont, store};
use crate::i18n::{NoticesKey, filled, tr};
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

/// Fetches the slices of `font` holding `chars`, then reinstalls the fonts.
/// Characters `font` lacks (or all of them, when its index is unavailable)
/// go on to the first of `rest`.
pub fn request(ctx: egui::Context, font: CjkFont, chars: BTreeSet<char>, rest: Vec<CjkFont>) {
    wasm_bindgen_futures::spawn_local(async move {
        let index = usable_index(font).await;
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
            super::apply(&ctx, crate::i18n::language());
            ctx.request_repaint();
        }
    });
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
    let slot = store::index(font);
    let cached = INDEXES
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(slot)
        .cloned()
        .flatten();
    if let Some(index) = cached {
        return Ok(index);
    }
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
