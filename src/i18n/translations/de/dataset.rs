use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Aktiver Datensatz",
        DatasetKey::NoDatasetLoaded => "Kein Datensatz geladen",
        DatasetKey::MemoryCache => "Speicher & Cache",
        DatasetKey::SampleDatasets => "Beispiele",
        DatasetKey::DropFilesHere => "Datei oder Ordner per Drag & Drop",
        DatasetKey::SourceUrlOrPath => "Ziel-URL / Dateipfad",
        DatasetKey::LoadingMetadata => "Metadaten werden im Hintergrund geprüft...",
        // Dataset panel
        DatasetKey::OpenSource => "Quelle öffnen",
        DatasetKey::DatasetManager => "Datensatzverwaltung",
        DatasetKey::CloseDatasetHover => "Datensatz schließen",
        // Active dataset card
        DatasetKey::TargetUriHover => "Ziel-URI:\n{}",
        DatasetKey::VariableCountOne => "{} Variable",
        DatasetKey::VariableCountMany => "{} Variablen",
        DatasetKey::InspectVariables => "Variablen ansehen",
        DatasetKey::Reload => "Neu laden",
        DatasetKey::BadgeVolume4D => "4D-Volumen",
        DatasetKey::BadgeMatrix2D => "2D-Matrix",
        DatasetKey::BadgeStore => "Store",
        // Source intake
        DatasetKey::Load => "Laden",
        DatasetKey::TargetPathHint => "https://... oder Pfad (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Katalog...",
        DatasetKey::Clear => "Leeren",
        DatasetKey::FormatLabel => "Format:",
        DatasetKey::SampleSeasFire => "SeasFire globale Waldbrände (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Prozedurale 2D-Matrix (Test)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "Entferntes Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Lokales Zarr (Dateisystem)",
        DatasetKey::FormatRemoteIcechunk => "Entferntes Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Lokales Icechunk (Dateisystem)",
        DatasetKey::FormatRemoteGeoTiff => "Entferntes GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "Lokales GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "Lokales NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "4D-Referenzvolumen (prozedural)",
        DatasetKey::FormatMatrix2D => "Prozedurale 2D-Matrix (prozedural)",
        DatasetKey::FormatRemoteGeoTiffShort => "Entferntes GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "Lokales GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "Lokales NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "4D-Referenzvolumen",
        DatasetKey::FormatMatrix2DShort => "Prozedurale 2D-Matrix",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "Derzeit keine Datensätze in der Verwaltung.",
        DatasetKey::LoadedCount => "Geladen ({}):",
        DatasetKey::ClearAll => "Alle entfernen",
        DatasetKey::ClearAllHover => "Alle Datensätze aus der Verwaltung entfernen",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Diesen Datensatz entfernen",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} Blöcke, {}% Trefferquote",
        DatasetKey::CacheMenuTitle => "Gemeinsamer Speicher-Cache für alle Variablen",
        DatasetKey::GlobalMemoryUsage => "Gesamte RAM-Nutzung:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Speicher nach Art:",
        DatasetKey::ResidentHyperslabs => "Geladene Hyperslabs: {} MB ({} Blöcke)",
        DatasetKey::MultiResPyramid => "Multi-Res-Pyramide: {} MB",
        DatasetKey::CacheHitsMisses => "Cache-Treffer: {} | Fehltreffer: {} (Trefferquote: {}%)",
        DatasetKey::PrefetchInFlight => "Hintergrund-Prefetching: {} laufend",
        DatasetKey::AllBlocksCached => "Puffer bereit / alle Blöcke im Cache",
        DatasetKey::SummaryPerVariable => "Übersicht pro Variable:",
        DatasetKey::NoVariablesCached => "(Derzeit keine Variablen im Speicher)",
        DatasetKey::VariableCacheSize => "({} MB, {} Bl.)",
        DatasetKey::ClearVariableCacheHover => "Cache für diese Variable leeren",
        DatasetKey::PrefetchCapacitySettings => "Prefetch- & Kapazitätseinstellungen",
        DatasetKey::EnablePrefetching => "Hintergrund-Prefetching aktivieren",
        DatasetKey::CacheLimitSuffix => " MB Limit",
        DatasetKey::WindowSizeSuffix => " Fenstergröße (Schritte)",
        DatasetKey::PrefetchWorkersSuffix => " Prefetch-Worker (Threads)",
        DatasetKey::FlushAllCaches => "Alle Caches leeren",
        // Catalog
        DatasetKey::CatalogTitle => "Datensatzkatalog",
        DatasetKey::CatalogStoreCount => "{} Stores",
        DatasetKey::CatalogBlurb => {
            "Ausgewählte Cloud-Datensätze (Zarr, Icechunk) und prozedurale Referenzdaten. Wählen Sie einen Datensatz zum Laden."
        }
        DatasetKey::CatalogSearchHint => "Nach Name, Beschreibung oder URL filtern...",
        DatasetKey::CatalogCategory => "Kategorie:",
        DatasetKey::CategoryAll => "Alle",
        DatasetKey::CategoryProcedural => "Prozedural",
        DatasetKey::NoCatalogMatches => "Keine Datensätze passend zu Ihrer Suche.",
        DatasetKey::ClearSearch => "Suche leeren",
        DatasetKey::SelectAndLoadDataset => "Datensatz wählen & laden",
        DatasetKey::SelectAndLoad => "Wählen & laden",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "Typ nicht unterstützt",
        DatasetKey::DropReleaseToLoad => "Loslassen, um den Datensatz zu laden",
        DatasetKey::DropSupportedList => "Unterstützt: {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "probieren:",
        DatasetKey::HeroSeasFireHover => {
            "Globaler Waldbrand- und Klimadatensatz, neu gechunkt (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A Echtfarben-COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Synthetisches raumzeitliches 4D-Volumen",
        DatasetKey::HeroCatalogHover => "Datensatzkatalog durchsuchen",
        DatasetKey::LoadSampleHover => "Beispiel laden: {}",
        DatasetKey::HeroHintLong => "URL oder lokalen Pfad einfügen, oder Dateien irgendwo ablegen",
        DatasetKey::HeroHintMedium => "URL oder Pfad einfügen, oder Dateien ablegen",
        DatasetKey::HeroHintShort => "URL einfügen oder Dateien ablegen",
        DatasetKey::HeroDropLong => {
            "Datensatz zum Laden ablegen (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Datensatz ablegen (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Datensatz zum Laden ablegen",
        DatasetKey::HeroUnsupportedLong => {
            "Nicht unterstützter Typ. Unterstützt: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => {
            "Format nicht unterstützt (.nc, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroUnsupportedShort => "Format nicht unterstützt",
        DatasetKey::HeroInputHintLong => {
            "https://... oder Pfad (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... oder Pfad (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL oder Pfad...",
        DatasetKey::ClearInputHover => "Eingabe leeren",
        DatasetKey::LoadDatasetHover => "Datensatz laden & Variablen öffnen",
        DatasetKey::HopOctantHover => "Klicken, um den Oktanten zu wechseln",
        DatasetKey::HeroLoadingSource => "wird geladen: {}",
        DatasetKey::HeroLoading => "wird geladen...",
        DatasetKey::HeroLoaded => "geladen: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "Lokale Dateipfade können im Browser aufgrund der Sandbox-Sicherheit nicht direkt gelesen werden.\n\nSo zeigen Sie lokale Datensätze im Browser an:\n1. Stellen Sie Ihr Verzeichnis oder Ihre Datei über einen lokalen HTTP-Server bereit: `npx serve` oder `python3 -m http.server`\n2. Geben Sie die URL ein: `http://localhost:8000/my_dataset`\n\nOder starten Sie die native Desktop-Version von Octant (`cargo run --release`)."
        }
    })
}
