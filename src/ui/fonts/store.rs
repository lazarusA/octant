//! Where each CJK font's bytes come from: bundled in the binary, or (on the
//! web) fetched once and kept for the session.

use super::CjkFont;
use std::sync::{Arc, Mutex};

/// Translation subsets fetched at runtime (web), by `CjkFont::ALL` index.
static FETCHED: Mutex<[Option<Arc<egui::FontData>>; 3]> = Mutex::new([None, None, None]);

/// The best font data available for `font`: a fetched translation subset,
/// else the bundled one.
pub fn get(font: CjkFont) -> Option<Arc<egui::FontData>> {
    fetched(font).or_else(|| Some(Arc::new(egui::FontData::from_static(bundled(font)))))
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
