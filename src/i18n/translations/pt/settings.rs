use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "As cores vêm dos canais da composição.",
        SettingsKey::ReasonCustomColor => {
            "Cor personalizada desenha todos os pontos com uma só cor."
        }
        SettingsKey::ReasonAllSeries => "Todas as séries colore cada linha pelo seu índice.",
        SettingsKey::ReasonIndexed => "RGBA indexado colore os voxels pelo índice do rótulo.",
        SettingsKey::ReasonIndexedMissing => "RGBA indexado ignora voxels ausentes.",
        SettingsKey::ReasonClassicOpaque => {
            "Apenas Raymarching de volume (DVR) desenha cores translúcidas."
        }
        SettingsKey::ReasonNotTranslucent => {
            "Tem efeito com Opacidade abaixo de 1 ou uma Curva alfa."
        }
        SettingsKey::ReasonCompositeOpaque => "Composições são desenhadas opacas.",
        SettingsKey::ReasonLabelOpaque => {
            "Superfície de rótulos categóricos é sempre desenhada opaca."
        }
        SettingsKey::ReasonAlwaysBlends => "Este algoritmo sempre mistura ao longo do raio.",
        SettingsKey::CloseSettingsHover => "Fechar configurações",
        SettingsKey::SectionOverlays => "Sobreposições",
        SettingsKey::SectionResolution => "Resolução",
        SettingsKey::SectionView => "Visualização",
        SettingsKey::Layers => "Camadas",
        SettingsKey::BaseLayerHover => "Camada base: define a tela, os eixos e a visualização",
        SettingsKey::BaseTag => "base",
        SettingsKey::Color => "Cor",
        SettingsKey::HideLayerHover => "Ocultar camada",
        SettingsKey::ShowLayerHover => "Mostrar camada",
        SettingsKey::RemoveLayerHover => "Remover camada",
        SettingsKey::MoveDownHover => "Mover para baixo",
        SettingsKey::MoveUpHover => "Mover para cima",
        SettingsKey::NoVariable => "(sem variável)",
        SettingsKey::ColorbarLabel => "Rótulo da barra de cores",
        SettingsKey::ResetLabelHover => "Redefinir o rótulo da barra de cores para o padrão",
        SettingsKey::ColorRange => "Intervalo de cores",
        SettingsKey::Categorical => "Categórico",
        SettingsKey::CategoricalHover => {
            "Barra de cores discreta (detecta valores únicos ou usa 10 classes iguais)."
        }
        SettingsKey::LockBoundsHover => {
            "Fixar mín/máx para que o mapeamento de cores não mude entre passos de tempo."
        }
        SettingsKey::ResetBoundsHover => {
            "Redefinir os limites para o mínimo e máximo da fatia/conjunto de dados atual"
        }
        SettingsKey::ResetAllColorbar => "Redefinir toda a barra de cores",
        SettingsKey::ResetAllColorbarHover => {
            "Redefinir o rótulo e o intervalo da barra de cores para os valores padrão"
        }
        SettingsKey::RangeMin => "Mín:",
        SettingsKey::RangeMax => "Máx:",
        SettingsKey::NanColor => "Cor de NaN",
        SettingsKey::NanColorHover => "Se desmarcado, valores NaN/Inf ficam transparentes.",
        SettingsKey::NanColorPickerHover => "Cor de NaN. Clique para selecionar a cor.",
        SettingsKey::LowClip => "Abaixo do intervalo",
        SettingsKey::LowClipHover => "Valores < cmin recebem esta cor.",
        SettingsKey::LowClipPickerHover => {
            "Cor abaixo do intervalo (< Mín). Clique para selecionar a cor."
        }
        SettingsKey::HighClip => "Acima do intervalo",
        SettingsKey::HighClipHover => "Valores > cmax recebem esta cor.",
        SettingsKey::HighClipPickerHover => {
            "Cor acima do intervalo (> Máx). Clique para selecionar a cor."
        }
        SettingsKey::Scale => "Escala",
        SettingsKey::ScaleLinear => "Linear",
        SettingsKey::ScaleLog => "Logarítmica",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Raiz quadrada",
        SettingsKey::ScaleExponential => "Exponencial",
        SettingsKey::ScaleLogHover => "Escala logarítmica (dados não negativos)",
        SettingsKey::ScaleLogDisabledHover => {
            "Desativada: requer mín >= 0. Use Symlog para dados negativos."
        }
        SettingsKey::ScaleParam => "Parâmetro:",
        SettingsKey::Opacity => "Opacidade",
        SettingsKey::OpacityHover => {
            "Opacidade dos valores mapeados em cores. As cores de NaN e fora do intervalo mantêm o próprio alfa. \
             Clique duas vezes para redefinir."
        }
        SettingsKey::AlphaCurve => "Curva alfa",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  ou  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Alfa ao longo da barra de cores, do início ao fim. Os valores são distribuídos \
             uniformemente (Degrau: uma classe igual para cada); paradas posição:alfa são \
             colocadas ao longo da barra. Vazio para nenhuma."
        }
        SettingsKey::ClearCurveHover => "Limpar curva",
        SettingsKey::Presets => "Predefinições",
        SettingsKey::InterpStep => "Degrau",
        SettingsKey::InterpLinear => "Linear",
        SettingsKey::PresetRampUp => "Rampa crescente",
        SettingsKey::PresetRampDown => "Rampa decrescente",
        SettingsKey::PresetPeak => "Pico",
        SettingsKey::PresetSink => "Vale",
        SettingsKey::PresetHideLow => "Ocultar baixos",
        SettingsKey::PresetHideCenter => "Ocultar centro",
        SettingsKey::Transparency => "Transparência",
        SettingsKey::TransparencyHover => {
            "Com cores translúcidas (Opacidade ou Curva alfa), desenha todas as camadas \
             em vez de deixar a mais próxima esconder as que estão atrás dela."
        }
        SettingsKey::Lighting => "Iluminação",
        SettingsKey::LightingHover => {
            "Sombreia as amostras pelo gradiente para dar forma a frentes e bordas"
        }
        SettingsKey::Coastlines => "Litorais",
        SettingsKey::CoastlinesHover => {
            "Sobrepõe os litorais do Natural Earth em gráficos geográficos."
        }
        SettingsKey::ResetCoastlineColor => "redefinir",
        SettingsKey::ResetCoastlineColorHover => "Redefinir a cor do litoral para o padrão do tema",
        SettingsKey::CoastlineColorHover => "Cor da linha do litoral. Clique para selecionar.",
        SettingsKey::CoastlineResolution => "Resolução:",
        SettingsKey::Lod110mHover => "110 m - rápido, sempre disponível (incorporado)",
        SettingsKey::Lod50mHover => "50 m - detalhe médio (carregado em segundo plano)",
        SettingsKey::Lod10mHover => "10 m - alto detalhe (carregado em segundo plano)",
        SettingsKey::LineWidth => "Espessura da linha",
        SettingsKey::CropToDomain => "Recortar ao domínio dos dados",
        SettingsKey::CropToDomainHover => {
            "Recorta os litorais estritamente ao limite espacial do conjunto de dados ativo."
        }
        SettingsKey::AutoRotate => "Rotação automática",
        SettingsKey::ResetView => "Redefinir visualização",
        SettingsKey::ResetViewHover => "Redefinir a orientação da câmera 3D",
        SettingsKey::HoverCard => "Cartão de informações",
        SettingsKey::HideHoverCardHover => "Ocultar o cartão de informações",
        SettingsKey::ShowHoverCardHover => "Mostrar o cartão de informações",
        SettingsKey::Export => "Exportar",
        SettingsKey::ExportFormat => "Formato:",
        SettingsKey::ExportFolder => "Pasta:",
        SettingsKey::OpenSaveDialog => "Abrir caixa de salvamento",
        SettingsKey::CropTool => "Ferramenta de recorte",
        SettingsKey::Aggregation => "Agregação 2D",
        SettingsKey::AggregationMandatoryHover => {
            "Obrigatória: a matriz excede o limite de um único buffer da GPU ({} MB). \
             A agregação não pode ser desativada."
        }
        SettingsKey::AggregationHover => {
            "Redução em pirâmide de multi-resolução e reamostragem da área visível."
        }
        SettingsKey::AggregationMode => "Modo:",
        SettingsKey::AggMean => "Média (filtro de caixa)",
        SettingsKey::AggMax => "Máx (preserva picos)",
        SettingsKey::AggMin => "Mín (preserva vales)",
        SettingsKey::AggNearest => "Vizinho mais próximo",
        SettingsKey::AggMeanHover => "Faz a média dos subpixels em células 2x2, ignorando NaNs.",
        SettingsKey::AggMaxHover => "Preserva valores extremos altos.",
        SettingsKey::AggMinHover => "Preserva valores extremos baixos.",
        SettingsKey::AggNearestHover => "Seleção rápida do ponto mais próximo, sem interpolação.",
        SettingsKey::CmykAutoMapped => {
            "Canais mapeados automaticamente: C (1), M (2), Y (3), K (4)"
        }
        SettingsKey::MultiChannelOverlay => "Sobreposição multicanal",
        SettingsKey::MultiChannelOverlayHover => {
            "Sobrepõe vários canais de forma aditiva, cada um com sua própria tonalidade."
        }
        SettingsKey::CmykComposite => "Composição CMYK",
        SettingsKey::CmykCompositeHover => {
            "Combina os 4 canais ciano, magenta, amarelo e preto (CMYK) em RGB de cor verdadeira."
        }
        SettingsKey::RgbComposite => "Composição RGB",
        SettingsKey::RgbCompositeHover => {
            "Combina os 3 canais selecionados em RGB de cor verdadeira."
        }
        SettingsKey::Channels => "Canais:",
        SettingsKey::ChannelsAll => "Todos",
        SettingsKey::ChannelsNone => "Nenhum",
        SettingsKey::ChannelTintHover => "Clique para personalizar a tonalidade do canal",
        SettingsKey::BandNumber => "Banda {}",
        SettingsKey::ScaleBar => "Barra de escala",
        SettingsKey::ShowScaleBarHover => "Mostrar a barra de escala",
        SettingsKey::HideScaleBarHover => "Ocultar a barra de escala",
        SettingsKey::ChannelsAuto => "Auto",
        SettingsKey::ChannelsAutoHover => {
            "Ajustar a faixa de exibição de cada canal aos dados mostrados (de 0,1 a 99,9 por cento dos seus valores)"
        }
        SettingsKey::ChannelsAutoUnavailable => "O contraste automático funciona em um corte 2D",
        SettingsKey::ChannelSettingsHover => "Faixa de exibição, curva e inversão",
        SettingsKey::ChannelWindow => "Faixa de exibição",
        SettingsKey::ChannelWindowHover => {
            "Os valores do início ao fim vão do preto até a cor plena do canal"
        }
        SettingsKey::ChannelInvert => "Inverter",
        SettingsKey::ChannelCurve => "Curva",
        SettingsKey::CurveLinear => "Linear",
        SettingsKey::CurveLogarithmic => "Logarítmica",
        SettingsKey::CurveExponential => "Exponencial",
        SettingsKey::CurvePolynomial => "Polinomial",
        SettingsKey::ChannelCoefficientHover => "Coeficiente da curva",
        SettingsKey::LabelsFill => "Preenchimento",
        SettingsKey::LabelsFillHover => "Colorir cada rótulo por inteiro",
        SettingsKey::LabelsOutline => "Contorno",
        SettingsKey::LabelsOutlineHover => "Desenhar só as bordas entre rótulos",
    })
}
