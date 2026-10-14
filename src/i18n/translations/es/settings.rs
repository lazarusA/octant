use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Los colores provienen de los canales de la composición.",
        SettingsKey::ReasonCustomColor => {
            "Color personalizado dibuja todos los puntos de un solo color."
        }
        SettingsKey::ReasonAllSeries => "Todas las series colorea cada línea según su índice.",
        SettingsKey::ReasonIndexed => {
            "RGBA indexado colorea los vóxeles según su índice de etiqueta."
        }
        SettingsKey::ReasonIndexedMissing => "RGBA indexado omite los vóxeles faltantes.",
        SettingsKey::ReasonClassicOpaque => {
            "Solo Raymarching volumétrico (DVR) dibuja colores translúcidos."
        }
        SettingsKey::ReasonNotTranslucent => {
            "Surte efecto con una Opacidad menor que 1 o una Curva alfa."
        }
        SettingsKey::ReasonCompositeOpaque => "Las composiciones se dibujan opacas.",
        SettingsKey::ReasonLabelOpaque => {
            "Superficie de etiquetas categóricas siempre se dibuja opaca."
        }
        SettingsKey::ReasonAlwaysBlends => "Este algoritmo siempre mezcla a lo largo del rayo.",
        SettingsKey::CloseSettingsHover => "Cerrar configuración",
        SettingsKey::SectionOverlays => "Superposiciones",
        SettingsKey::SectionResolution => "Resolución",
        SettingsKey::SectionView => "Vista",
        SettingsKey::Layers => "Capas",
        SettingsKey::BaseLayerHover => "Capa base: determina el lienzo, los ejes y la vista",
        SettingsKey::BaseTag => "base",
        SettingsKey::Color => "Color",
        SettingsKey::HideLayerHover => "Ocultar capa",
        SettingsKey::ShowLayerHover => "Mostrar capa",
        SettingsKey::RemoveLayerHover => "Quitar capa",
        SettingsKey::MoveDownHover => "Bajar",
        SettingsKey::MoveUpHover => "Subir",
        SettingsKey::NoVariable => "(sin variable)",
        SettingsKey::ColorbarLabel => "Etiqueta de la barra de color",
        SettingsKey::ResetLabelHover => {
            "Restablecer la etiqueta de la barra de color a su valor predeterminado"
        }
        SettingsKey::ColorRange => "Rango de color",
        SettingsKey::Categorical => "Categórico",
        SettingsKey::CategoricalHover => {
            "Barra de color discreta (detecta valores únicos o usa 10 intervalos iguales)."
        }
        SettingsKey::LockBoundsHover => {
            "Fijar mín./máx. para que la asignación de colores no cambie entre pasos de tiempo."
        }
        SettingsKey::ResetBoundsHover => {
            "Restablecer los límites al mínimo y máximo del corte o conjunto de datos actual"
        }
        SettingsKey::ResetAllColorbar => "Restablecer la barra de color",
        SettingsKey::ResetAllColorbarHover => {
            "Restablecer la etiqueta y el rango de la barra de color a sus valores predeterminados"
        }
        SettingsKey::RangeMin => "Mín.:",
        SettingsKey::RangeMax => "Máx.:",
        SettingsKey::NanColor => "Color NaN",
        SettingsKey::NanColorHover => {
            "Si no está marcado, los valores NaN/Inf se muestran transparentes."
        }
        SettingsKey::NanColorPickerHover => "Color NaN. Haga clic para seleccionar el color.",
        SettingsKey::LowClip => "Bajo el rango",
        SettingsKey::LowClipHover => "Los valores < cmin toman este color.",
        SettingsKey::LowClipPickerHover => {
            "Color bajo el rango (< Mín.). Haga clic para seleccionar el color."
        }
        SettingsKey::HighClip => "Sobre el rango",
        SettingsKey::HighClipHover => "Los valores > cmax toman este color.",
        SettingsKey::HighClipPickerHover => {
            "Color sobre el rango (> Máx.). Haga clic para seleccionar el color."
        }
        SettingsKey::Scale => "Escala",
        SettingsKey::ScaleLinear => "Lineal",
        SettingsKey::ScaleLog => "Logarítmica",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Raíz cuadrada",
        SettingsKey::ScaleExponential => "Exponencial",
        SettingsKey::ScaleLogHover => "Escala logarítmica (datos no negativos)",
        SettingsKey::ScaleLogDisabledHover => {
            "Desactivado: requiere mín. >= 0. Use Symlog para datos negativos."
        }
        SettingsKey::ScaleParam => "Parám.:",
        SettingsKey::Opacity => "Opacidad",
        SettingsKey::OpacityHover => {
            "Opacidad de los valores con mapa de color. Los colores NaN y fuera del rango conservan su propio alfa. \
             Doble clic para restablecer."
        }
        SettingsKey::AlphaCurve => "Curva alfa",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  o  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alfa a lo largo de la barra de color, de su inicio a su fin. Los valores se \
             reparten uniformemente (Escalón: un intervalo igual cada uno); las paradas \
             posición:alfa se colocan a lo largo de la barra. Vacío para ninguna."
        }
        SettingsKey::ClearCurveHover => "Borrar curva",
        SettingsKey::Presets => "Predefinidas",
        SettingsKey::InterpStep => "Escalón",
        SettingsKey::InterpLinear => "Lineal",
        SettingsKey::PresetRampUp => "Rampa ascendente",
        SettingsKey::PresetRampDown => "Rampa descendente",
        SettingsKey::PresetPeak => "Pico",
        SettingsKey::PresetSink => "Valle",
        SettingsKey::PresetHideLow => "Ocultar bajos",
        SettingsKey::PresetHideCenter => "Ocultar centro",
        SettingsKey::Transparency => "Transparencia",
        SettingsKey::TransparencyHover => {
            "Con colores translúcidos (Opacidad o Curva alfa), dibujar todas las capas \
             en lugar de dejar que la más cercana oculte las que están detrás."
        }
        SettingsKey::Lighting => "Iluminación",
        SettingsKey::LightingHover => {
            "Sombrear las muestras según su gradiente para que frentes y bordes ganen forma"
        }
        SettingsKey::Coastlines => "Costas",
        SettingsKey::CoastlinesHover => {
            "Superponer las líneas de costa de Natural Earth en gráficos geográficos."
        }
        SettingsKey::ResetCoastlineColor => "restablecer",
        SettingsKey::ResetCoastlineColorHover => {
            "Restablecer el color de la costa al predeterminado del tema"
        }
        SettingsKey::CoastlineColorHover => {
            "Color de la línea de costa. Haga clic para seleccionarlo."
        }
        SettingsKey::CoastlineResolution => "Resolución:",
        SettingsKey::Lod110mHover => "110 m: rápida, siempre disponible (integrada)",
        SettingsKey::Lod50mHover => "50 m: detalle medio (se carga en segundo plano)",
        SettingsKey::Lod10mHover => "10 m: alto detalle (se carga en segundo plano)",
        SettingsKey::LineWidth => "Grosor de línea",
        SettingsKey::CropToDomain => "Recortar al dominio de datos",
        SettingsKey::CropToDomainHover => {
            "Recortar las costas estrictamente al límite espacial del conjunto de datos activo."
        }
        SettingsKey::AutoRotate => "Rotación automática",
        SettingsKey::ResetView => "Restablecer vista",
        SettingsKey::ResetViewHover => "Restablecer la orientación de la cámara 3D",
        SettingsKey::HoverCard => "Tarjeta flotante",
        SettingsKey::HideHoverCardHover => "Ocultar la tarjeta flotante",
        SettingsKey::ShowHoverCardHover => "Mostrar la tarjeta flotante",
        SettingsKey::Export => "Exportar",
        SettingsKey::ExportFormat => "Formato:",
        SettingsKey::ExportFolder => "Carpeta:",
        SettingsKey::OpenSaveDialog => "Abrir diálogo de guardado",
        SettingsKey::CropTool => "Herramienta de recorte",
        SettingsKey::Aggregation => "Agregación 2D",
        SettingsKey::AggregationMandatoryHover => {
            "Obligatoria: el arreglo supera el límite de un búfer de GPU (128 MB / 33,5 M celdas). \
             La agregación no se puede desactivar."
        }
        SettingsKey::AggregationHover => {
            "Submuestreo en pirámide multirresolución y remuestreo de la vista."
        }
        SettingsKey::AggregationMode => "Modo:",
        SettingsKey::AggMean => "Media (filtro de caja)",
        SettingsKey::AggMax => "Máx. (conserva picos)",
        SettingsKey::AggMin => "Mín. (conserva valles)",
        SettingsKey::AggNearest => "Vecino más cercano",
        SettingsKey::AggMeanHover => "Promedia subpíxeles en celdas de 2x2, ignorando los NaN.",
        SettingsKey::AggMaxHover => "Conserva los valores extremos altos.",
        SettingsKey::AggMinHover => "Conserva los valores extremos bajos.",
        SettingsKey::AggNearestHover => {
            "Selección rápida del punto más cercano, sin interpolación."
        }
        SettingsKey::CmykAutoMapped => {
            "Canales asignados automáticamente: C (1), M (2), Y (3), K (4)"
        }
        SettingsKey::MultiChannelOverlay => "Superposición multicanal",
        SettingsKey::MultiChannelOverlayHover => {
            "Superpone varios canales de forma aditiva, cada uno con su propio tinte de color."
        }
        SettingsKey::CmykComposite => "Composición CMYK",
        SettingsKey::CmykCompositeHover => {
            "Combina los 4 canales cian, magenta, amarillo y negro (CMYK) en RGB de color verdadero."
        }
        SettingsKey::RgbComposite => "Composición RGB",
        SettingsKey::RgbCompositeHover => {
            "Combina los 3 canales seleccionados en RGB de color verdadero."
        }
        SettingsKey::Channels => "Canales:",
        SettingsKey::ChannelsAll => "Todos",
        SettingsKey::ChannelsNone => "Ninguno",
        SettingsKey::ChannelTintHover => "Haga clic para personalizar el tinte del canal",
        SettingsKey::BandNumber => "Banda {}",
        SettingsKey::ScaleBar => "Barra de escala",
        SettingsKey::ShowScaleBarHover => "Mostrar la barra de escala",
        SettingsKey::HideScaleBarHover => "Ocultar la barra de escala",
        SettingsKey::ChannelsAuto => "Auto",
        SettingsKey::ChannelsAutoHover => {
            "Ajustar el rango de visualización de cada canal a los datos mostrados (del 0,1 al 99,9 por ciento de sus valores)"
        }
        SettingsKey::ChannelsAutoUnavailable => {
            "El contraste automático funciona sobre un corte 2D"
        }
        SettingsKey::ChannelSettingsHover => "Rango de visualización, curva e inversión",
        SettingsKey::ChannelWindow => "Rango de visualización",
        SettingsKey::ChannelWindowHover => {
            "Los valores de inicio a fin recorren el canal del negro a su color pleno"
        }
        SettingsKey::ChannelInvert => "Invertir",
        SettingsKey::ChannelCurve => "Curva",
        SettingsKey::CurveLinear => "Lineal",
        SettingsKey::CurveLogarithmic => "Logarítmica",
        SettingsKey::CurveExponential => "Exponencial",
        SettingsKey::CurvePolynomial => "Polinómica",
        SettingsKey::ChannelCoefficientHover => "Coeficiente de la curva",
        SettingsKey::LabelsFill => "Relleno",
        SettingsKey::LabelsFillHover => "Colorear cada etiqueta entera",
        SettingsKey::LabelsOutline => "Contorno",
        SettingsKey::LabelsOutlineHover => "Dibujar solo los bordes entre etiquetas",
    })
}
