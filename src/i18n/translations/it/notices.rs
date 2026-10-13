use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Chiudi",
        NoticesKey::CopyDetails => "Copia dettagli",
        NoticesKey::RevealInFinder => "Mostra nel Finder",
        NoticesKey::OpenFolder => "Apri cartella",
        NoticesKey::NoVariablesFound => "Nessuna variabile trovata",
        NoticesKey::CouldntOpenDataset => "Impossibile aprire il set di dati",
        NoticesKey::UnsupportedFile => "File non supportato",
        NoticesKey::CouldntLoadData => "Impossibile caricare i dati",
        NoticesKey::CoordinatesUnavailable => "Coordinate non disponibili",
        NoticesKey::CoordinatesFailedDetail => {
            "Le coordinate di '{}' non sono state caricate ({}); gli assi mostrano gli indici"
        }
        NoticesKey::CoastlinesUnavailable => "Linee di costa non disponibili",
        NoticesKey::VolumeTooLargeForGpu => "Volume troppo grande per la GPU",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "La dimensione del buffer dei dati del volume ({} elementi / {} byte) supera il limite massimo del buffer di archiviazione della GPU ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Il volume {}x{}x{} supera il limite delle texture 3D della GPU ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Caricamento del volume saltato",
        NoticesKey::VolumeUploadMismatchDetail => "{} valori per una texture {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "Dati troppo grandi per il buffer della GPU",
        NoticesKey::BufferTooLargeDetail => {
            "{}: la dimensione dei dati ({} byte) supera la capacità del buffer della GPU ({} byte), scrittura saltata"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: l'intervallo dall'elemento {} (+{} elementi) supera la capacità del buffer della GPU ({} byte), scrittura saltata"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Grafico a linee troppo grande per la GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Le linee occupano {} byte; questa GPU ne collega al massimo {}."
        }
        NoticesKey::FontLoadFailed => "Impossibile caricare un font",
        NoticesKey::FontLoadFailedDetail => "Il testo in {} potrebbe apparire come riquadri: {}",
        NoticesKey::DataFontMissing => "Nessun font per parte del testo del set di dati",
        NoticesKey::DataFontMissingDetail => {
            "Il testo in {} di questo set di dati potrebbe apparire come riquadri. Installa un font come Noto Sans CJK."
        }
        NoticesKey::OmeMetadataNonConforming => "I metadati OME-Zarr non sono conformi",
        NoticesKey::OmeMetadataNonConformingDetail => {
            "I metadati della versione {} di {} non seguono la specifica; viene mostrato ciò che è stato possibile leggere."
        }
    })
}
