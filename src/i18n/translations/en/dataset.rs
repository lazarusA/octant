use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> &'static str {
    match key {
        DatasetKey::ActiveDataset => "Active Dataset",
        DatasetKey::NoDatasetLoaded => "No dataset currently loaded",
        DatasetKey::MemoryCache => "Memory & Cache",
        DatasetKey::SampleDatasets => "Samples",
        DatasetKey::DropFilesHere => "Drag & Drop File or Directory",
        DatasetKey::SourceUrlOrPath => "Target URL / File Path",
        DatasetKey::LoadingMetadata => "Inspecting store metadata in background...",
        // Dataset panel
        DatasetKey::OpenSource => "Open Source",
        DatasetKey::DatasetManager => "Dataset Manager",
        DatasetKey::CloseDatasetHover => "Close Dataset",
        // Active dataset card
        DatasetKey::TargetUriHover => "Target URI:\n{}",
        DatasetKey::VariableCountOne => "{} variable",
        DatasetKey::VariableCountMany => "{} variables",
        DatasetKey::InspectVariables => "Inspect Variables",
        DatasetKey::Reload => "Reload",
        DatasetKey::BadgeVolume4D => "4D Volume",
        DatasetKey::BadgeMatrix2D => "2D Matrix",
        DatasetKey::BadgeStore => "Store",
        // Source intake
        DatasetKey::Load => "Load",
        DatasetKey::TargetPathHint => "https://... or path (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Catalog...",
        DatasetKey::Clear => "Clear",
        DatasetKey::FormatLabel => "Format:",
        DatasetKey::SampleSeasFire => "SeasFire Global Wildfire (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "2D Procedural Matrix (Test)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "Remote Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Local Zarr (Filesystem)",
        DatasetKey::FormatRemoteIcechunk => "Remote Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Local Icechunk (Filesystem)",
        DatasetKey::FormatRemoteGeoTiff => "Remote GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "Local GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "Local NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "4D Known-Truth Volume (Procedural)",
        DatasetKey::FormatMatrix2D => "2D Procedural Matrix (Procedural)",
        DatasetKey::FormatRemoteGeoTiffShort => "Remote GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "Local GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "Local NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "4D Known-Truth Volume",
        DatasetKey::FormatMatrix2DShort => "2D Procedural Matrix",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "No datasets currently registered in manager.",
        DatasetKey::LoadedCount => "Loaded ({}):",
        DatasetKey::ClearAll => "Clear All",
        DatasetKey::ClearAllHover => "Remove all datasets from manager",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Remove this dataset",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} blocks | {}% hit rate",
        DatasetKey::CacheMenuTitle => "Unified Multi-Variable Memory Cache",
        DatasetKey::GlobalMemoryUsage => "Global RAM Memory Usage:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Memory Breakdown by Kind:",
        DatasetKey::ResidentHyperslabs => "Resident Hyperslabs: {} MB ({} blocks)",
        DatasetKey::MultiResPyramid => "Multi-Res Pyramid: {} MB",
        DatasetKey::CacheHitsMisses => "Cache Hits: {} | Misses: {} (Hit Rate: {}%)",
        DatasetKey::PrefetchInFlight => "Background Prefetching: {} in-flight",
        DatasetKey::AllBlocksCached => "Buffer Warm / All Blocks Cached",
        DatasetKey::SummaryPerVariable => "Summary per Variable:",
        DatasetKey::NoVariablesCached => "(No variables currently cached in memory)",
        DatasetKey::VariableCacheSize => "({} MB, {} blks)",
        DatasetKey::ClearVariableCacheHover => "Clear cache for this variable",
        DatasetKey::PrefetchCapacitySettings => "Prefetch & Capacity Settings",
        DatasetKey::EnablePrefetching => "Enable Background Prefetching",
        DatasetKey::CacheLimitSuffix => " MB Limit",
        DatasetKey::WindowSizeSuffix => " Window Size (steps)",
        DatasetKey::PrefetchWorkersSuffix => " Prefetch Workers (threads)",
        DatasetKey::FlushAllCaches => "Flush & Clear All Caches",
        // Catalog
        DatasetKey::CatalogTitle => "Dataset Catalog",
        DatasetKey::CatalogStoreCount => "{} stores",
        DatasetKey::CatalogBlurb => {
            "Curated cloud Zarr, Icechunk, and procedural ground-truth datasets. Select any dataset to load."
        }
        DatasetKey::CatalogSearchHint => "Filter by name, description, or URL...",
        DatasetKey::CatalogCategory => "Category:",
        DatasetKey::CategoryAll => "All",
        DatasetKey::CategoryProcedural => "Procedural",
        DatasetKey::NoCatalogMatches => "No datasets found matching your search.",
        DatasetKey::ClearSearch => "Clear Search",
        DatasetKey::SelectAndLoadDataset => "Select & Load Dataset",
        DatasetKey::SelectAndLoad => "Select & Load",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "Type not supported",
        DatasetKey::DropReleaseToLoad => "Release to load dataset",
        DatasetKey::DropSupportedList => "Supported: {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "try:",
        DatasetKey::HeroSeasFireHover => "Global wildfire & climate rechunked dataset (Zarr)",
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A True Color COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Synthetic 4D spatiotemporal volume",
        DatasetKey::HeroCatalogHover => "Browse dataset catalog",
        DatasetKey::LoadSampleHover => "Load sample: {}",
        DatasetKey::HeroHintLong => "URL / local path / drag & drop files anywhere",
        DatasetKey::HeroHintMedium => "URL / path / drag & drop files",
        DatasetKey::HeroHintShort => "URL / drop files",
        DatasetKey::HeroDropLong => "Drop dataset to load (.nc, .h5, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropMedium => "Drop dataset (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Drop dataset to load",
        DatasetKey::HeroUnsupportedLong => {
            "Unsupported type. Supported: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Unsupported format (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Unsupported format",
        DatasetKey::HeroInputHintLong => {
            "https://... or path (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... or path (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL or path...",
        DatasetKey::ClearInputHover => "Clear input",
        DatasetKey::LoadDatasetHover => "Load Dataset & Open Variables",
        DatasetKey::HopOctantHover => "Click to hop octant",
        DatasetKey::HeroLoadingSource => "loading: {}",
        DatasetKey::HeroLoading => "loading...",
        DatasetKey::HeroLoaded => "loaded: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "Direct local file paths cannot be read in a browser due to web sandbox security.\n\nTo view local datasets in the browser:\n1. Serve your directory or file with a local HTTP server: `npx serve` or `python3 -m http.server`\n2. Enter the URL: `http://localhost:8000/my_dataset`\n\nOr run the native desktop version of Octant (`cargo run --release`)."
        }
        DatasetKey::Plate => "Plate",
        DatasetKey::PlateAcquisition => "Acquisition",
        DatasetKey::PlateAllAcquisitions => "All",
        DatasetKey::PlateFields => "Fields:",
        DatasetKey::PlatePickWell => "Pick a well to list its fields",
        DatasetKey::PlateLoadingFields => "Reading the well's fields",
        DatasetKey::PlateNoFields => "No fields in this acquisition",
        DatasetKey::PlateWellUnreadable => "Couldn't read the well's fields. Click to try again",
        DatasetKey::CategoryMicroscopy => "Microscopy",
    }
}
