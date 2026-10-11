use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Aktivt dataset",
        DatasetKey::NoDatasetLoaded => "Inget dataset inläst",
        DatasetKey::MemoryCache => "Minne och cache",
        DatasetKey::SampleDatasets => "Exempel",
        DatasetKey::DropFilesHere => "Dra och släpp fil eller mapp",
        DatasetKey::SourceUrlOrPath => "Mål-URL / filsökväg",
        DatasetKey::LoadingMetadata => "Inspekterar lagringsmetadata i bakgrunden...",
        DatasetKey::OpenSource => "Öppna källa",
        DatasetKey::DatasetManager => "Datasethanterare",
        DatasetKey::CloseDatasetHover => "Stäng dataset",
        DatasetKey::TargetUriHover => "Mål-URI:\n{}",
        DatasetKey::VariableCountOne => "{} variabel",
        DatasetKey::VariableCountMany => "{} variabler",
        DatasetKey::InspectVariables => "Inspektera variabler",
        DatasetKey::Reload => "Läs in igen",
        DatasetKey::BadgeVolume4D => "4D-volym",
        DatasetKey::BadgeMatrix2D => "2D-matris",
        DatasetKey::BadgeStore => "Lagring",
        DatasetKey::Load => "Läs in",
        DatasetKey::TargetPathHint => "https://... eller sökväg (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Katalog...",
        DatasetKey::Clear => "Rensa",
        DatasetKey::FormatLabel => "Format:",
        DatasetKey::SampleSeasFire => "SeasFire globala skogsbränder (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Procedurell 2D-matris (test)",
        DatasetKey::FormatRemoteZarr => "Fjärr-Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Lokal Zarr (filsystem)",
        DatasetKey::FormatRemoteIcechunk => "Fjärr-Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Lokal Icechunk (filsystem)",
        DatasetKey::FormatRemoteGeoTiff => "Fjärr-GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "Lokal GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "Lokal NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "4D-volym med känt facit (procedurell)",
        DatasetKey::FormatMatrix2D => "Procedurell 2D-matris (procedurell)",
        DatasetKey::FormatRemoteGeoTiffShort => "Fjärr-GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "Lokal GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "Lokal NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "4D-volym med känt facit",
        DatasetKey::FormatMatrix2DShort => "Procedurell 2D-matris",
        DatasetKey::NoDatasetsRegistered => "Inga dataset registrerade i hanteraren.",
        DatasetKey::LoadedCount => "Inlästa ({}):",
        DatasetKey::ClearAll => "Rensa alla",
        DatasetKey::ClearAllHover => "Ta bort alla dataset från hanteraren",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Ta bort detta dataset",
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} block - {}% träffar",
        DatasetKey::CacheMenuTitle => "Gemensam minnescache för alla variabler",
        DatasetKey::GlobalMemoryUsage => "Total RAM-användning:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Minnesanvändning per typ:",
        DatasetKey::ResidentHyperslabs => "Inlästa hyperslabs: {} MB ({} block)",
        DatasetKey::MultiResPyramid => "Flerupplöst pyramid: {} MB",
        DatasetKey::CacheHitsMisses => "Cacheträffar: {} | Missar: {} (träffgrad: {}%)",
        DatasetKey::PrefetchInFlight => "Förhämtning i bakgrunden: {} pågående",
        DatasetKey::AllBlocksCached => "Bufferten är varm / alla block cachade",
        DatasetKey::SummaryPerVariable => "Sammanfattning per variabel:",
        DatasetKey::NoVariablesCached => "(Inga variabler cachade i minnet)",
        DatasetKey::VariableCacheSize => "({} MB, {} block)",
        DatasetKey::ClearVariableCacheHover => "Rensa cachen för denna variabel",
        DatasetKey::PrefetchCapacitySettings => "Förhämtning och kapacitet",
        DatasetKey::EnablePrefetching => "Aktivera förhämtning i bakgrunden",
        DatasetKey::CacheLimitSuffix => " MB gräns",
        DatasetKey::WindowSizeSuffix => " fönsterstorlek (steg)",
        DatasetKey::PrefetchWorkersSuffix => " förhämtningsarbetare (trådar)",
        DatasetKey::FlushAllCaches => "Töm och rensa alla cachar",
        DatasetKey::CatalogTitle => "Datasetkatalog",
        DatasetKey::CatalogStoreCount => "{} lagringar",
        DatasetKey::CatalogBlurb => {
            "Utvalda Zarr- och Icechunk-dataset i molnet samt procedurella dataset med känt facit. Välj ett dataset för att läsa in det."
        }
        DatasetKey::CatalogSearchHint => "Filtrera på namn, beskrivning eller URL...",
        DatasetKey::CatalogCategory => "Kategori:",
        DatasetKey::CategoryAll => "Alla",
        DatasetKey::CategoryProcedural => "Procedurella",
        DatasetKey::NoCatalogMatches => "Inga dataset matchar din sökning.",
        DatasetKey::ClearSearch => "Rensa sökning",
        DatasetKey::SelectAndLoadDataset => "Välj och läs in dataset",
        DatasetKey::SelectAndLoad => "Välj och läs in",
        DatasetKey::DropTypeNotSupported => "Typen stöds inte",
        DatasetKey::DropReleaseToLoad => "Släpp för att läsa in dataset",
        DatasetKey::DropSupportedList => "Stöds: {}",
        DatasetKey::HeroTryPrefix => "prova:",
        DatasetKey::HeroSeasFireHover => {
            "Globalt dataset om skogsbränder och klimat, omchunkat (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A äkta färg COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Syntetisk 4D-volym i rum och tid",
        DatasetKey::HeroCatalogHover => "Bläddra i datasetkatalogen",
        DatasetKey::LoadSampleHover => "Läs in exempel: {}",
        DatasetKey::HeroHintLong => {
            "klistra in URL, lokal sökväg eller dra och släpp filer var som helst"
        }
        DatasetKey::HeroHintMedium => "klistra in URL, sökväg eller dra och släpp filer",
        DatasetKey::HeroHintShort => "klistra in URL eller släpp filer",
        DatasetKey::HeroDropLong => {
            "Släpp dataset för att läsa in (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Släpp dataset (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Släpp dataset för att läsa in",
        DatasetKey::HeroUnsupportedLong => {
            "Typen stöds inte. Stöds: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Formatet stöds inte (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Formatet stöds inte",
        DatasetKey::HeroInputHintLong => {
            "https://... eller sökväg (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... eller sökväg (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL eller sökväg...",
        DatasetKey::ClearInputHover => "Rensa fältet",
        DatasetKey::LoadDatasetHover => "Läs in dataset och öppna variabler",
        DatasetKey::HopOctantHover => "Klicka för att hoppa till nästa oktant",
        DatasetKey::HeroLoadingSource => "laddar: {}",
        DatasetKey::HeroLoading => "laddar...",
        DatasetKey::HeroLoaded => "inläst: {}",
        DatasetKey::BrowserLocalPaths => {
            "Lokala filsökvägar kan inte läsas direkt i en webbläsare på grund av webbens sandlådesäkerhet.\n\nSå här visar du lokala dataset i webbläsaren:\n1. Servera mappen eller filen med en lokal HTTP-server: `npx serve` eller `python3 -m http.server`\n2. Ange URL:en: `http://localhost:8000/my_dataset`\n\nEller kör den inbyggda skrivbordsversionen av Octant (`cargo run --release`)."
        }
    })
}
