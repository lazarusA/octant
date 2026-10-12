use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Sekventiaalinen",
        ColormapKey::Diverging => "Divergoiva",
        ColormapKey::Cyclic => "Syklinen",
        ColormapKey::Categorical => "Kategorinen",
        ColormapKey::Other => "Muu",
        ColormapKey::InterpolationLinear => "Lineaarinen",
        ColormapKey::BlendLinearRgb => "Lineaarinen RGB",
        ColormapKey::FilterAll => "Kaikki",
        ColormapKey::FamilyCustom => "Mukautettu",
        ColormapKey::FamilyTitle => "Perhe: {}",
        ColormapKey::Colormap => "Värikartta",
        ColormapKey::Smooth => "Sileä",
        ColormapKey::SmoothHover => "Sekoita paletin värit yhtenäiseksi liukuväriksi (Oklab)",
        ColormapKey::Reversed => "Käänteinen",
        ColormapKey::SearchHint => "Etsi värikarttoja...",
        ColormapKey::NoMatches => "Ei vastaavia värikarttoja",
        ColormapKey::ShowColorbar => "Näytä väripalkki",
        ColormapKey::BoxTransparency => "Laatikon läpinäkyvyys",
        ColormapKey::CustomColormap => "Mukautettu värikartta",
        ColormapKey::Name => "Nimi",
        ColormapKey::Colors => "Värit",
        ColormapKey::ColorsHover => {
            "Heksa, rgb(), hsl() tai CSS-värinimet, valinnaisesti sijainteineen"
        }
        ColormapKey::Interpolation => "Interpolointi",
        ColormapKey::Blend => "Sekoitus",
        ColormapKey::Classes => "Luokat",
        ColormapKey::ClassesHover => "0 pitää liukuvärin jatkuvana",
        ColormapKey::RenameAndApply => "Nimeä ja käytä",
        ColormapKey::SaveAndApply => "Tallenna ja käytä",
        ColormapKey::SaveAsNew => "Tallenna uutena",
        ColormapKey::NameTaken => "Tallennetulla värikartalla on jo tämä nimi",
        ColormapKey::NameRequired => "Nimi vaaditaan",
        ColormapKey::Saved => "Tallennettu",
        ColormapKey::BrokenHover => "Ei toimi tässä versiossa: korjaa muokkaamalla tai poista",
        ColormapKey::Edit => "Muokkaa",
        ColormapKey::Delete => "Poista",
        ColormapKey::DeleteHover => "Poista värikartta",
        ColormapKey::FlipOrientation => "Käännä suunta",
        ColormapKey::RangeMinHover => {
            "Välin alapää (Min). Säädä vetämällä tai kirjoita napsauttamalla."
        }
        ColormapKey::RangeMaxHover => {
            "Välin yläpää (Max). Säädä vetämällä tai kirjoita napsauttamalla."
        }
        ColormapKey::LowClipTitle => "Alaleikkauksen väri (< Min)",
        ColormapKey::LowClipHover => "Alaleikkauksen väri (< Min). Valitse väri napsauttamalla.",
        ColormapKey::HighClipTitle => "Yläleikkauksen väri (> Max)",
        ColormapKey::HighClipHover => "Yläleikkauksen väri (> Max). Valitse väri napsauttamalla.",
        ColormapKey::HoveredValue => "Arvo: {}",
        ColormapKey::TitleHover => "Väripalkin otsikko. Muokkaa napsauttamalla.",
        ColormapKey::ColorPicker => "Värivalitsin",
        ColormapKey::Copy => "Kopioi",
        ColormapKey::CopyHover => "Kopioi väriarvot leikepöydälle",
        ColormapKey::AlphaNotANumber => "'{}' ei ole luku",
        ColormapKey::AlphaOutOfRange => "{} on välin 0..1 ulkopuolella",
        ColormapKey::AlphaMixedForms => "käytä joko arvoja tai position:alpha-pisteitä",
        ColormapKey::AlphaUnsorted => "pisteiden sijaintien on kasvettava",
        ColormapKey::PrefsUnreadable => "Värikarttojen asetuksia ei voi lukea",
        ColormapKey::PrefsBackedUp => "Varmuuskopioitu nimellä `{}`",
        ColormapKey::PrefsNotBackedUp => "Värikarttojen asetuksia ei varmuuskopioitu",
        ColormapKey::BackupsFull => "Kaikki {} värikarttojen asetusvarmuuskopiota ovat käytössä",
        ColormapKey::CustomNotLoaded => "Mukautettua värikarttaa ei ladattu",
        ColormapKey::CustomDoesNotBuild => "`{}` ei muodostu: {}",
        ColormapKey::NotRegistered => "Värikarttaa {} ei rekisteröity",
        ColormapKey::BundledUnreadable => "Mukana tulevia värikarttoja ei voi lukea",
        ColormapKey::FallingBackToGray => "{}; käytetään harmaasävyjä",
    })
}
