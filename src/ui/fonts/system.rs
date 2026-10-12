//! Desktop only: the CJK fonts the operating system already has, for text
//! in datasets that the bundled UI subsets don't cover. The system font
//! folders are scanned once per session (`fontdb`), the first time a
//! dataset needs one; each font file is read once and shared by its faces,
//! as `'static` bytes egui doesn't copy on font rebuilds.

use super::{CjkFont, store};
use crate::i18n::{NoticesKey, filled, tr};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Families tried for each font, in order: macOS, Windows, then the Noto
/// and Source Han fonts most Linux systems install.
pub fn families(font: CjkFont) -> &'static [&'static str] {
    match font {
        CjkFont::Sc => &[
            "PingFang SC",
            "Hiragino Sans GB",
            "Heiti SC",
            "Microsoft YaHei",
            "Noto Sans CJK SC",
            "Source Han Sans SC",
            "WenQuanYi Micro Hei",
        ],
        CjkFont::Jp => &[
            "Hiragino Sans",
            "Hiragino Kaku Gothic ProN",
            "Yu Gothic",
            "Meiryo",
            "Noto Sans CJK JP",
            "Source Han Sans JP",
        ],
        CjkFont::Kr => &[
            "Apple SD Gothic Neo",
            "Malgun Gothic",
            "Noto Sans CJK KR",
            "Source Han Sans KR",
            "NanumGothic",
        ],
    }
}

/// The system's fonts, scanned on first use.
fn database() -> &'static fontdb::Database {
    static DATABASE: OnceLock<fontdb::Database> = OnceLock::new();
    DATABASE.get_or_init(|| {
        let mut database = fontdb::Database::new();
        database.load_system_fonts();
        database
    })
}

/// Font files read so far, by path: every face a file holds shares its
/// bytes (Linux's Noto Sans CJK is one collection for all three languages).
static FILES: Mutex<Vec<(PathBuf, &'static [u8])>> = Mutex::new(Vec::new());

/// The bytes of the font file at `path`, read once per session.
fn file_bytes(path: &Path) -> Option<&'static [u8]> {
    let mut files = FILES.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(&(_, bytes)) = files.iter().find(|(p, _)| p == path) {
        return Some(bytes);
    }
    let bytes = store::leak(std::fs::read(path).ok()?);
    files.push((path.to_path_buf(), bytes));
    Some(bytes)
}

/// The first installed family of `font` (its regular face).
pub fn load(font: CjkFont) -> Option<Arc<egui::FontData>> {
    let database = database();
    families(font).iter().find_map(|&family| {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            ..Default::default()
        };
        let (source, index) = database.face_source(database.query(&query)?)?;
        let bytes = match source {
            fontdb::Source::File(path) => file_bytes(&path)?,
            fontdb::Source::Binary(data) => store::leak((*data).as_ref().to_vec()),
        };
        let mut data = egui::FontData::from_static(bytes);
        data.index = index;
        Some(Arc::new(data))
    })
}

/// Whether `data` draws every one of `chars`.
pub(super) fn covers(data: &egui::FontData, chars: &BTreeSet<char>) -> bool {
    use skrifa::MetadataProvider;
    let Ok(face) = skrifa::FontRef::from_index(&data.font, data.index) else {
        return false;
    };
    let charmap = face.charmap();
    chars.iter().all(|&c| charmap.map(c).is_some())
}

/// Fonts asked for this session, by `CjkFont::ALL` index.
static REQUESTED: Mutex<[bool; 3]> = Mutex::new([false; 3]);

/// Marks `font` requested; `true` the first time.
fn first_request(font: CjkFont) -> bool {
    let mut requested = REQUESTED.lock().unwrap_or_else(|p| p.into_inner());
    requested
        .get_mut(store::index(font))
        .is_some_and(|done| !std::mem::replace(done, true))
}

/// Loads, in the background, the system fonts among `needed` (by
/// `CjkFont::ALL` index) not requested before, and the Chinese font as a
/// catch-all when the font for Han text (`han`) lacks some of `han_chars`
/// (each region's font covers a different set; the Chinese one the most).
/// Then reinstalls the fonts. A font the system lacks is reported once.
pub fn request(ctx: &egui::Context, needed: [bool; 3], han: CjkFont, han_chars: BTreeSet<char>) {
    let fonts: Vec<CjkFont> = CjkFont::ALL
        .into_iter()
        .zip(needed)
        .filter(|&(font, need)| need && first_request(font))
        .map(|(font, _)| font)
        .collect();
    let catch_all =
        han != CjkFont::Sc && !han_chars.is_empty() && store::system(CjkFont::Sc).is_none();
    if fonts.is_empty() && !catch_all {
        return;
    }
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        for font in fonts {
            load_or_report(font);
        }
        let han_covered = store::system(han).is_some_and(|data| covers(&data, &han_chars));
        if catch_all && !han_covered && first_request(CjkFont::Sc) {
            load_or_report(CjkFont::Sc);
        }
        super::fonts_arrived(&ctx);
    });
}

/// Loads `font` from the system, or reports that it has none.
fn load_or_report(font: CjkFont) {
    match load(font) {
        Some(data) => store::set_system(font, data),
        None => crate::ui::toast::report(
            crate::ui::toast::Severity::Warning,
            tr(NoticesKey::DataFontMissing),
            filled(
                tr(NoticesKey::DataFontMissingDetail),
                &[&font.language().display_name()],
            ),
        ),
    }
}
