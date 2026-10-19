use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Jeu de données actif",
        DatasetKey::NoDatasetLoaded => "Aucun jeu de données chargé",
        DatasetKey::MemoryCache => "Mémoire et cache",
        DatasetKey::SampleDatasets => "Exemples",
        DatasetKey::DropFilesHere => "Glisser-déposer un fichier ou un dossier",
        DatasetKey::SourceUrlOrPath => "URL cible / chemin de fichier",
        DatasetKey::LoadingMetadata => "Inspection des métadonnées en arrière-plan...",
        // Dataset panel
        DatasetKey::OpenSource => "Ouvrir une source",
        DatasetKey::DatasetManager => "Gestionnaire de jeux de données",
        DatasetKey::CloseDatasetHover => "Fermer le jeu de données",
        // Active dataset card
        DatasetKey::TargetUriHover => "URI cible :\n{}",
        DatasetKey::VariableCountOne => "{} variable",
        DatasetKey::VariableCountMany => "{} variables",
        DatasetKey::InspectVariables => "Inspecter les variables",
        DatasetKey::Reload => "Recharger",
        DatasetKey::BadgeVolume4D => "Volume 4D",
        DatasetKey::BadgeMatrix2D => "Matrice 2D",
        DatasetKey::BadgeStore => "Dépôt",
        // Source intake
        DatasetKey::Load => "Charger",
        DatasetKey::TargetPathHint => "https://... ou chemin (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Catalogue...",
        DatasetKey::Clear => "Effacer",
        DatasetKey::FormatLabel => "Format :",
        DatasetKey::SampleSeasFire => "SeasFire, feux de forêt mondiaux (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Matrice procédurale 2D (test)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "Zarr distant (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Zarr local (système de fichiers)",
        DatasetKey::FormatRemoteIcechunk => "Icechunk distant (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Icechunk local (système de fichiers)",
        DatasetKey::FormatRemoteGeoTiff => "GeoTIFF / COG distant (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "GeoTIFF local (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "NetCDF / HDF5 local (.nc/.h5)",
        DatasetKey::FormatVolume4D => "Volume 4D de vérité connue (procédural)",
        DatasetKey::FormatMatrix2D => "Matrice procédurale 2D (procédural)",
        DatasetKey::FormatRemoteGeoTiffShort => "GeoTIFF / COG distant",
        DatasetKey::FormatLocalGeoTiffShort => "GeoTIFF local",
        DatasetKey::FormatLocalNetCdfShort => "NetCDF / HDF5 local",
        DatasetKey::FormatVolume4DShort => "Volume 4D de vérité connue",
        DatasetKey::FormatMatrix2DShort => "Matrice procédurale 2D",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "Aucun jeu de données enregistré dans le gestionnaire.",
        DatasetKey::LoadedCount => "Chargés ({}) :",
        DatasetKey::ClearAll => "Tout effacer",
        DatasetKey::ClearAllHover => "Retirer tous les jeux de données du gestionnaire",
        DatasetKey::DatasetRowHover => "{}\nURI : {}",
        DatasetKey::RemoveDatasetHover => "Retirer ce jeu de données",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} blocs - {}% de succès",
        DatasetKey::CacheMenuTitle => "Cache mémoire unifié multi-variables",
        DatasetKey::GlobalMemoryUsage => "Utilisation globale de la RAM :",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Répartition de la mémoire par type :",
        DatasetKey::ResidentHyperslabs => "Hyperslabs résidents : {} MB ({} blocs)",
        DatasetKey::MultiResPyramid => "Pyramide multi-résolutions : {} MB",
        DatasetKey::CacheHitsMisses => "Succès du cache : {} | Échecs : {} (taux de succès : {}%)",
        DatasetKey::PrefetchInFlight => "Préchargement en arrière-plan : {} en cours",
        DatasetKey::AllBlocksCached => "Tampon prêt / tous les blocs en cache",
        DatasetKey::SummaryPerVariable => "Résumé par variable :",
        DatasetKey::NoVariablesCached => "(Aucune variable actuellement en cache mémoire)",
        DatasetKey::VariableCacheSize => "({} MB, {} blocs)",
        DatasetKey::ClearVariableCacheHover => "Vider le cache de cette variable",
        DatasetKey::PrefetchCapacitySettings => "Préchargement et capacité",
        DatasetKey::EnablePrefetching => "Activer le préchargement en arrière-plan",
        DatasetKey::CacheLimitSuffix => " MB de limite",
        DatasetKey::WindowSizeSuffix => " taille de fenêtre (pas)",
        DatasetKey::PrefetchWorkersSuffix => " workers de préchargement (threads)",
        DatasetKey::FlushAllCaches => "Vider tous les caches",
        // Catalog
        DatasetKey::CatalogTitle => "Catalogue de jeux de données",
        DatasetKey::CatalogStoreCount => "{} dépôts",
        DatasetKey::CatalogBlurb => {
            "Jeux de données Zarr et Icechunk dans le cloud et jeux procéduraux de vérité terrain, sélectionnés avec soin. Sélectionnez un jeu de données pour le charger."
        }
        DatasetKey::CatalogSearchHint => "Filtrer par nom, description ou URL...",
        DatasetKey::CatalogCategory => "Catégorie :",
        DatasetKey::CategoryAll => "Tous",
        DatasetKey::CategoryProcedural => "Procédural",
        DatasetKey::NoCatalogMatches => "Aucun jeu de données ne correspond à votre recherche.",
        DatasetKey::ClearSearch => "Effacer la recherche",
        DatasetKey::SelectAndLoadDataset => "Sélectionner et charger le jeu de données",
        DatasetKey::SelectAndLoad => "Sélectionner et charger",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "Type non pris en charge",
        DatasetKey::DropReleaseToLoad => "Relâchez pour charger le jeu de données",
        DatasetKey::DropSupportedList => "Pris en charge : {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "essayez :",
        DatasetKey::HeroSeasFireHover => "Jeu de données mondial feux et climat, rechunké (Zarr)",
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A couleurs vraies COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Volume spatio-temporel 4D synthétique",
        DatasetKey::HeroCatalogHover => "Parcourir le catalogue de jeux de données",
        DatasetKey::LoadSampleHover => "Charger l'exemple : {}",
        DatasetKey::HeroHintLong => {
            "URL / chemin local / glissez-déposez des fichiers n'importe où"
        }
        DatasetKey::HeroHintMedium => "URL / chemin / glissez-déposez des fichiers",
        DatasetKey::HeroHintShort => "URL / déposez des fichiers",
        DatasetKey::HeroDropLong => {
            "Déposez un jeu de données à charger (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Déposez un jeu de données (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Déposez un jeu de données à charger",
        DatasetKey::HeroUnsupportedLong => {
            "Type non pris en charge. Pris en charge : .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => {
            "Format non pris en charge (.nc, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroUnsupportedShort => "Format non pris en charge",
        DatasetKey::HeroInputHintLong => {
            "https://... ou chemin (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... ou chemin (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL ou chemin...",
        DatasetKey::ClearInputHover => "Effacer la saisie",
        DatasetKey::LoadDatasetHover => "Charger le jeu de données et ouvrir les variables",
        DatasetKey::HopOctantHover => "Cliquez pour changer d'octant",
        DatasetKey::HeroLoadingSource => "chargement : {}",
        DatasetKey::HeroLoading => "chargement...",
        DatasetKey::HeroLoaded => "chargé : {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "Les chemins de fichiers locaux ne peuvent pas être lus directement dans un navigateur en raison du bac à sable de sécurité du web.\n\nPour afficher des jeux de données locaux dans le navigateur :\n1. Servez votre dossier ou fichier avec un serveur HTTP local : `npx serve` ou `python3 -m http.server`\n2. Saisissez l'URL : `http://localhost:8000/my_dataset`\n\nOu lancez la version de bureau native d'Octant (`cargo run --release`)."
        }
        DatasetKey::Plate => "Plaque",
        DatasetKey::PlateAcquisition => "Acquisition",
        DatasetKey::PlateAllAcquisitions => "Toutes",
        DatasetKey::PlateFields => "Champs :",
        DatasetKey::PlatePickWell => "Choisissez un puits pour voir ses champs",
        DatasetKey::PlateLoadingFields => "Lecture des champs du puits",
        DatasetKey::PlateNoFields => "Aucun champ dans cette acquisition",
        DatasetKey::PlateWellUnreadable => {
            "Impossible de lire les champs du puits. Cliquez pour réessayer"
        }
        DatasetKey::CategoryMicroscopy => "Microscopie",
        DatasetKey::CategorySoil => "Sol",
        DatasetKey::BadgeDesktopOnly => "Bureau uniquement",
        DatasetKey::DesktopOnlyHint => {
            "Ce serveur n'autorise pas l'accès depuis le navigateur. Ouvrez-le dans l'application de bureau Octant."
        }
    })
}
