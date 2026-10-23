use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Datensatz",
        CommonKey::Variables => "Variablen",
        CommonKey::Dimensions => "Dimensionen",
        CommonKey::Settings => "Einstellungen",
        CommonKey::About => "Über Octant",
        CommonKey::Cache => "Cache",
        CommonKey::Language => "Sprache",
        CommonKey::Loading => "Laden...",
        CommonKey::CloseEsc => "Schließen (Esc)",
        CommonKey::Cancel => "Abbrechen",
        CommonKey::Save => "Speichern",
        CommonKey::Reset => "Zurücksetzen",
        CommonKey::Move => "Verschieben",
        CommonKey::MoveHover => "Ziehen zum Verschieben; Doppelklick zum Zurücksetzen",
        CommonKey::Light => "Hell",
        CommonKey::Dark => "Dunkel",
        CommonKey::SwitchToLightHover => "Zum hellen Modus wechseln",
        CommonKey::SwitchToDarkHover => "Zum dunklen Modus wechseln",
        CommonKey::ToggleHeroHover => "Startansicht ein-/ausblenden (Rechtsklick für Über Octant)",
        CommonKey::AboutOctantHover => "Über Octant",
        CommonKey::ToggleVariableControlsHover => "Variablen-Bedienfeld ein-/ausblenden",
        CommonKey::FetchingProgress => "Abruf {} / {} ({}%)",
        CommonKey::Abort => "Abbrechen",
        CommonKey::AbortHover => "Laufende Datenübertragung unterbrechen und abbrechen",
        CommonKey::Voice => "Sprache",
        CommonKey::VoiceHover => "Spracheingabe: Mikrofon und Spektrogramm",
    })
}
