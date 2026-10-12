use crate::i18n::keys::ColormapKey;

pub fn text(key: ColormapKey) -> Option<&'static str> {
    Some(match key {
        ColormapKey::Sequential => "Sequenziell",
        ColormapKey::Diverging => "Divergierend",
        ColormapKey::Cyclic => "Zyklisch",
        ColormapKey::Categorical => "Kategorisch",
        ColormapKey::Other => "Sonstige",
        ColormapKey::InterpolationLinear => "Linear",
        ColormapKey::BlendLinearRgb => "Lineares RGB",
        ColormapKey::FilterAll => "Alle",
        ColormapKey::FamilyCustom => "Eigene",
        ColormapKey::FamilyTitle => "Familie: {}",
        ColormapKey::Colormap => "Farbtabelle",
        ColormapKey::Smooth => "Glatt",
        ColormapKey::SmoothHover => {
            "Die Farben dieser Palette zu einem stetigen Verlauf mischen (Oklab)"
        }
        ColormapKey::Reversed => "Umgekehrt",
        ColormapKey::SearchHint => "Farbtabellen suchen...",
        ColormapKey::NoMatches => "Keine passenden Farbtabellen",
        ColormapKey::ShowColorbar => "Farbleiste zeigen",
        ColormapKey::BoxTransparency => "Kastentransparenz",
        ColormapKey::CustomColormap => "Eigene Farbtabelle",
        ColormapKey::Name => "Name",
        ColormapKey::Colors => "Farben",
        ColormapKey::ColorsHover => "Hex, rgb(), hsl() oder CSS-Farbnamen, optional mit Positionen",
        ColormapKey::Interpolation => "Interpolation",
        ColormapKey::Blend => "Mischung",
        ColormapKey::Classes => "Klassen",
        ColormapKey::ClassesHover => "0 lässt den Verlauf stetig",
        ColormapKey::RenameAndApply => "Umbenennen & anwenden",
        ColormapKey::SaveAndApply => "Speichern & anwenden",
        ColormapKey::SaveAsNew => "Als neu speichern",
        ColormapKey::NameTaken => "Eine gespeicherte Farbtabelle hat bereits diesen Namen",
        ColormapKey::NameRequired => "Name erforderlich",
        ColormapKey::Saved => "Gespeichert",
        ColormapKey::BrokenHover => {
            "Lässt sich mit dieser Version nicht erstellen: bearbeiten oder löschen"
        }
        ColormapKey::Edit => "Bearbeiten",
        ColormapKey::Delete => "Löschen",
        ColormapKey::DeleteHover => "Farbtabelle löschen",
        ColormapKey::FlipOrientation => "Ausrichtung drehen",
        ColormapKey::RangeMinHover => {
            "Unteres Bereichsende (Min). Ziehen zum Anpassen oder klicken zum Eingeben."
        }
        ColormapKey::RangeMaxHover => {
            "Oberes Bereichsende (Max). Ziehen zum Anpassen oder klicken zum Eingeben."
        }
        ColormapKey::LowClipTitle => "Farbe unter dem Bereich (< Min)",
        ColormapKey::LowClipHover => {
            "Farbe unter dem Bereich (< Min). Klicken, um eine Farbe zu wählen."
        }
        ColormapKey::HighClipTitle => "Farbe über dem Bereich (> Max)",
        ColormapKey::HighClipHover => {
            "Farbe über dem Bereich (> Max). Klicken, um eine Farbe zu wählen."
        }
        ColormapKey::HoveredValue => "Wert: {}",
        ColormapKey::TitleHover => "Titel der Farbleiste. Klicken zum Bearbeiten.",
        ColormapKey::ColorPicker => "Farbwähler",
        ColormapKey::Copy => "Kopieren",
        ColormapKey::CopyHover => "Farbwerte in die Zwischenablage kopieren",
        ColormapKey::AlphaNotANumber => "'{}' ist keine Zahl",
        ColormapKey::AlphaOutOfRange => "{} liegt außerhalb von 0..1",
        ColormapKey::AlphaMixedForms => "entweder Werte oder position:alpha-Stopps verwenden",
        ColormapKey::AlphaUnsorted => "Stopp-Positionen müssen ansteigen",
        ColormapKey::PrefsUnreadable => "Farbtabellen-Einstellungen nicht lesbar",
        ColormapKey::PrefsBackedUp => "Gesichert unter `{}`",
        ColormapKey::PrefsNotBackedUp => "Farbtabellen-Einstellungen nicht gesichert",
        ColormapKey::BackupsFull => {
            "Alle {} Sicherungen der Farbtabellen-Einstellungen sind belegt"
        }
        ColormapKey::CustomNotLoaded => "Eigene Farbtabelle nicht geladen",
        ColormapKey::CustomDoesNotBuild => "`{}` lässt sich nicht erstellen: {}",
        ColormapKey::NotRegistered => "Farbtabelle {} wurde nicht registriert",
        ColormapKey::BundledUnreadable => "Mitgelieferte Farbtabellen nicht lesbar",
        ColormapKey::FallingBackToGray => "{}; Rückfall auf Graustufen",
    })
}
