//! Where each CJK font's bytes come from: bundled in the binary, (on the
//! web) fetched once and kept for the session, or (on the desktop) the
//! system's own font, loaded when a dataset's text needs it.

use super::CjkFont;
use std::sync::{Arc, Mutex};

/// Translation subsets fetched at runtime (web), by `CjkFont::ALL` index.
static FETCHED: Mutex<[Option<Arc<egui::FontData>>; 3]> = Mutex::new([None, None, None]);

/// System fonts loaded for dataset text (desktop), by `CjkFont::ALL` index.
static SYSTEM: Mutex<[Option<Arc<egui::FontData>>; 3]> = Mutex::new([None, None, None]);

/// Font slices fetched for dataset text (web): font, slice number, data.
static SLICES: Mutex<Vec<(CjkFont, u16, Arc<egui::FontData>)>> = Mutex::new(Vec::new());

/// The font tried first for Han characters in dataset text, when a dataset
/// has some; the UI language's font still comes before it.
static DATA_FIRST: Mutex<Option<CjkFont>> = Mutex::new(None);

/// The subset available for `font`: a fetched translation subset, else the
/// bundled one.
pub fn subset(font: CjkFont) -> Arc<egui::FontData> {
    fetched(font).unwrap_or_else(|| Arc::new(egui::FontData::from_static(bundled(font))))
}

/// The system font loaded for `font`, if any.
pub fn system(font: CjkFont) -> Option<Arc<egui::FontData>> {
    let slots = SYSTEM.lock().unwrap_or_else(|p| p.into_inner());
    slots.get(index(font)).and_then(Clone::clone)
}

/// Keeps `data`, the system font found for `font`.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub fn set_system(font: CjkFont, data: Arc<egui::FontData>) {
    let mut slots = SYSTEM.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(slot) = slots.get_mut(index(font)) {
        *slot = Some(data);
    }
}

/// The slices of `font` fetched so far, by slice number, with their names
/// in egui's font definitions.
pub fn slices(font: CjkFont) -> Vec<(String, Arc<egui::FontData>)> {
    let slices = SLICES.lock().unwrap_or_else(|p| p.into_inner());
    let mut of_font: Vec<_> = slices.iter().filter(|(f, _, _)| *f == font).collect();
    of_font.sort_by_key(|(_, slice, _)| *slice);
    of_font
        .into_iter()
        .map(|(f, slice, data)| (format!("{}-slice-{slice}", f.code()), Arc::clone(data)))
        .collect()
}

/// Keeps slice `slice` of `font`.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn add_slice(font: CjkFont, slice: u16, bytes: Vec<u8>) {
    let mut slices = SLICES.lock().unwrap_or_else(|p| p.into_inner());
    if !slices.iter().any(|(f, s, _)| *f == font && *s == slice) {
        slices.push((font, slice, Arc::new(egui::FontData::from_owned(bytes))));
    }
}

/// The font tried first for Han characters in dataset text.
pub fn data_first() -> Option<CjkFont> {
    *DATA_FIRST.lock().unwrap_or_else(|p| p.into_inner())
}

/// Sets [`data_first`]; `true` when it changed.
pub fn set_data_first(font: CjkFont) -> bool {
    let mut first = DATA_FIRST.lock().unwrap_or_else(|p| p.into_inner());
    first.replace(font) != Some(font)
}

/// Whether `font`'s translation subset has been fetched.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn fetched(font: CjkFont) -> Option<Arc<egui::FontData>> {
    let slots = FETCHED.lock().unwrap_or_else(|p| p.into_inner());
    slots.get(index(font)).and_then(Clone::clone)
}

/// Keeps `bytes`, the fetched translation subset of `font`.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn set_fetched(font: CjkFont, bytes: Vec<u8>) {
    let mut slots = FETCHED.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(slot) = slots.get_mut(index(font)) {
        *slot = Some(Arc::new(egui::FontData::from_owned(bytes)));
    }
}

/// Position of `font` in `CjkFont::ALL`.
pub fn index(font: CjkFont) -> usize {
    CjkFont::ALL.iter().position(|&f| f == font).unwrap_or(0)
}

/// The desktop build bundles the translation subsets.
#[cfg(not(target_arch = "wasm32"))]
pub fn bundled(font: CjkFont) -> &'static [u8] {
    match font {
        CjkFont::Sc => include_bytes!("../../../assets/fonts/noto-sans-sc-ui.otf"),
        CjkFont::Jp => include_bytes!("../../../assets/fonts/noto-sans-jp-ui.otf"),
        CjkFont::Kr => include_bytes!("../../../assets/fonts/noto-sans-kr-ui.otf"),
    }
}

/// The web build bundles only the language names; the rest is fetched.
#[cfg(target_arch = "wasm32")]
pub fn bundled(font: CjkFont) -> &'static [u8] {
    match font {
        CjkFont::Sc => include_bytes!("../../../assets/fonts/noto-sans-sc-names.otf"),
        CjkFont::Jp => include_bytes!("../../../assets/fonts/noto-sans-jp-names.otf"),
        CjkFont::Kr => include_bytes!("../../../assets/fonts/noto-sans-kr-names.otf"),
    }
}
