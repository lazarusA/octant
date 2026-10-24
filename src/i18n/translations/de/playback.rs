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
        PlaybackKey::VoiceListen => "Zuhören starten",
        PlaybackKey::VoiceStop => "Zuhören beenden",
        PlaybackKey::VoiceListening => "Hört zu...",
        PlaybackKey::VoiceMicOff => "Mikrofon aus",
        PlaybackKey::VoiceMicError => "Mikrofon nicht verfügbar",
        PlaybackKey::VoiceSilent => "Das Mikrofon liefert nur Stille",
        PlaybackKey::VoiceSilentHint => {
            "Erlauben Sie der App, die Octant gestartet hat, in den Datenschutzeinstellungen des Systems den Mikrofonzugriff und starten Sie sie neu."
        }
        PlaybackKey::VoiceSttFailed => "Transkription fehlgeschlagen",
        PlaybackKey::VoiceDownloading => "Sprachmodell wird heruntergeladen: {}%",
        PlaybackKey::VoiceLoading => "Sprachmodell wird geladen...",
        PlaybackKey::VoiceModelHover => "Modell für Spracherkennung",
        PlaybackKey::VoiceModelSize => "{} ({} MB)",
        PlaybackKey::VoiceCloseHover => "Spracheingabe schließen",
        PlaybackKey::VoiceDidOpen => "{} wird geöffnet",
        PlaybackKey::VoiceNoSource => "Kein Datensatz passt zu '{}'",
        PlaybackKey::VoiceDidSelect => "{} ausgewählt",
        PlaybackKey::VoiceDidPlot => "{} als {} dargestellt",
        PlaybackKey::VoiceDidColormap => "Farbskala {}",
        PlaybackKey::VoiceNotUnderstood => "Befehl nicht verstanden",
        PlaybackKey::VoiceNoDataset => "Öffnen Sie zuerst einen Datensatz",
        PlaybackKey::VoiceNoVariable => "Keine Variable passt zu '{}'",
        PlaybackKey::VoiceNoPlotType => "Kein Diagrammtyp passt zu '{}'",
        PlaybackKey::VoiceNoColormap => "Keine Farbskala passt zu '{}'",
        PlaybackKey::VoiceAmbiguous => "Mehrere Variablen passen zu '{}'",
        PlaybackKey::VoiceDidYouMean => "Meinten Sie: {}",
        PlaybackKey::VoiceNothingToAnimate => "Die Darstellung hat keine animierte Dimension",
        PlaybackKey::VoiceCannotPlot => "Darstellung als {} nicht möglich",
    })
}
