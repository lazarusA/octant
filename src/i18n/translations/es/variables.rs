use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Grupo",
        VariablesKey::DataType => "Tipo",
        VariablesKey::Shape => "Forma",
        VariablesKey::Description => "Descripción",
        VariablesKey::SearchVariables => "Buscar variables...",
        VariablesKey::NoVariablesMatching => "Ninguna variable coincide con '{}'",
        VariablesKey::CloseVariablesWindow => "Cerrar la ventana de variables",
        VariablesKey::NoMetadataLoaded => "Aún no se han cargado los metadatos del almacén.",
        VariablesKey::FetchStoreMetadata => "Obtener / cargar metadatos del almacén",
        VariablesKey::NoVariablesFound => "No se encontraron variables en este almacén.",
        VariablesKey::RefreshStoreMetadata => "Actualizar metadatos del almacén",
        VariablesKey::TruncatedVariables => {
            "Mostrando {} de {} variables en esta carpeta. Use la búsqueda para verlas todas."
        }
        VariablesKey::TruncatedFolders => {
            "Mostrando {} de {} carpetas. Use la búsqueda para verlas todas."
        }
        VariablesKey::NoVariableSelected => "Ninguna variable seleccionada.",
        VariablesKey::DimensionSliders => "Controles de dimensiones",
        VariablesKey::CloseDimensionPanel => "Cerrar el panel de dimensiones",
        VariablesKey::PlotData => "Graficar datos",
        VariablesKey::PlotDataHover => "Obtener esta variable y graficarla",
        VariablesKey::PlotDataOverlayHover => {
            "Obtener esta variable y añadirla como superposición sobre el gráfico"
        }
        VariablesKey::AddOverlay => "Añadir superposición",
        VariablesKey::AddOverlayHover => {
            "Mientras está activo, Graficar datos añade esta variable como superposición"
        }
        VariablesKey::InfoPath => "Ruta:",
        VariablesKey::InfoUnits => "Unidades: {}",
        VariablesKey::InfoDescription => "Descripción: {}",
        VariablesKey::InfoShape => "Forma: {}",
        VariablesKey::InfoDimensions => "Dimensiones: {}",
        VariablesKey::InfoTime => "Tiempo: {} -> {}",
        VariablesKey::InfoResolution => "Resolución: {}",
        VariablesKey::InfoSize => "Tamaño: {} MB",
        VariablesKey::InfoAttributes => "Atributos (.zattrs)",
        VariablesKey::Download => "Descargar",
        VariablesKey::DownloadCells => "{} ({} celdas)",
        VariablesKey::LargeSelectionPyramid => {
            "Selección 2D grande ({} celdas, {} MB): la agregación automática en pirámide multirresolución está activada."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "Las mallas de Globo 3D y Superficie 3D están desactivadas para esta selección grande ({} celdas, {} MB). Los gráficos de Plano 2D y Línea 1D siguen plenamente activos."
        }
        VariablesKey::LargeVolumeDisabled => {
            "Volumen 3D y Nube de puntos están desactivados para esta selección: el tamaño del volumen ({} MB) supera el límite de 128 MB del búfer de almacenamiento de la GPU. Plano 2D, Línea 1D y Globo 3D siguen activos."
        }
        VariablesKey::RoleNone => "Ninguno",
        VariablesKey::RoleAnimated => "Animado",
        VariablesKey::RoleGrid => "Malla",
        VariablesKey::Index => "Índice:",
        VariablesKey::ChannelN => "Canal {}",
    })
}
