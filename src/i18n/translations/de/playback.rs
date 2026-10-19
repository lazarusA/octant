use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Abspielen",
        PlaybackKey::Pause => "Pause",
        PlaybackKey::Loop => "Schleife",
        PlaybackKey::FirstStep => "Erster Schritt",
        PlaybackKey::PreviousStep => "Vorheriger Schritt",
        PlaybackKey::NextStep => "Nächster Schritt",
        PlaybackKey::LastStep => "Letzter Schritt",
        PlaybackKey::Crop => "Zuschnitt",
        PlaybackKey::CropHover => "Hilfslinien für den Zuschnitt",
        PlaybackKey::PlaybackSpeed => "Wiedergabetempo",
        PlaybackKey::PlaybackBuffer => "Puffer vor der Wiedergabe",
        PlaybackKey::Buffering => "Puffern {}/{}",
        PlaybackKey::SaveExportHover => {
            "Abbildung speichern / exportieren (Cmd+S: Schnellspeichern, Cmd+Shift+S: Einstellungen)"
        }
        PlaybackKey::CollapsedHint => "Wiedergabe (klicken oder ziehen zum Aufklappen)",
        PlaybackKey::TimelineDetails => "Zeitleistendetails",
        PlaybackKey::TimelineDimension => "Dimension: {}",
        PlaybackKey::TimelineStep => "Schritt: {} / {}",
        PlaybackKey::TimelineCurrent => "Aktuell: {}",
        PlaybackKey::TimelineRange => "Bereich: {} -> {}",
        PlaybackKey::TimelineStepSize => "Schrittweite: {}",
        PlaybackKey::DefaultStepSize => "Schritt: 1",
    })
}
