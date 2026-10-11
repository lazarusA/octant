use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Schließen",
        NoticesKey::CopyDetails => "Details kopieren",
        NoticesKey::RevealInFinder => "Im Finder zeigen",
        NoticesKey::OpenFolder => "Ordner öffnen",
        NoticesKey::NoVariablesFound => "Keine Variablen gefunden",
        NoticesKey::CouldntOpenDataset => "Datensatz konnte nicht geöffnet werden",
        NoticesKey::UnsupportedFile => "Nicht unterstützte Datei",
        NoticesKey::CouldntLoadData => "Daten konnten nicht geladen werden",
        NoticesKey::CoordinatesUnavailable => "Koordinaten nicht verfügbar",
        NoticesKey::CoordinatesFailedDetail => {
            "Koordinaten von '{}' konnten nicht geladen werden ({}); Achsen zeigen Indizes"
        }
        NoticesKey::CoastlinesUnavailable => "Küstenlinien nicht verfügbar",
        NoticesKey::VolumeTooLargeForGpu => "Volumen zu groß für die GPU",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "Größe des Volumendatenpuffers ({} Elemente / {} Bytes) überschreitet das maximale GPU-Speicherpufferlimit ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Volumen {}x{}x{} überschreitet das GPU-Limit für 3D-Texturen ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Volumen-Upload übersprungen",
        NoticesKey::VolumeUploadMismatchDetail => "{} Werte für eine {}x{}x{}-Textur",
        NoticesKey::DataTooLargeForGpuBuffer => "Daten zu groß für den GPU-Puffer",
        NoticesKey::BufferTooLargeDetail => {
            "{}: Datengröße ({} Bytes) überschreitet die GPU-Pufferkapazität ({} Bytes), Schreiben übersprungen"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: Bereich ab Element {} (+{} Elemente) überschreitet die GPU-Pufferkapazität ({} Bytes), Schreiben übersprungen"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Linienplot zu groß für die GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Die Linien belegen {} Bytes; diese GPU bindet höchstens {}."
        }
    })
}
