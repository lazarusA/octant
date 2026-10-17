use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Descartar",
        NoticesKey::CopyDetails => "Copiar detalles",
        NoticesKey::RevealInFinder => "Mostrar en Finder",
        NoticesKey::OpenFolder => "Abrir carpeta",
        NoticesKey::NoVariablesFound => "No se encontraron variables",
        NoticesKey::CouldntOpenDataset => "No se pudo abrir el conjunto de datos",
        NoticesKey::UnsupportedFile => "Archivo no compatible",
        NoticesKey::CouldntLoadData => "No se pudieron cargar los datos",
        NoticesKey::CoordinatesUnavailable => "Coordenadas no disponibles",
        NoticesKey::CoordinatesFailedDetail => {
            "Las coordenadas de '{}' no se pudieron cargar ({}); los ejes muestran índices"
        }
        NoticesKey::CoastlinesUnavailable => "Costas no disponibles",
        NoticesKey::VolumeTooLargeForGpu => "Volumen demasiado grande para la GPU",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "El tamaño del búfer de datos del volumen ({} elementos / {} bytes) supera el límite máximo del búfer de almacenamiento de la GPU ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "El volumen {}x{}x{} supera el límite de texturas 3D de la GPU ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Carga del volumen omitida",
        NoticesKey::VolumeUploadMismatchDetail => "{} valores para una textura de {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "Datos demasiado grandes para el búfer de la GPU",
        NoticesKey::BufferTooLargeDetail => {
            "{}: el tamaño de los datos ({} bytes) supera la capacidad del búfer de la GPU ({} bytes); se omite la escritura"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: el rango en el elemento {} (+{} elementos) supera la capacidad del búfer de la GPU ({} bytes); se omite la escritura"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Gráfico de líneas demasiado grande para la GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Las líneas ocupan {} bytes; esta GPU enlaza como máximo {}."
        }
        NoticesKey::FontLoadFailed => "No se pudo cargar una fuente",
        NoticesKey::FontLoadFailedDetail => "El texto en {} puede verse como recuadros: {}",
        NoticesKey::OmeMetadataNonConforming => {
            "Los metadatos OME-Zarr no cumplen la especificación"
        }
        NoticesKey::LabelIdsBeyond32Bits => {
            "Los ID de etiqueta de más de 32 bits comparten colores con ID menores"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Los metadatos de la versión {} de {} no siguen la especificación; se muestra lo que se pudo leer."
        }
        NoticesKey::PlateWellStopped => "Pocillo {}: la lectura se interrumpió",
        NoticesKey::PlateWellNotJson => "Pocillo {}: sus metadatos no son JSON válido",
        NoticesKey::PlateWellNotOme => "Pocillo {}: no es un pocillo OME-Zarr",
        NoticesKey::PlateWellNoMetadata => "Pocillo {}: no hay metadatos de pocillo",
    })
}
