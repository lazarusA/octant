use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Mapa de calor",
        PlotKey::Line => "Linha 1D",
        PlotKey::Surface => "Superfície",
        PlotKey::Volume => "Volume",
        PlotKey::Sphere => "Esfera",
        PlotKey::PointCloud => "Nuvem de pontos",
        PlotKey::CompositeTrueColor => "Cor verdadeira",
        PlotKey::CompositeFalseColor => "Falsa cor",
        PlotKey::CompositeRgb => "Composição RGB",
        PlotKey::CompositeOverlay => "Sobreposição de canais",
        PlotKey::AlignGeo => "Em outra grade de longitude/latitude (ainda não suportado)",
        PlotKey::AlignIndexOnly => {
            "Mesma forma de grade, mas nenhuma coordenada corresponde ao gráfico"
        }
        PlotKey::AlignIncompatible => "Não compartilha grade com o gráfico",
        PlotKey::OverlayPlotFirst => "Plote uma variável primeiro",
        PlotKey::OverlayHeatmapOnly => "Sobreposições são desenhadas sobre mapas de calor",
        PlotKey::OverlayStackFull => "No máximo 4 sobreposições",
        PlotKey::OverlayAlreadyLayer => "Já é uma camada do gráfico",
        PlotKey::OverlayCantAdd => "Não é possível adicionar a sobreposição",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Selecionar projeção de visualização",
        PlotKey::GridHealpixDiscrete => "HEALPix / discreta",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Não suportado para grade {}",
        PlotKey::DisabledAggregation => "Desativado: a Agregação 2D está ativada",
        PlotKey::DisabledGpuLimit => "Desativado: {} MB > limite de {} MB da GPU",
        PlotKey::ScalarField => "Campo escalar",
        PlotKey::LabelsHeatmapOnly => "Imagens de rótulos são desenhadas só no mapa de calor",
    })
}
