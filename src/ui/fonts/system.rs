//! Desktop only: the CJK fonts the operating system already has, for text
//! in datasets that the bundled UI subsets don't cover. The system font
//! folders are scanned once per session (`fontdb`), the first time a
//! dataset needs one, and a found font file is read into memory.

use super::{CjkFont, store};
use crate::i18n::{NoticesKey, filled, tr};
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

/// The first installed family of `font` (its regular face), read into memory.
pub fn load(font: CjkFont) -> Option<Arc<egui::FontData>> {
    let database = database();
    families(font).iter().find_map(|&family| {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            ..Default::default()
        };
        let (source, index) = database.face_source(database.query(&query)?)?;
        let bytes = match source {
            fontdb::Source::File(path) => std::fs::read(path).ok()?,
            fontdb::Source::Binary(data) => (*data).as_ref().to_vec(),
        };
        let mut data = egui::FontData::from_owned(bytes);
        data.index = index;
        Some(Arc::new(data))
    })
}

/// Loads, in the background, the system fonts among `needed` (by
/// `CjkFont::ALL` index) not requested before, then reinstalls the fonts.
/// A font the system lacks is reported once.
pub fn request(ctx: &egui::Context, needed: [bool; 3]) {
    static REQUESTED: Mutex<[bool; 3]> = Mutex::new([false; 3]);
    let fonts: Vec<CjkFont> = {
        let mut requested = REQUESTED.lock().unwrap_or_else(|p| p.into_inner());
        CjkFont::ALL
            .into_iter()
            .zip(needed)
            .zip(requested.iter_mut())
            .filter(|((_, need), done)| *need && !**done)
            .map(|((font, _), done)| {
                *done = true;
                font
            })
            .collect()
    };
    if fonts.is_empty() {
        return;
    }
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        for font in fonts {
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
        super::apply(&ctx, crate::i18n::language());
        ctx.request_repaint();
    });
}
