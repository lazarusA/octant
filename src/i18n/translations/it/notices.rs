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
        NoticesKey::VolumeLodOff => "Il dettaglio secondo la vista è disattivato per questo volume",
        NoticesKey::VolumeLodOffDetail => {
            "Il suo livello rappresentato richiede {} blocchi; l'atlante ne contiene {} e ne lascia metà ai più fini. Il volume viene disegnato con un solo livello intero."
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "I suoi livelli non sono multipli interi l'uno dell'altro con celle allineate, come richiedono i blocchi. Il volume viene disegnato con un livello intero."
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "I suoi valori interi (come gli id delle etichette) coprono un intervallo troppo ampio perché i blocchi li distinguano. Il volume viene disegnato con un livello intero."
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "Questa GPU non può scrivere dallo shader del volume, come richiedono i blocchi. Il volume viene disegnato con un livello intero."
        }
        NoticesKey::VolumeLodBrickFailed => "Una parte del volume non è stata letta a un livello più fine",
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
        NoticesKey::OmeMetadataNonConforming => "I metadati OME-Zarr non sono conformi",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Gli id delle etichette oltre 32 bit condividono i colori con id minori"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "I metadati della versione {} di {} non seguono la specifica; viene mostrato ciò che è stato possibile leggere."
        }
        NoticesKey::PlateWellStopped => "Pozzetto {}: la lettura si è interrotta",
        NoticesKey::PlateWellNotJson => "Pozzetto {}: i metadati non sono JSON valido",
        NoticesKey::PlateWellNotOme => "Pozzetto {}: non è un pozzetto OME-Zarr",
        NoticesKey::PlateWellNoMetadata => "Pozzetto {}: nessun metadato del pozzetto",
    })
}
