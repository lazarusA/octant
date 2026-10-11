use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Líneas",
        PlotOptionsKey::LinesHover => "Mostrar líneas continuas que unen los puntos",
        PlotOptionsKey::Scatter => "Dispersión",
        PlotOptionsKey::ScatterHover => "Mostrar marcadores de dispersión en cada punto",
        PlotOptionsKey::PointSize => "Tamaño de punto:",
        PlotOptionsKey::CustomColor => "Color personalizado",
        PlotOptionsKey::CustomColorHover => {
            "Usar un color sólido de línea/dispersión en lugar del mapa de color"
        }
        PlotOptionsKey::LineColorHover => {
            "Color de línea / dispersión. Haga clic para seleccionar el color."
        }
        PlotOptionsKey::LineProfile => "Perfil de línea",
        PlotOptionsKey::AllLinesSeries => "Todas las series",
        PlotOptionsKey::ProfileIndex => "Índice de perfil",
        PlotOptionsKey::SingleProfile => "Solo hay un perfil disponible.",
        PlotOptionsKey::AspectRatio => "Relación de aspecto",
        PlotOptionsKey::AspectRatioHover => {
            "Si está marcado, los gráficos 2D conservan la relación de aspecto de la matriz (ancho/alto). \
             Si no, los gráficos 2D se expanden hasta llenar el lienzo."
        }
        PlotOptionsKey::Algorithm => "Algoritmo",
        PlotOptionsKey::AlgoDvr => "Raymarching volumétrico (DVR)",
        PlotOptionsKey::AlgoMip => "Intensidad máxima (MIP)",
        PlotOptionsKey::AlgoMinIp => "Intensidad mínima (MinIP)",
        PlotOptionsKey::AlgoXray => "Proyección media (rayos X)",
        PlotOptionsKey::AlgoLabelSurface => "Superficie de etiquetas categóricas",
        PlotOptionsKey::AlgoAbsorption => "RGBA por absorción",
        PlotOptionsKey::AlgoAdditive => "RGBA aditivo",
        PlotOptionsKey::AlgoIndexed => "RGBA indexado",
        PlotOptionsKey::Quality => "Calidad",
        PlotOptionsKey::QualityHover => "Muestras por vóxel a lo largo de cada rayo",
        PlotOptionsKey::Density => "Densidad",
        PlotOptionsKey::Attenuation => "Atenuación",
        PlotOptionsKey::ZScale => "Escala Z",
        PlotOptionsKey::ModeSmooth => "Suave",
        PlotOptionsKey::ModeBumpy => "Rugoso",
        PlotOptionsKey::ModeSteps => "Escalones",
        PlotOptionsKey::ModeVoxel => "Vóxel",
        PlotOptionsKey::Height => "Altura",
        PlotOptionsKey::FlipHeights => "Invertir alturas",
        PlotOptionsKey::FlipHeightsHover => {
            "El valor más alto queda plano y los valores menores se elevan sobre él"
        }
        PlotOptionsKey::Size => "Tamaño",
    })
}
