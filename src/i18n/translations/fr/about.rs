use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "À propos d'Octant",
        AboutKey::IconsWindowTitle => "Icônes vectorielles natives",
        AboutKey::IconsTab => "Icônes vectorielles",
        AboutKey::Subtitle => "Explorateur de données N-dimensionnelles",
        AboutKey::Description => {
            "Octant est un visualiseur interactif de jeux de données N-dimensionnels avec prise en charge native du stockage objet local et cloud, de Zarr (v2/v3) et d'Icechunk. Développé en Rust avec un rendu accéléré par GPU via WGPU. Octant fonctionne nativement sur macOS, Linux et Windows."
        }
        AboutKey::HyperslabTitle => "Découpage d'hyperslabs",
        AboutKey::HyperslabDesc => "Cache LRU asynchrone de chunks et pyramides multi-résolutions.",
        AboutKey::StorageTitle => "Natif Zarr & Icechunk",
        AboutKey::StorageDesc => "Backends locaux, S3, GCS, Azure et flux HTTP.",
        AboutKey::ShadersTitle => "Shaders WGPU accélérés par le matériel",
        AboutKey::ShadersDesc => {
            "Cartes 2D, sphères 3D, surfaces d'élévation, volumes et profils 1D."
        }
        AboutKey::LinksHeading => "Liens et ressources",
        AboutKey::DocsLink => "Documentation d'Octant",
        AboutKey::GoodFaithNotice => {
            "Octant crédite les travaux de tiers de bonne foi. Si vous pensez qu'un élément manque ou est mal attribué, faites-le-nous savoir et nous le corrigerons ou le retirerons."
        }
        AboutKey::BrowseIcons => "Parcourir les icônes vectorielles natives",
        AboutKey::BrowseIconsHover => {
            "Explorer les icônes vectorielles procédurales dans une fenêtre"
        }
        AboutKey::Licensed => "Sous licence MIT ou Apache-2.0",
        AboutKey::OctantTeaser => {
            "Les signes de x, y et z (+/-, +/-, +/-) divisent l'espace en huit octants. Explorez vos données de la même façon : choisissez les axes, découpez une tranche et observez une région à la fois."
        }
        AboutKey::OctantHover => "Cliquez pour passer à l'octant suivant",
        AboutKey::ColormapCredits => "Crédits des palettes",
        AboutKey::ColormapCreditsNote => {
            "Les palettes intégrées conservent les licences de leurs auteurs."
        }
        AboutKey::LicenseTexts => "Textes des licences",
        AboutKey::ThirdPartyLicenses => "Licences tierces",
        AboutKey::ThirdPartyNote => {
            "Octant inclut des crates, des polices et des bibliothèques C (netCDF, HDF5) open source, chacune sous sa propre licence."
        }
        AboutKey::ViewOnGithub => "Voir sur GitHub",
        AboutKey::IconsSearchHint => "Rechercher des icônes...",
        AboutKey::IconsAll => "Toutes",
        AboutKey::IconsCopied => "{} copié",
        AboutKey::IconsNoMatch => "Aucune icône vectorielle correspondante.",
        AboutKey::IconsCategory => "Catégorie : {}",
        AboutKey::IconsClickToCopy => "Cliquez pour copier l'identifiant de l'enum",
        AboutKey::IconsSizesTones => "Tailles et tons",
        AboutKey::ClearSearchHover => "Effacer la recherche",
        AboutKey::FontLicensesNote => {
            "Noto Sans SC, JP et KR (texte chinois, japonais et coréen) :"
        }
    })
}
