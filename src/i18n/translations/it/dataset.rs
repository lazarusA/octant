use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Set di dati attivo",
        DatasetKey::NoDatasetLoaded => "Nessun set di dati caricato",
        DatasetKey::MemoryCache => "Memoria e cache",
        DatasetKey::SampleDatasets => "Esempi",
        DatasetKey::DropFilesHere => "Trascina un file o una cartella",
        DatasetKey::SourceUrlOrPath => "URL / percorso del file",
        DatasetKey::LoadingMetadata => "Ispezione dei metadati dell'archivio in background...",
        // Dataset panel
        DatasetKey::OpenSource => "Apri sorgente",
        DatasetKey::DatasetManager => "Gestione set di dati",
        DatasetKey::CloseDatasetHover => "Chiudi set di dati",
        // Active dataset card
        DatasetKey::TargetUriHover => "URI di destinazione:\n{}",
        DatasetKey::VariableCountOne => "{} variabile",
        DatasetKey::VariableCountMany => "{} variabili",
        DatasetKey::InspectVariables => "Esamina variabili",
        DatasetKey::Reload => "Ricarica",
        DatasetKey::BadgeVolume4D => "Volume 4D",
        DatasetKey::BadgeMatrix2D => "Matrice 2D",
        DatasetKey::BadgeStore => "Archivio",
        // Source intake
        DatasetKey::Load => "Carica",
        DatasetKey::TargetPathHint => "https://... o percorso (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Catalogo...",
        DatasetKey::Clear => "Cancella",
        DatasetKey::FormatLabel => "Formato:",
        DatasetKey::SampleSeasFire => "SeasFire, incendi globali (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Matrice procedurale 2D (test)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "Zarr remoto (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Zarr locale (file system)",
        DatasetKey::FormatRemoteIcechunk => "Icechunk remoto (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Icechunk locale (file system)",
        DatasetKey::FormatRemoteGeoTiff => "GeoTIFF / COG remoto (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "GeoTIFF locale (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "NetCDF / HDF5 locale (.nc/.h5)",
        DatasetKey::FormatVolume4D => "Volume 4D di verità nota (procedurale)",
        DatasetKey::FormatMatrix2D => "Matrice procedurale 2D (procedurale)",
        DatasetKey::FormatRemoteGeoTiffShort => "GeoTIFF / COG remoto",
        DatasetKey::FormatLocalGeoTiffShort => "GeoTIFF locale",
        DatasetKey::FormatLocalNetCdfShort => "NetCDF / HDF5 locale",
        DatasetKey::FormatVolume4DShort => "Volume 4D di verità nota",
        DatasetKey::FormatMatrix2DShort => "Matrice procedurale 2D",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "Nessun set di dati registrato nella gestione.",
        DatasetKey::LoadedCount => "Caricati ({}):",
        DatasetKey::ClearAll => "Cancella tutto",
        DatasetKey::ClearAllHover => "Rimuovi tutti i set di dati dalla gestione",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Rimuovi questo set di dati",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} blocchi - {}% di successi",
        DatasetKey::CacheMenuTitle => "Cache di memoria unificata multivariabile",
        DatasetKey::GlobalMemoryUsage => "Uso globale della memoria RAM:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Ripartizione della memoria per tipo:",
        DatasetKey::ResidentHyperslabs => "Iperslab residenti: {} MB ({} blocchi)",
        DatasetKey::MultiResPyramid => "Piramide multirisoluzione: {} MB",
        DatasetKey::CacheHitsMisses => {
            "Successi in cache: {} | Mancati: {} (tasso di successo: {}%)"
        }
        DatasetKey::PrefetchInFlight => "Precaricamento in background: {} in corso",
        DatasetKey::AllBlocksCached => "Buffer pronto / tutti i blocchi in cache",
        DatasetKey::SummaryPerVariable => "Riepilogo per variabile:",
        DatasetKey::NoVariablesCached => "(Nessuna variabile attualmente in cache)",
        DatasetKey::VariableCacheSize => "({} MB, {} blocchi)",
        DatasetKey::ClearVariableCacheHover => "Svuota la cache di questa variabile",
        DatasetKey::PrefetchCapacitySettings => "Impostazioni di precaricamento e capacità",
        DatasetKey::EnablePrefetching => "Attiva il precaricamento in background",
        DatasetKey::CacheLimitSuffix => " MB di limite",
        DatasetKey::WindowSizeSuffix => " Ampiezza finestra (passi)",
        DatasetKey::PrefetchWorkersSuffix => " Worker di precaricamento (thread)",
        DatasetKey::FlushAllCaches => "Svuota tutte le cache",
        // Catalog
        DatasetKey::CatalogTitle => "Catalogo dei set di dati",
        DatasetKey::CatalogStoreCount => "{} archivi",
        DatasetKey::CatalogBlurb => {
            "Set di dati selezionati: Zarr e Icechunk nel cloud e dati procedurali di riferimento. Seleziona un set di dati per caricarlo."
        }
        DatasetKey::CatalogSearchHint => "Filtra per nome, descrizione o URL...",
        DatasetKey::CatalogCategory => "Categoria:",
        DatasetKey::CategoryAll => "Tutti",
        DatasetKey::CategoryProcedural => "Procedurali",
        DatasetKey::NoCatalogMatches => "Nessun set di dati corrisponde alla ricerca.",
        DatasetKey::ClearSearch => "Cancella ricerca",
        DatasetKey::SelectAndLoadDataset => "Seleziona e carica set di dati",
        DatasetKey::SelectAndLoad => "Seleziona e carica",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "Tipo non supportato",
        DatasetKey::DropReleaseToLoad => "Rilascia per caricare il set di dati",
        DatasetKey::DropSupportedList => "Supportati: {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "prova:",
        DatasetKey::HeroSeasFireHover => {
            "Set di dati globale su incendi e clima, con chunk riorganizzati (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A COG a colori reali (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Volume spazio-temporale 4D sintetico",
        DatasetKey::HeroCatalogHover => "Sfoglia il catalogo dei set di dati",
        DatasetKey::LoadSampleHover => "Carica esempio: {}",
        DatasetKey::HeroHintLong => "URL / percorso locale / trascina i file ovunque",
        DatasetKey::HeroHintMedium => "URL / percorso / trascina i file",
        DatasetKey::HeroHintShort => "URL / rilascia i file",
        DatasetKey::HeroDropLong => {
            "Rilascia il set di dati da caricare (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Rilascia il set di dati (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Rilascia per caricare",
        DatasetKey::HeroUnsupportedLong => {
            "Tipo non supportato. Supportati: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Formato non supportato (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Formato non supportato",
        DatasetKey::HeroInputHintLong => {
            "https://... o percorso (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... o percorso (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL o percorso...",
        DatasetKey::ClearInputHover => "Cancella il testo",
        DatasetKey::LoadDatasetHover => "Carica il set di dati e apri le variabili",
        DatasetKey::HopOctantHover => "Clicca per cambiare ottante",
        DatasetKey::HeroLoadingSource => "caricamento: {}",
        DatasetKey::HeroLoading => "caricamento...",
        DatasetKey::HeroLoaded => "caricato: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "I percorsi di file locali non possono essere letti direttamente nel browser per via della sandbox di sicurezza web.\n\nPer visualizzare set di dati locali nel browser:\n1. Servi la cartella o il file con un server HTTP locale: `npx serve` o `python3 -m http.server`\n2. Inserisci l'URL: `http://localhost:8000/my_dataset`\n\nOppure usa la versione desktop nativa di Octant (`cargo run --release`)."
        }
        DatasetKey::Plate => "Piastra",
        DatasetKey::PlateAcquisition => "Acquisizione",
        DatasetKey::PlateAllAcquisitions => "Tutte",
        DatasetKey::PlateFields => "Campi:",
        DatasetKey::PlatePickWell => "Scegli un pozzetto per vederne i campi",
        DatasetKey::PlateLoadingFields => "Lettura dei campi del pozzetto",
        DatasetKey::PlateNoFields => "Nessun campo in questa acquisizione",
        DatasetKey::PlateWellUnreadable => {
            "Impossibile leggere i campi del pozzetto. Clicca per riprovare"
        }
        DatasetKey::CategoryMicroscopy => "Microscopia",
    })
}
