use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Conjunto de datos activo",
        DatasetKey::NoDatasetLoaded => "No hay ningún conjunto de datos cargado",
        DatasetKey::MemoryCache => "Memoria y caché",
        DatasetKey::SampleDatasets => "Muestras",
        DatasetKey::DropFilesHere => "Arrastre un archivo o directorio",
        DatasetKey::SourceUrlOrPath => "URL de destino / ruta de archivo",
        DatasetKey::LoadingMetadata => "Inspeccionando metadatos del almacén en segundo plano...",
        // Dataset panel
        DatasetKey::OpenSource => "Abrir fuente",
        DatasetKey::DatasetManager => "Gestor de conjuntos de datos",
        DatasetKey::CloseDatasetHover => "Cerrar conjunto de datos",
        // Active dataset card
        DatasetKey::TargetUriHover => "URI de destino:\n{}",
        DatasetKey::VariableCountOne => "{} variable",
        DatasetKey::VariableCountMany => "{} variables",
        DatasetKey::InspectVariables => "Inspeccionar variables",
        DatasetKey::Reload => "Recargar",
        DatasetKey::BadgeVolume4D => "Volumen 4D",
        DatasetKey::BadgeMatrix2D => "Matriz 2D",
        DatasetKey::BadgeStore => "Almacén",
        // Source intake
        DatasetKey::Load => "Cargar",
        DatasetKey::TargetPathHint => "https://... o ruta (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Catálogo...",
        DatasetKey::Clear => "Borrar",
        DatasetKey::FormatLabel => "Formato:",
        DatasetKey::SampleSeasFire => "SeasFire: incendios globales (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Matriz procedural 2D (prueba)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "Zarr remoto (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Zarr local (sistema de archivos)",
        DatasetKey::FormatRemoteIcechunk => "Icechunk remoto (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Icechunk local (sistema de archivos)",
        DatasetKey::FormatRemoteGeoTiff => "GeoTIFF / COG remoto (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "GeoTIFF local (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "NetCDF / HDF5 local (.nc/.h5)",
        DatasetKey::FormatVolume4D => "Volumen 4D de referencia (procedural)",
        DatasetKey::FormatMatrix2D => "Matriz procedural 2D (procedural)",
        DatasetKey::FormatRemoteGeoTiffShort => "GeoTIFF / COG remoto",
        DatasetKey::FormatLocalGeoTiffShort => "GeoTIFF local",
        DatasetKey::FormatLocalNetCdfShort => "NetCDF / HDF5 local",
        DatasetKey::FormatVolume4DShort => "Volumen 4D de referencia",
        DatasetKey::FormatMatrix2DShort => "Matriz procedural 2D",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "No hay conjuntos de datos registrados en el gestor.",
        DatasetKey::LoadedCount => "Cargados ({}):",
        DatasetKey::ClearAll => "Borrar todo",
        DatasetKey::ClearAllHover => "Quitar todos los conjuntos de datos del gestor",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Quitar este conjunto de datos",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} bloques, {}% de aciertos",
        DatasetKey::CacheMenuTitle => "Caché de memoria unificada multivariable",
        DatasetKey::GlobalMemoryUsage => "Uso global de memoria RAM:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Desglose de memoria por tipo:",
        DatasetKey::ResidentHyperslabs => "Hiperbloques residentes: {} MB ({} bloques)",
        DatasetKey::MultiResPyramid => "Pirámide multirresolución: {} MB",
        DatasetKey::CacheHitsMisses => "Aciertos: {} | Fallos: {} (tasa de aciertos: {}%)",
        DatasetKey::PrefetchInFlight => "Precarga en segundo plano: {} en curso",
        DatasetKey::AllBlocksCached => "Búfer listo / todos los bloques en caché",
        DatasetKey::SummaryPerVariable => "Resumen por variable:",
        DatasetKey::NoVariablesCached => "(Ninguna variable en la caché de memoria)",
        DatasetKey::VariableCacheSize => "({} MB, {} bloq.)",
        DatasetKey::ClearVariableCacheHover => "Borrar la caché de esta variable",
        DatasetKey::PrefetchCapacitySettings => "Ajustes de precarga y capacidad",
        DatasetKey::EnablePrefetching => "Activar precarga en segundo plano",
        DatasetKey::CacheLimitSuffix => " MB de límite",
        DatasetKey::WindowSizeSuffix => " Tamaño de ventana (pasos)",
        DatasetKey::PrefetchWorkersSuffix => " Trabajadores de precarga (hilos)",
        DatasetKey::FlushAllCaches => "Vaciar todas las cachés",
        // Catalog
        DatasetKey::CatalogTitle => "Catálogo de conjuntos de datos",
        DatasetKey::CatalogStoreCount => "{} almacenes",
        DatasetKey::CatalogBlurb => {
            "Conjuntos de datos Zarr e Icechunk en la nube y conjuntos procedurales de referencia seleccionados. Elija cualquiera para cargarlo."
        }
        DatasetKey::CatalogSearchHint => "Filtrar por nombre, descripción o URL...",
        DatasetKey::CatalogCategory => "Categoría:",
        DatasetKey::CategoryAll => "Todos",
        DatasetKey::CategoryProcedural => "Procedural",
        DatasetKey::NoCatalogMatches => "Ningún conjunto de datos coincide con su búsqueda.",
        DatasetKey::ClearSearch => "Borrar búsqueda",
        DatasetKey::SelectAndLoadDataset => "Seleccionar y cargar conjunto de datos",
        DatasetKey::SelectAndLoad => "Seleccionar y cargar",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "Tipo no compatible",
        DatasetKey::DropReleaseToLoad => "Suelte para cargar el conjunto de datos",
        DatasetKey::DropSupportedList => "Compatibles: {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "pruebe:",
        DatasetKey::HeroSeasFireHover => {
            "Conjunto de datos global de incendios y clima, refragmentado (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A en color verdadero, COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Volumen espaciotemporal 4D sintético",
        DatasetKey::HeroCatalogHover => "Explorar el catálogo de conjuntos de datos",
        DatasetKey::LoadSampleHover => "Cargar muestra: {}",
        DatasetKey::HeroHintLong => {
            "pegue una URL o ruta local, o arrastre y suelte archivos en cualquier lugar"
        }
        DatasetKey::HeroHintMedium => "pegue una URL o ruta, o arrastre archivos",
        DatasetKey::HeroHintShort => "pegue una URL o suelte archivos",
        DatasetKey::HeroDropLong => {
            "Suelte el conjunto de datos para cargarlo (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Suelte el conjunto de datos (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Suelte para cargar",
        DatasetKey::HeroUnsupportedLong => {
            "Tipo no compatible. Compatibles: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Formato no compatible (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Formato no compatible",
        DatasetKey::HeroInputHintLong => {
            "https://... o ruta (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... o ruta (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL o ruta...",
        DatasetKey::ClearInputHover => "Borrar entrada",
        DatasetKey::LoadDatasetHover => "Cargar conjunto de datos y abrir variables",
        DatasetKey::HopOctantHover => "Haga clic para saltar de octante",
        DatasetKey::HeroLoadingSource => "cargando: {}",
        DatasetKey::HeroLoading => "cargando...",
        DatasetKey::HeroLoaded => "cargado: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "Las rutas de archivos locales no se pueden leer en un navegador debido al aislamiento de seguridad web.\n\nPara ver conjuntos de datos locales en el navegador:\n1. Sirva su directorio o archivo con un servidor HTTP local: `npx serve` o `python3 -m http.server`\n2. Introduzca la URL: `http://localhost:8000/my_dataset`\n\nO ejecute la versión nativa de escritorio de Octant (`cargo run --release`)."
        }
    })
}
