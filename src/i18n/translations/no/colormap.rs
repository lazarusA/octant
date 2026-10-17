use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Sekvensiell",
        ColormapKey::Diverging => "Divergerende",
        ColormapKey::Cyclic => "Syklisk",
        ColormapKey::Categorical => "Kategorisk",
        ColormapKey::Other => "Annet",
        ColormapKey::InterpolationLinear => "Lineær",
        ColormapKey::BlendLinearRgb => "Lineær RGB",
        ColormapKey::FilterAll => "Alle",
        ColormapKey::FamilyCustom => "Egendefinert",
        ColormapKey::FamilyTitle => "Familie: {}",
        ColormapKey::Smooth => "Glatt",
        ColormapKey::SmoothHover => {
            "Bland fargene i paletten til en sammenhengende gradient (Oklab)"
        }
        ColormapKey::Reversed => "Omvendt",
        ColormapKey::SearchHint => "Søk etter fargekart...",
        ColormapKey::NoMatches => "Ingen fargekart samsvarer",
        ColormapKey::ShowColorbar => "Vis fargeskala",
        ColormapKey::ChangeColormapHover => "Klikk for å bytte fargekart",
        ColormapKey::BoxTransparency => "Boksens gjennomsiktighet",
        ColormapKey::CustomColormap => "Egendefinert fargekart",
        ColormapKey::Name => "Navn",
        ColormapKey::Colors => "Farger",
        ColormapKey::ColorsHover => {
            "Hex, rgb(), hsl() eller CSS-fargenavn, eventuelt med posisjoner"
        }
        ColormapKey::Interpolation => "Interpolasjon",
        ColormapKey::Blend => "Blanding",
        ColormapKey::Classes => "Klasser",
        ColormapKey::ClassesHover => "0 holder gradienten sammenhengende",
        ColormapKey::RenameAndApply => "Gi nytt navn og bruk",
        ColormapKey::SaveAndApply => "Lagre og bruk",
        ColormapKey::SaveAsNew => "Lagre som ny",
        ColormapKey::NameTaken => "Et lagret fargekart har allerede dette navnet",
        ColormapKey::NameRequired => "Navn er påkrevd",
        ColormapKey::Saved => "Lagret",
        ColormapKey::BrokenHover => {
            "Kan ikke bygges med denne versjonen: rediger for å rette, eller slett"
        }
        ColormapKey::Edit => "Rediger",
        ColormapKey::Delete => "Slett",
        ColormapKey::DeleteHover => "Slett fargekart",
        ColormapKey::FlipOrientation => "Bytt retning",
        ColormapKey::RangeMinHover => {
            "Nedre grense (Min). Dra for å justere eller klikk for å skrive."
        }
        ColormapKey::RangeMaxHover => {
            "Øvre grense (Maks). Dra for å justere eller klikk for å skrive."
        }
        ColormapKey::LowClipTitle => "Farge for nedre klipp (< Min)",
        ColormapKey::LowClipHover => "Farge for nedre klipp (< Min). Klikk for å velge farge.",
        ColormapKey::HighClipTitle => "Farge for øvre klipp (> Maks)",
        ColormapKey::HighClipHover => "Farge for øvre klipp (> Maks). Klikk for å velge farge.",
        ColormapKey::HoveredValue => "Verdi: {}",
        ColormapKey::TitleHover => "Tittel på fargeskalaen. Klikk for å redigere.",
        ColormapKey::ColorPicker => "Fargevelger",
        ColormapKey::Copy => "Kopier",
        ColormapKey::CopyHover => "Kopier fargeverdiene til utklippstavlen",
        ColormapKey::AlphaNotANumber => "'{}' er ikke et tall",
        ColormapKey::AlphaOutOfRange => "{} er utenfor 0..1",
        ColormapKey::AlphaMixedForms => "bruk enten verdier eller posisjon:alfa-punkter",
        ColormapKey::AlphaUnsorted => "punktenes posisjoner må øke",
        ColormapKey::PrefsUnreadable => "Fargekartinnstillingene kan ikke leses",
        ColormapKey::PrefsBackedUp => "Sikkerhetskopiert under `{}`",
        ColormapKey::PrefsNotBackedUp => "Fargekartinnstillingene ble ikke sikkerhetskopiert",
        ColormapKey::BackupsFull => "Alle {} sikkerhetskopier av fargekartinnstillingene er i bruk",
        ColormapKey::CustomNotLoaded => "Egendefinert fargekart ble ikke lastet inn",
        ColormapKey::CustomDoesNotBuild => "`{}` kan ikke bygges: {}",
        ColormapKey::NotRegistered => "Fargekartet {} ble ikke registrert",
        ColormapKey::BundledUnreadable => "De medfølgende fargekartene kan ikke leses",
        ColormapKey::FallingBackToGray => "{}; bruker gråtoner i stedet",
    })
}
