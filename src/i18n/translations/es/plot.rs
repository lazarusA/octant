use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Mapa de calor",
        PlotKey::Line => "Línea 1D",
        PlotKey::Surface => "Superficie",
        PlotKey::Volume => "Volumen",
        PlotKey::Sphere => "Esfera",
        PlotKey::PointCloud => "Nube de puntos",
        PlotKey::CompositeTrueColor => "Color verdadero",
        PlotKey::CompositeFalseColor => "Falso color",
        PlotKey::CompositeRgb => "Composición RGB",
        PlotKey::CompositeOverlay => "Superposición de canales",
        PlotKey::AlignGeo => "En otra cuadrícula de longitud/latitud (aún no compatible)",
        PlotKey::AlignIndexOnly => {
            "Misma forma de cuadrícula, pero ninguna coordenada coincide con el gráfico"
        }
        PlotKey::AlignIncompatible => "No comparte cuadrícula con el gráfico",
        PlotKey::OverlayPlotFirst => "Primero grafique una variable",
        PlotKey::OverlayHeatmapOnly => "Las superposiciones se dibujan sobre mapas de calor",
        PlotKey::OverlayStackFull => "Como máximo 4 superposiciones",
        PlotKey::OverlayAlreadyLayer => "Ya es una capa del gráfico",
        PlotKey::OverlayCantAdd => "No se puede añadir la superposición",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Seleccionar proyección de visualización",
        PlotKey::GridHealpixDiscrete => "HEALPix / discreta",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "No compatible con la cuadrícula {}",
        PlotKey::DisabledAggregation => "Desactivado: la agregación 2D está activa",
        PlotKey::Requires3D => "Requiere datos 3D",
        PlotKey::DisabledGpuLimit => "Desactivado: {} MB > límite de GPU de 128 MB",
        PlotKey::ScalarField => "Campo escalar",
        PlotKey::LabelsHeatmapOnly => "Las imágenes de etiquetas se dibujan en el mapa de calor",
    })
}
