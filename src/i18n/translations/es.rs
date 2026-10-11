//! Spanish (Español) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Conjunto de datos",
        Key::Variables => "Variables",
        Key::Dimensions => "Dimensiones",
        Key::Plot => "Gráfico",
        Key::Colormaps => "Mapas de color",
        Key::Settings => "Configuración",
        Key::About => "Acerca de",

        Key::InspectMetadata => "Inspeccionar metadatos",
        Key::ActiveDatasets => "Conjuntos de datos activos",
        Key::NoActiveDatasets => {
            "No hay conjuntos activos. Abra un conjunto de datos para comenzar."
        }
        Key::CacheUsage => "Uso de caché",
        Key::ClearCache => "Limpiar caché",
        Key::OpenLocal => "Abrir archivo local",
        Key::CloudStore => "Almacén en la nube",
        Key::SampleDatasets => "Conjuntos de muestra",
        Key::DropFilesHere => "Arrastre archivos o carpetas aquí",
        Key::SourceUrlOrPath => "URI del almacén o ruta local",
        Key::Inspect => "Inspeccionar",
        Key::LoadingMetadata => "Inspeccionando metadatos en segundo plano...",

        Key::Group => "Grupo:",
        Key::DataType => "Tipo",
        Key::Shape => "Forma",
        Key::Description => "Descripción",
        Key::SearchVariables => "Buscar variables...",
        Key::NoVariablesMatching => "No hay variables que coincidan con la búsqueda",
        Key::ShowingItemsOfTotal => "Mostrando elementos del total",

        Key::Ready => "Listo",
        Key::Loading => "Cargando...",
        Key::Close => "Cerrar",
        Key::Cancel => "Cancelar",
        Key::Save => "Guardar",
        Key::Reset => "Restablecer",
        Key::Error => "Error",

        Key::Play => "Reproducir",
        Key::Pause => "Pausar",
        Key::StepForward => "Avanzar paso",
        Key::StepBackward => "Retroceder paso",
        Key::Speed => "Velocidad",
        Key::Loop => "Bucle",

        Key::AboutSubtitle => "Explorador de datos N-dimensionales",
        Key::AboutDescription => {
            "Octant es un visor interactivo de conjuntos de datos N-dimensionales con soporte nativo para almacenamiento local y en la nube, Zarr (v2/v3) e Icechunk. Construido en Rust con aceleración GPU mediante WGPU. Funciona en macOS, Linux y Windows."
        }
        Key::AboutHyperslabTitle => "Corte de hiperbloques",
        Key::AboutHyperslabDesc => "Caché asíncrona LRU y pirámides multirresolución.",
        Key::AboutStorageTitle => "Nativo en Zarr e Icechunk",
        Key::AboutStorageDesc => "Backends locales, S3, GCS, Azure y transmisión HTTP.",
        Key::AboutShadersTitle => "Sombreadores WGPU acelerados por hardware",
        Key::AboutShadersDesc => {
            "Mapas 2D, esferas 3D, superficies de elevación, volúmenes y perfiles 1D."
        }
        Key::AboutLinksHeading => "Enlaces y recursos",
        Key::AboutGoodFaithNotice => {
            "Octant reconoce el trabajo de terceros de buena fe. Si considera que algo falta o está incorrectamente atribuido, contáctenos para corregirlo o retirarlo."
        }
        Key::AboutBrowseIcons => "Explorar iconos vectoriales nativos",
    })
}
