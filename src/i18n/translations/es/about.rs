use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Acerca de Octant",
        AboutKey::IconsWindowTitle => "Iconos vectoriales nativos",
        AboutKey::IconsTab => "Iconos vectoriales",
        AboutKey::Subtitle => "Explorador de datos N-dimensionales",
        AboutKey::Description => {
            "Octant es un visor interactivo de conjuntos de datos N-dimensionales con soporte nativo para almacenamiento local y de objetos en la nube, Zarr (v2/v3) e Icechunk. Construido en Rust con renderizado acelerado por GPU mediante WGPU. Octant se ejecuta de forma nativa en macOS, Linux y Windows."
        }
        AboutKey::HyperslabTitle => "Corte de hyperslabs",
        AboutKey::HyperslabDesc => "Caché asíncrona LRU y pirámides multirresolución.",
        AboutKey::StorageTitle => "Nativo en Zarr e Icechunk",
        AboutKey::StorageDesc => "Backends locales, S3, GCS, Azure y transmisión HTTP.",
        AboutKey::ShadersTitle => "Sombreadores WGPU acelerados por hardware",
        AboutKey::ShadersDesc => {
            "Mapas 2D, esferas 3D, superficies de elevación, volúmenes y perfiles 1D."
        }
        AboutKey::LinksHeading => "Enlaces y recursos",
        AboutKey::DocsLink => "Documentación de Octant",
        AboutKey::GoodFaithNotice => {
            "Octant reconoce el trabajo de terceros de buena fe. Si considera que algo falta o está incorrectamente atribuido, contáctenos para corregirlo o retirarlo."
        }
        AboutKey::BrowseIcons => "Explorar iconos vectoriales nativos",
        AboutKey::BrowseIconsHover => "Explorar los iconos vectoriales procedurales en una ventana",
        AboutKey::Licensed => "Licencia MIT o Apache-2.0",
        AboutKey::OctantTeaser => {
            "Los signos de x, y y z (+/-, +/-, +/-) dividen el espacio en ocho octantes. Explore sus datos de la misma manera: elija los ejes, corte un bloque y observe una región a la vez."
        }
        AboutKey::OctantHover => "Haga clic para ir al siguiente octante",
        AboutKey::ColormapCredits => "Créditos de los mapas de color",
        AboutKey::ColormapCreditsNote => {
            "Los mapas de color incluidos conservan las licencias de sus autores."
        }
        AboutKey::LicenseTexts => "Textos de licencia",
        AboutKey::ThirdPartyLicenses => "Licencias de terceros",
        AboutKey::ThirdPartyNote => {
            "Octant incluye crates de código abierto, fuentes y bibliotecas C (netCDF, HDF5), cada uno con su propia licencia."
        }
        AboutKey::ViewOnGithub => "Ver en GitHub",
        AboutKey::IconsSearchHint => "Buscar iconos...",
        AboutKey::IconsAll => "Todos",
        AboutKey::IconsCopied => "Copiado: {}",
        AboutKey::IconsNoMatch => "No se encontraron iconos vectoriales coincidentes.",
        AboutKey::IconsCategory => "Categoría: {}",
        AboutKey::IconsClickToCopy => "Haga clic para copiar el identificador del enum",
        AboutKey::IconsSizesTones => "Tamaños y tonos",
        AboutKey::ClearSearchHover => "Borrar búsqueda",
        AboutKey::FontLicensesNote => "Noto Sans SC, JP y KR (texto en chino, japonés y coreano):",
    })
}
