//! French (Français) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Jeu de données",
        Key::Variables => "Variables",
        Key::Dimensions => "Dimensions",
        Key::Plot => "Tracé",
        Key::Colormaps => "Palettes de couleurs",
        Key::Settings => "Paramètres",
        Key::About => "À propos",

        Key::InspectMetadata => "Inspecter les métadonnées",
        Key::ActiveDatasets => "Jeux de données actifs",
        Key::NoActiveDatasets => "Aucun jeu actif. Ouvrez un jeu de données pour commencer.",
        Key::CacheUsage => "Utilisation du cache",
        Key::ClearCache => "Vider le cache",
        Key::OpenLocal => "Ouvrir un fichier local",
        Key::CloudStore => "Stockage objet cloud",
        Key::SampleDatasets => "Jeux d'échantillons",
        Key::DropFilesHere => "Déposez des fichiers ou dossiers ici",
        Key::SourceUrlOrPath => "URI de stockage ou chemin local",
        Key::Inspect => "Inspecter",
        Key::LoadingMetadata => "Inspection des métadonnées en arrière-plan...",

        Key::Group => "Groupe :",
        Key::DataType => "Type",
        Key::Shape => "Forme",
        Key::Description => "Description",
        Key::SearchVariables => "Rechercher des variables...",
        Key::NoVariablesMatching => "Aucune variable ne correspond à la requête",
        Key::ShowingItemsOfTotal => "Affichage des éléments sur le total",

        Key::Ready => "Prêt",
        Key::Loading => "Chargement...",
        Key::Close => "Fermer",
        Key::Cancel => "Annuler",
        Key::Save => "Enregistrer",
        Key::Reset => "Réinitialiser",
        Key::Error => "Erreur",

        Key::Play => "Lire",
        Key::Pause => "Pause",
        Key::StepForward => "Pas en avant",
        Key::StepBackward => "Pas en arrière",
        Key::Speed => "Vitesse",
        Key::Loop => "Boucle",

        Key::AboutSubtitle => "Explorateur de données N-dimensionnelles",
        Key::AboutDescription => {
            "Octant est un visualiseur interactif de données N-dimensionnelles avec support natif pour le stockage local et cloud, Zarr (v2/v3) et Icechunk. Développé en Rust avec rendu accéléré par GPU via WGPU. Fonctionne sur macOS, Linux et Windows."
        }
        Key::AboutHyperslabTitle => "Découpage d'hyperslabs",
        Key::AboutHyperslabDesc => "Cache LRU asynchrone et pyramides multi-résolutions.",
        Key::AboutStorageTitle => "Natif Zarr & Icechunk",
        Key::AboutStorageDesc => "Backends locaux, S3, GCS, Azure et flux HTTP.",
        Key::AboutShadersTitle => "Shaders WGPU accélérés par le matériel",
        Key::AboutShadersDesc => {
            "Cartes 2D, sphères 3D, surfaces d'élévation, volumes et profils 1D."
        }
        Key::AboutLinksHeading => "Liens et ressources",
        Key::AboutGoodFaithNotice => {
            "Octant mentionne les œuvres tierces de bonne foi. Si vous constatez une omission ou une erreur, contactez-nous pour correction."
        }
        Key::AboutBrowseIcons => "Parcourir les icônes vectorielles natives",
    })
}
