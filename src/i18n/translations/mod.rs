//! Translation registry and language dispatch with canonical English fallback.

/// Declares the module of language `$lang` (`translations/$lang/`, one file
/// per UI area) and its `text`, which looks a key up in its area's file and
/// returns `$ret`: English's every text, the others' `Option` (English fills
/// what they lack).
macro_rules! language {
    ($lang:ident, $doc:literal, $ret:ty) => {
        #[doc = $doc]
        mod $lang {
            mod about;
            mod colormap;
            mod common;
            mod dataset;
            mod export;
            mod hover;
            mod keymap;
            mod notices;
            mod playback;
            mod plot;
            mod plot_options;
            mod settings;
            mod variables;

            use crate::i18n::keys::Key;

            pub fn text(key: Key) -> $ret {
                match key {
                    Key::Common(key) => common::text(key),
                    Key::Dataset(key) => dataset::text(key),
                    Key::Variables(key) => variables::text(key),
                    Key::Playback(key) => playback::text(key),
                    Key::Plot(key) => plot::text(key),
                    Key::PlotOptions(key) => plot_options::text(key),
                    Key::Settings(key) => settings::text(key),
                    Key::Colormap(key) => colormap::text(key),
                    Key::Export(key) => export::text(key),
                    Key::Hover(key) => hover::text(key),
                    Key::Keymap(key) => keymap::text(key),
                    Key::About(key) => about::text(key),
                    Key::Notices(key) => notices::text(key),
                }
            }
        }
    };
}

language!(
    de,
    "German translations, one file per UI area.",
    Option<&'static str>
);
language!(
    en,
    "English (canonical) translations, one file per UI area.",
    &'static str
);
language!(
    es,
    "Spanish translations, one file per UI area.",
    Option<&'static str>
);
language!(
    fi,
    "Finnish translations, one file per UI area.",
    Option<&'static str>
);
language!(
    fr,
    "French translations, one file per UI area.",
    Option<&'static str>
);
language!(
    id,
    "Indonesian translations, one file per UI area.",
    Option<&'static str>
);
language!(
    it,
    "Italian translations, one file per UI area.",
    Option<&'static str>
);
language!(
    ja,
    "Japanese translations, one file per UI area.",
    Option<&'static str>
);
language!(
    ko,
    "Korean translations, one file per UI area.",
    Option<&'static str>
);
language!(
    no,
    "Norwegian translations, one file per UI area.",
    Option<&'static str>
);
language!(
    pt,
    "Portuguese translations, one file per UI area.",
    Option<&'static str>
);
language!(
    ru,
    "Russian translations, one file per UI area.",
    Option<&'static str>
);
language!(
    sv,
    "Swedish translations, one file per UI area.",
    Option<&'static str>
);
language!(
    uk,
    "Ukrainian translations, one file per UI area.",
    Option<&'static str>
);
language!(
    zh,
    "Simplified Chinese translations, one file per UI area.",
    Option<&'static str>
);

use crate::i18n::keys::Key;
use crate::i18n::lang::Language;

/// Looks up the translation for `(lang, key)`, falling back to English for
/// untranslated keys.
#[inline]
pub fn lookup(lang: Language, key: Key) -> &'static str {
    exact(lang, key).unwrap_or_else(|| en::text(key))
}

/// The translation of `key` in `lang` itself, `None` when it has none.
pub fn exact(lang: Language, key: Key) -> Option<&'static str> {
    match lang {
        Language::English => Some(en::text(key)),
        Language::Spanish => es::text(key),
        Language::French => fr::text(key),
        Language::German => de::text(key),
        Language::Portuguese => pt::text(key),
        Language::Italian => it::text(key),
        Language::Ukrainian => uk::text(key),
        Language::Russian => ru::text(key),
        Language::Indonesian => id::text(key),
        Language::Swedish => sv::text(key),
        Language::Norwegian => no::text(key),
        Language::Finnish => fi::text(key),
        Language::ChineseSimp => zh::text(key),
        Language::Japanese => ja::text(key),
        Language::Korean => ko::text(key),
    }
}
