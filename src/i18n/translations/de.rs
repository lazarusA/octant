//! German (Deutsch) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Datensatz",
        Key::Variables => "Variablen",
        Key::Dimensions => "Dimensionen",
        Key::Plot => "Diagramm",
        Key::Colormaps => "Farbtabellen",
        Key::Settings => "Einstellungen",
        Key::About => "Über Octant",

        Key::InspectMetadata => "Speicher-Metadaten prüfen",
        Key::ActiveDatasets => "Aktive Datensätze",
        Key::NoActiveDatasets => {
            "Keine aktiven Datensätze. Öffnen Sie einen Datensatz, um zu beginnen."
        }
        Key::CacheUsage => "Cache-Nutzung",
        Key::ClearCache => "Cache leeren",
        Key::OpenLocal => "Lokale Datei öffnen",
        Key::CloudStore => "Cloud-Objektspeicher",
        Key::SampleDatasets => "Beispieldatensätze",
        Key::DropFilesHere => "Dateien oder Ordner hier ablegen",
        Key::SourceUrlOrPath => "Speicher-URI oder lokaler Pfad",
        Key::Inspect => "Prüfen",
        Key::LoadingMetadata => "Metadaten werden im Hintergrund geprüft...",

        Key::Group => "Gruppe:",
        Key::DataType => "Typ",
        Key::Shape => "Form",
        Key::Description => "Beschreibung",
        Key::SearchVariables => "Variablen suchen...",
        Key::NoVariablesMatching => "Keine passenden Variablen gefunden",
        Key::ShowingItemsOfTotal => "Elemente angezeigt von insgesamt",

        Key::Ready => "Bereit",
        Key::Loading => "Laden...",
        Key::Close => "Schließen",
        Key::Cancel => "Abbrechen",
        Key::Save => "Speichern",
        Key::Reset => "Zurücksetzen",
        Key::Error => "Fehler",

        Key::Play => "Abspielen",
        Key::Pause => "Pause",
        Key::StepForward => "Schritt vorwärts",
        Key::StepBackward => "Schritt rückwärts",
        Key::Speed => "Geschwindigkeit",
        Key::Loop => "Schleife",

        Key::AboutSubtitle => "N-Dimensionaler Daten-Explorer",
        Key::AboutDescription => {
            "Octant ist ein interaktiver Betrachter für N-dimensionale Datensätze mit nativer Unterstützung für lokalen und Cloud-Objektspeicher, Zarr (v2/v3) und Icechunk. Entwickelt in Rust mit GPU-beschleunigtem Rendering über WGPU. Läuft nativ auf macOS, Linux und Windows."
        }
        Key::AboutHyperslabTitle => "Hyperslab-Slicing",
        Key::AboutHyperslabDesc => "Asynchroner LRU-Chunk-Cache und Multi-Resolution-Pyramiden.",
        Key::AboutStorageTitle => "Nativ für Zarr & Icechunk",
        Key::AboutStorageDesc => "Lokale, S3-, GCS-, Azure- und HTTP-Streaming-Backends.",
        Key::AboutShadersTitle => "Hardware-beschleunigte WGPU-Shader",
        Key::AboutShadersDesc => "2D-Karten, 3D-Kugeln, Geländeflächen, Volumina & 1D-Profile.",
        Key::AboutLinksHeading => "Links & Ressourcen",
        Key::AboutGoodFaithNotice => {
            "Octant würdigt Werke Dritter nach bestem Wissen. Sollte etwas fehlen oder fehlerhaft sein, informieren Sie uns bitte zur Korrektur."
        }
        Key::AboutBrowseIcons => "Native Vektor-Icons durchsuchen",
    })
}
