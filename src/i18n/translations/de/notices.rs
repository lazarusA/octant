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
        NoticesKey::NoDataAtStep => "Keine Daten in diesem Schritt",
        NoticesKey::NoDataAtStepDetail => "'{}' hat in diesem Schritt keine Werte: Der Speicher enthält hier keine Daten. Versuchen Sie einen anderen Schritt.",
        NoticesKey::LargeChunks => "Große Chunks",
        NoticesKey::LargeChunksDetail => "'{}' ist in Chunks von {} gespeichert: Jeder Schritt lädt mindestens einen ganzen Chunk herunter, daher laden Schritte langsam.",
        NoticesKey::VolumeTooLargeForGpu => "Volumen zu groß für die GPU",
        NoticesKey::VolumeLodOff => "Detail nach Ansicht ist für dieses Volumen aus",
        NoticesKey::VolumeLodOffDetail => {
            "Seine dargestellte Stufe braucht {} Bricks; der Atlas fasst {} und hält die Hälfte für feinere frei. Das Volumen wird als eine ganze Stufe gezeichnet."
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "Seine Stufen sind keine ganzzahligen Vielfachen voneinander mit ausgerichteten Zellen, wie Bricks sie brauchen. Das Volumen wird als eine ganze Stufe gezeichnet."
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "Seine ganzzahligen Werte (etwa Label-IDs) umfassen einen zu großen Bereich, als dass Bricks sie auseinanderhalten könnten. Das Volumen wird als eine ganze Stufe gezeichnet."
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "Diese GPU kann aus dem Shader des Volumens nicht schreiben, was Bricks brauchen. Das Volumen wird als eine ganze Stufe gezeichnet."
        }
        NoticesKey::VolumeLodBrickFailed => "Ein Teil des Volumens konnte nicht feiner gelesen werden",
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
        NoticesKey::FontLoadFailed => "Schriftart konnte nicht geladen werden",
        NoticesKey::FontLoadFailedDetail => "Text in {} wird eventuell als Kästchen angezeigt: {}",
        NoticesKey::OmeMetadataNonConforming => "OME-Zarr-Metadaten sind nicht konform",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Label-IDs über 32 Bit teilen sich Farben mit kleineren IDs"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Die Metadaten der Version {} von {} folgen nicht der Spezifikation; angezeigt wird, was gelesen werden konnte."
        }
        NoticesKey::PlateWellStopped => "Well {}: das Lesen wurde abgebrochen",
        NoticesKey::PlateWellNotJson => "Well {}: die Metadaten sind kein gültiges JSON",
        NoticesKey::PlateWellNotOme => "Well {}: kein OME-Zarr-Well",
        NoticesKey::PlateWellNoMetadata => "Well {}: keine Well-Metadaten",
    })
}
