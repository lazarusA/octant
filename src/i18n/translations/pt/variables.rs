use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Grupo",
        VariablesKey::DataType => "Tipo",
        VariablesKey::Shape => "Formato",
        VariablesKey::Description => "Descrição",
        VariablesKey::SearchVariables => "Pesquisar variáveis...",
        VariablesKey::NoVariablesMatching => "Nenhuma variável corresponde a '{}'",
        VariablesKey::CloseVariablesWindow => "Fechar a janela de variáveis",
        VariablesKey::NoMetadataLoaded => "Nenhum metadado do repositório carregado ainda.",
        VariablesKey::FetchStoreMetadata => "Buscar / carregar metadados do repositório",
        VariablesKey::NoVariablesFound => "Nenhuma variável encontrada neste repositório.",
        VariablesKey::RefreshStoreMetadata => "Atualizar metadados do repositório",
        VariablesKey::TruncatedVariables => {
            "Mostrando {} de {} variáveis nesta pasta. Use a pesquisa para ver todas."
        }
        VariablesKey::TruncatedFolders => {
            "Mostrando {} de {} pastas. Use a pesquisa para ver todas."
        }
        VariablesKey::NoVariableSelected => "Nenhuma variável selecionada.",
        VariablesKey::DimensionSliders => "Controles de dimensão",
        VariablesKey::CloseDimensionPanel => "Fechar o painel de dimensões",
        VariablesKey::PlotData => "Plotar dados",
        VariablesKey::PlotDataHover => "Buscar esta variável e plotá-la",
        VariablesKey::PlotDataOverlayHover => {
            "Buscar esta variável e adicioná-la como sobreposição ao gráfico"
        }
        VariablesKey::AddOverlay => "Adicionar sobreposição",
        VariablesKey::AddOverlayHover => {
            "Quando ativado, Plotar dados adiciona esta variável como sobreposição"
        }
        VariablesKey::InfoPath => "Caminho:",
        VariablesKey::InfoUnits => "Unidades: {}",
        VariablesKey::InfoDescription => "Descrição: {}",
        VariablesKey::InfoShape => "Formato: {}",
        VariablesKey::InfoDimensions => "Dimensões: {}",
        VariablesKey::InfoTime => "Tempo: {} -> {}",
        VariablesKey::InfoResolution => "Resolução: {}",
        VariablesKey::InfoSize => "Tamanho: {} MB",
        VariablesKey::InfoAttributes => "Atributos (.zattrs)",
        VariablesKey::Download => "Download",
        VariablesKey::DownloadCells => "{} ({} células)",
        VariablesKey::LargeSelectionPyramid => {
            "Seleção 2D grande ({} células, {} MB): a agregação automática em pirâmide de multi-resolução está ativada."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "As malhas de Globo 3D e Superfície 3D estão desativadas para esta seleção grande ({} células, {} MB). Os gráficos de Plano 2D e Linha 1D continuam totalmente ativos."
        }
        VariablesKey::LargeVolumeDisabled => {
            "Volume 3D e Nuvem de pontos estão desativados para esta seleção: o tamanho do volume ({} MB) excede o limite de 128 MB do buffer de armazenamento da GPU. Plano 2D, Linha 1D e Globo 3D continuam ativos."
        }
        VariablesKey::RoleNone => "Nenhum",
        VariablesKey::RoleAnimated => "Animado",
        VariablesKey::RoleGrid => "Grade",
        VariablesKey::Index => "Índice:",
        VariablesKey::ChannelN => "Canal {}",
    })
}
