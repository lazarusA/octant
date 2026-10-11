use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Informazioni su Octant",
        AboutKey::IconsWindowTitle => "Icone vettoriali native",
        AboutKey::IconsTab => "Icone vettoriali",
        AboutKey::Subtitle => "Esploratore di dati N-dimensionali",
        AboutKey::Description => {
            "Octant è un visualizzatore interattivo per set di dati N-dimensionali con supporto nativo per archiviazione a oggetti locale e cloud, Zarr (v2/v3) e Icechunk. Sviluppato in Rust con rendering accelerato da GPU tramite WGPU. Octant funziona in modo nativo su macOS, Linux e Windows."
        }
        AboutKey::HyperslabTitle => "Sezionamento iperslab",
        AboutKey::HyperslabDesc => {
            "Cache LRU asincrona dei chunk e piramidi a risoluzione multipla."
        }
        AboutKey::StorageTitle => "Nativo per Zarr e Icechunk",
        AboutKey::StorageDesc => "Backend locali, S3, GCS, Azure e streaming HTTP.",
        AboutKey::ShadersTitle => "Shader WGPU accelerati via hardware",
        AboutKey::ShadersDesc => {
            "Mappe 2D, sfere 3D, superfici di elevazione, volumi e profili 1D."
        }
        AboutKey::LinksHeading => "Collegamenti e risorse",
        AboutKey::DocsLink => "Documentazione di Octant",
        AboutKey::GoodFaithNotice => {
            "Octant riconosce in buona fede il lavoro di terze parti. Se ritieni che manchi qualcosa o che sia attribuito in modo errato, faccelo sapere e lo correggeremo o lo rimuoveremo."
        }
        AboutKey::BrowseIcons => "Esplora icone vettoriali native",
        AboutKey::BrowseIconsHover => {
            "Esplora le icone vettoriali procedurali in una finestra a comparsa"
        }
        AboutKey::Licensed => "Concesso in licenza MIT o Apache-2.0",
        AboutKey::OctantTeaser => {
            "I segni di x, y e z (+/-, +/-, +/-) dividono lo spazio in otto ottanti. Esplora i tuoi dati allo stesso modo: scegli gli assi, taglia una fetta e osserva una regione alla volta."
        }
        AboutKey::OctantHover => "Clicca per passare all'ottante successivo",
        AboutKey::ColormapCredits => "Crediti delle mappe di colore",
        AboutKey::ColormapCreditsNote => {
            "Le mappe di colore incluse mantengono le licenze dei loro autori."
        }
        AboutKey::LicenseTexts => "Testi delle licenze",
        AboutKey::ThirdPartyLicenses => "Licenze di terze parti",
        AboutKey::ThirdPartyNote => {
            "Octant include crate open source, font e librerie C (netCDF, HDF5), ciascuno con la propria licenza."
        }
        AboutKey::ViewOnGithub => "Vedi su GitHub",
        AboutKey::IconsSearchHint => "Cerca icone...",
        AboutKey::IconsAll => "Tutte",
        AboutKey::IconsCopied => "Copiato {}",
        AboutKey::IconsNoMatch => "Nessuna icona vettoriale corrispondente.",
        AboutKey::IconsCategory => "Categoria: {}",
        AboutKey::IconsClickToCopy => "Clicca per copiare l'identificatore dell'enum",
        AboutKey::IconsSizesTones => "Dimensioni e toni",
        AboutKey::ClearSearchHover => "Cancella ricerca",
        AboutKey::FontLicensesNote => "Noto Sans SC, JP e KR (testo cinese, giapponese e coreano):",
    })
}
