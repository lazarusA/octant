use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Aktivt datasett",
        DatasetKey::NoDatasetLoaded => "Ingen datasett lastet inn",
        DatasetKey::MemoryCache => "Minne og hurtigbuffer",
        DatasetKey::SampleDatasets => "Eksempler",
        DatasetKey::DropFilesHere => "Dra og slipp fil eller mappe",
        DatasetKey::SourceUrlOrPath => "Mål-URL / filbane",
        DatasetKey::LoadingMetadata => "Inspiserer metadata i bakgrunnen...",
        // Dataset panel
        DatasetKey::OpenSource => "Åpne kilde",
        DatasetKey::DatasetManager => "Datasettbehandler",
        DatasetKey::CloseDatasetHover => "Lukk datasett",
        // Active dataset card
        DatasetKey::TargetUriHover => "Mål-URI:\n{}",
        DatasetKey::VariableCountOne => "{} variabel",
        DatasetKey::VariableCountMany => "{} variabler",
        DatasetKey::InspectVariables => "Inspiser variabler",
        DatasetKey::Reload => "Last inn på nytt",
        DatasetKey::BadgeVolume4D => "4D-volum",
        DatasetKey::BadgeMatrix2D => "2D-matrise",
        DatasetKey::BadgeStore => "Lager",
        // Source intake
        DatasetKey::Load => "Last inn",
        DatasetKey::TargetPathHint => "https://... eller bane (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Katalog...",
        DatasetKey::Clear => "Tøm",
        DatasetKey::FormatLabel => "Format:",
        DatasetKey::SampleSeasFire => "SeasFire globale skogbranner (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Prosedyrisk 2D-matrise (test)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "Ekstern Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Lokal Zarr (filsystem)",
        DatasetKey::FormatRemoteIcechunk => "Ekstern Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Lokal Icechunk (filsystem)",
        DatasetKey::FormatRemoteGeoTiff => "Ekstern GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "Lokal GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "Lokal NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "4D-volum med kjent fasit (prosedyrisk)",
        DatasetKey::FormatMatrix2D => "Prosedyrisk 2D-matrise (prosedyrisk)",
        DatasetKey::FormatRemoteGeoTiffShort => "Ekstern GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "Lokal GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "Lokal NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "4D-volum med kjent fasit",
        DatasetKey::FormatMatrix2DShort => "Prosedyrisk 2D-matrise",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "Ingen datasett er registrert i behandleren.",
        DatasetKey::LoadedCount => "Lastet inn ({}):",
        DatasetKey::ClearAll => "Tøm alt",
        DatasetKey::ClearAllHover => "Fjern alle datasett fra behandleren",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Fjern dette datasettet",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} blokker - {}% treffrate",
        DatasetKey::CacheMenuTitle => "Felles hurtigbuffer for alle variabler",
        DatasetKey::GlobalMemoryUsage => "Total bruk av RAM:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Minnebruk etter type:",
        DatasetKey::ResidentHyperslabs => "Hyperslabs i minnet: {} MB ({} blokker)",
        DatasetKey::MultiResPyramid => "Flernivåpyramide: {} MB",
        DatasetKey::CacheHitsMisses => "Treff: {} | Bom: {} (treffrate: {}%)",
        DatasetKey::PrefetchInFlight => "Forhåndshenting i bakgrunnen: {} pågår",
        DatasetKey::AllBlocksCached => "Bufferen er varm / alle blokker i hurtigbufferen",
        DatasetKey::SummaryPerVariable => "Oversikt per variabel:",
        DatasetKey::NoVariablesCached => "(Ingen variabler i hurtigbufferen nå)",
        DatasetKey::VariableCacheSize => "({} MB, {} blokker)",
        DatasetKey::ClearVariableCacheHover => "Tøm hurtigbufferen for denne variabelen",
        DatasetKey::PrefetchCapacitySettings => "Forhåndshenting og kapasitet",
        DatasetKey::EnablePrefetching => "Slå på forhåndshenting i bakgrunnen",
        DatasetKey::CacheLimitSuffix => " MB grense",
        DatasetKey::WindowSizeSuffix => " vindusstørrelse (steg)",
        DatasetKey::PrefetchWorkersSuffix => " forhåndshentere (tråder)",
        DatasetKey::FlushAllCaches => "Tøm alle hurtigbuffere",
        // Catalog
        DatasetKey::CatalogTitle => "Datasettkatalog",
        DatasetKey::CatalogStoreCount => "{} lagre",
        DatasetKey::CatalogBlurb => {
            "Utvalgte Zarr- og Icechunk-datasett i skyen og prosedyriske fasitdatasett. Velg et datasett for å laste det inn."
        }
        DatasetKey::CatalogSearchHint => "Filtrer etter navn, beskrivelse eller URL...",
        DatasetKey::CatalogCategory => "Kategori:",
        DatasetKey::CategoryAll => "Alle",
        DatasetKey::CategoryProcedural => "Prosedyrisk",
        DatasetKey::NoCatalogMatches => "Ingen datasett samsvarer med søket.",
        DatasetKey::ClearSearch => "Tøm søk",
        DatasetKey::SelectAndLoadDataset => "Velg og last inn datasett",
        DatasetKey::SelectAndLoad => "Velg og last inn",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "Typen støttes ikke",
        DatasetKey::DropReleaseToLoad => "Slipp for å laste inn datasettet",
        DatasetKey::DropSupportedList => "Støttes: {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "prøv:",
        DatasetKey::HeroSeasFireHover => {
            "Globalt datasett om skogbrann og klima, omdelt i blokker (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A ekte farger COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Syntetisk 4D-volum i rom og tid",
        DatasetKey::HeroCatalogHover => "Bla gjennom datasettkatalogen",
        DatasetKey::LoadSampleHover => "Last inn eksempel: {}",
        DatasetKey::HeroHintLong => "URL / lokal bane / dra og slipp filer hvor som helst",
        DatasetKey::HeroHintMedium => "URL / bane / dra og slipp filer",
        DatasetKey::HeroHintShort => "URL / slipp filer",
        DatasetKey::HeroDropLong => {
            "Slipp datasett for å laste inn (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Slipp datasett (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Slipp datasett for å laste inn",
        DatasetKey::HeroUnsupportedLong => {
            "Typen støttes ikke. Støttes: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Formatet støttes ikke (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Formatet støttes ikke",
        DatasetKey::HeroInputHintLong => {
            "https://... eller bane (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... eller bane (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL eller bane...",
        DatasetKey::ClearInputHover => "Tøm feltet",
        DatasetKey::LoadDatasetHover => "Last inn datasett og åpne variabler",
        DatasetKey::HopOctantHover => "Klikk for å hoppe til en annen oktant",
        DatasetKey::HeroLoadingSource => "laster inn: {}",
        DatasetKey::HeroLoading => "laster inn...",
        DatasetKey::HeroLoaded => "lastet inn: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "Lokale filbaner kan ikke leses direkte i en nettleser på grunn av nettleserens sikkerhetssandkasse.\n\nSlik viser du lokale datasett i nettleseren:\n1. Server mappen eller filen med en lokal HTTP-server: `npx serve` eller `python3 -m http.server`\n2. Skriv inn URL-en: `http://localhost:8000/my_dataset`\n\nEller kjør den native skrivebordsversjonen av Octant (`cargo run --release`)."
        }
    })
}
