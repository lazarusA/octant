use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Linhas",
        PlotOptionsKey::LinesHover => "Mostrar linhas contínuas ligando os pontos",
        PlotOptionsKey::Scatter => "Dispersão",
        PlotOptionsKey::ScatterHover => "Mostrar marcadores de dispersão em cada ponto",
        PlotOptionsKey::PointSize => "Tamanho do ponto:",
        PlotOptionsKey::CustomColor => "Cor personalizada",
        PlotOptionsKey::CustomColorHover => {
            "Usar uma cor sólida para linhas/pontos em vez do mapa de cores"
        }
        PlotOptionsKey::LineColorHover => "Cor das linhas / pontos. Clique para selecionar a cor.",
        PlotOptionsKey::LineProfile => "Perfil de linha",
        PlotOptionsKey::AllLinesSeries => "Todas as séries",
        PlotOptionsKey::ProfileIndex => "Índice do perfil",
        PlotOptionsKey::SingleProfile => "Apenas um perfil disponível.",
        PlotOptionsKey::AspectRatio => "Proporção",
        PlotOptionsKey::AspectRatioHover => {
            "Se marcado, os gráficos 2D preservam a proporção dos dados da matriz (largura/altura). \
             Se desmarcado, os gráficos 2D se expandem para preencher toda a tela."
        }
        PlotOptionsKey::Algorithm => "Algoritmo",
        PlotOptionsKey::AlgoDvr => "Raymarching de volume (DVR)",
        PlotOptionsKey::AlgoMip => "Intensidade máxima (MIP)",
        PlotOptionsKey::AlgoMinIp => "Intensidade mínima (MinIP)",
        PlotOptionsKey::AlgoXray => "Projeção média (raio X)",
        PlotOptionsKey::AlgoLabelSurface => "Superfície de rótulos categóricos",
        PlotOptionsKey::AlgoAbsorption => "RGBA por absorção",
        PlotOptionsKey::AlgoAdditive => "RGBA aditivo",
        PlotOptionsKey::AlgoIndexed => "RGBA indexado",
        PlotOptionsKey::Quality => "Qualidade",
        PlotOptionsKey::QualityHover => "Amostras por voxel ao longo de cada raio",
        PlotOptionsKey::Density => "Densidade",
        PlotOptionsKey::Attenuation => "Atenuação",
        PlotOptionsKey::ZScale => "Escala Z",
        PlotOptionsKey::ModeSmooth => "Suave",
        PlotOptionsKey::ModeBumpy => "Relevo",
        PlotOptionsKey::ModeSteps => "Degraus",
        PlotOptionsKey::ModeVoxel => "Voxel",
        PlotOptionsKey::Height => "Altura",
        PlotOptionsKey::FlipHeights => "Inverter alturas",
        PlotOptionsKey::FlipHeightsHover => {
            "O valor mais alto fica plano e os valores mais baixos se elevam acima dele"
        }
        PlotOptionsKey::Size => "Tamanho",
    })
}
