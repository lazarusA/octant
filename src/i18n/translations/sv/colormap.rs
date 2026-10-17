use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Sekventiell",
        ColormapKey::Diverging => "Divergerande",
        ColormapKey::Cyclic => "Cyklisk",
        ColormapKey::Categorical => "Kategorisk",
        ColormapKey::Other => "Övrig",
        ColormapKey::InterpolationLinear => "Linjär",
        ColormapKey::BlendLinearRgb => "Linjär RGB",
        ColormapKey::FilterAll => "Alla",
        ColormapKey::FamilyCustom => "Anpassade",
        ColormapKey::FamilyTitle => "Familj: {}",
        ColormapKey::Smooth => "Mjuk",
        ColormapKey::SmoothHover => "Blanda palettens färger till en kontinuerlig gradient (Oklab)",
        ColormapKey::Reversed => "Omvänd",
        ColormapKey::SearchHint => "Sök färgkartor...",
        ColormapKey::NoMatches => "Inga färgkartor matchar",
        ColormapKey::ShowColorbar => "Visa färgskala",
        ColormapKey::ChangeColormapHover => "Klicka för att byta färgkarta",
        ColormapKey::BoxTransparency => "Rutans transparens",
        ColormapKey::CustomColormap => "Anpassad färgkarta",
        ColormapKey::Name => "Namn",
        ColormapKey::Colors => "Färger",
        ColormapKey::ColorsHover => "Hex, rgb(), hsl() eller CSS-färgnamn, valfritt med positioner",
        ColormapKey::Interpolation => "Interpolering",
        ColormapKey::Blend => "Blandning",
        ColormapKey::Classes => "Klasser",
        ColormapKey::ClassesHover => "0 behåller gradienten kontinuerlig",
        ColormapKey::RenameAndApply => "Byt namn och använd",
        ColormapKey::SaveAndApply => "Spara och använd",
        ColormapKey::SaveAsNew => "Spara som ny",
        ColormapKey::NameTaken => "En sparad färgkarta har redan detta namn",
        ColormapKey::NameRequired => "Namn krävs",
        ColormapKey::Saved => "Sparad",
        ColormapKey::BrokenHover => {
            "Kan inte byggas med denna version: redigera för att åtgärda, eller ta bort"
        }
        ColormapKey::Edit => "Redigera",
        ColormapKey::Delete => "Ta bort",
        ColormapKey::DeleteHover => "Ta bort färgkarta",
        ColormapKey::FlipOrientation => "Vänd orientering",
        ColormapKey::RangeMinHover => {
            "Nedre gräns (Min). Dra för att justera eller klicka för att skriva."
        }
        ColormapKey::RangeMaxHover => {
            "Övre gräns (Max). Dra för att justera eller klicka för att skriva."
        }
        ColormapKey::LowClipTitle => "Färg för nedre klippning (< Min)",
        ColormapKey::LowClipHover => "Färg för nedre klippning (< Min). Klicka för att välja färg.",
        ColormapKey::HighClipTitle => "Färg för övre klippning (> Max)",
        ColormapKey::HighClipHover => "Färg för övre klippning (> Max). Klicka för att välja färg.",
        ColormapKey::HoveredValue => "Värde: {}",
        ColormapKey::TitleHover => "Färgskalans titel. Klicka för att redigera.",
        ColormapKey::ColorPicker => "Färgväljare",
        ColormapKey::Copy => "Kopiera",
        ColormapKey::CopyHover => "Kopiera färgvärden till urklipp",
        ColormapKey::AlphaNotANumber => "'{}' är inte ett tal",
        ColormapKey::AlphaOutOfRange => "{} ligger utanför 0..1",
        ColormapKey::AlphaMixedForms => "använd antingen värden eller position:alfa-stopp",
        ColormapKey::AlphaUnsorted => "stoppens positioner måste öka",
        ColormapKey::PrefsUnreadable => "Färgkartsinställningarna kan inte läsas",
        ColormapKey::PrefsBackedUp => "Säkerhetskopierade under `{}`",
        ColormapKey::PrefsNotBackedUp => "Färgkartsinställningarna säkerhetskopierades inte",
        ColormapKey::BackupsFull => "Alla {} säkerhetskopior av färgkartsinställningar används",
        ColormapKey::CustomNotLoaded => "Anpassad färgkarta lästes inte in",
        ColormapKey::CustomDoesNotBuild => "`{}` kan inte byggas: {}",
        ColormapKey::NotRegistered => "Färgkartan {} registrerades inte",
        ColormapKey::BundledUnreadable => "De medföljande färgkartorna kan inte läsas",
        ColormapKey::FallingBackToGray => "{}; använder gråskala i stället",
    })
}
