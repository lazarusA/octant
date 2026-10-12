//! The window title (desktop) or page title (web), "Octant: " followed by the
//! translated subtitle, kept in step with the UI language.

use crate::i18n::{AboutKey, Language, translate};

/// The title in `lang`: Chinese and Japanese join the name and subtitle
/// with a full-width colon, the others with ": ".
pub fn title_in(lang: Language) -> String {
    let separator = match lang {
        Language::ChineseSimp | Language::Japanese => "\u{FF1A}",
        _ => ": ",
    };
    format!("Octant{separator}{}", translate(lang, AboutKey::Subtitle))
}

/// Sets the title again when the language differs from the one it was last
/// set in; call every frame (the title is only rebuilt on a change).
pub fn sync(ctx: &egui::Context) {
    let lang = crate::i18n::language();
    let id = egui::Id::new("window_title_language");
    if ctx.data(|d| d.get_temp::<Language>(id)) == Some(lang) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(id, lang));
    set(ctx, title_in(lang));
}

#[cfg(not(target_arch = "wasm32"))]
fn set(ctx: &egui::Context, title: String) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
}

#[cfg(target_arch = "wasm32")]
fn set(_ctx: &egui::Context, title: String) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title(&title);
    }
}

#[cfg(test)]
mod tests {
    use super::title_in;
    use crate::i18n::Language;

    #[test]
    fn title_joins_name_and_translated_subtitle() {
        assert_eq!(
            title_in(Language::English),
            "Octant: N-Dimensional Data Viewer"
        );
        assert_eq!(
            title_in(Language::Spanish),
            "Octant: Visor de datos N-dimensionales"
        );
    }

    #[test]
    fn chinese_and_japanese_titles_use_a_full_width_colon() {
        assert_eq!(
            title_in(Language::Japanese),
            "Octant：N次元データビューアー"
        );
        assert_eq!(title_in(Language::ChineseSimp), "Octant：N 维数据查看器");
        assert_eq!(title_in(Language::Korean), "Octant: N차원 데이터 뷰어");
    }
}
