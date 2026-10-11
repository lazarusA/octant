//! Portuguese (Português) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Conjunto de dados",
        Key::Variables => "Variáveis",
        Key::Dimensions => "Dimensões",
        Key::Plot => "Gráfico",
        Key::Colormaps => "Mapas de cores",
        Key::Settings => "Configurações",
        Key::About => "Sobre",

        Key::InspectMetadata => "Inspecionar metadados",
        Key::ActiveDatasets => "Conjuntos de dados ativos",
        Key::NoActiveDatasets => "Nenhum conjunto ativo. Abra um conjunto de dados para começar.",
        Key::CacheUsage => "Uso do cache",
        Key::ClearCache => "Limpar cache",
        Key::OpenLocal => "Abrir arquivo local",
        Key::CloudStore => "Armazenamento em nuvem",
        Key::SampleDatasets => "Conjuntos de amostra",
        Key::DropFilesHere => "Solte arquivos ou pastas aqui",
        Key::SourceUrlOrPath => "URI do repositório ou caminho local",
        Key::Inspect => "Inspecionar",
        Key::LoadingMetadata => "Inspecionando metadados em segundo plano...",

        Key::Group => "Grupo:",
        Key::DataType => "Tipo",
        Key::Shape => "Formato",
        Key::Description => "Descrição",
        Key::SearchVariables => "Pesquisar variáveis...",
        Key::NoVariablesMatching => "Nenhuma variável encontrada",
        Key::ShowingItemsOfTotal => "Exibindo itens do total",

        Key::Ready => "Pronto",
        Key::Loading => "Carregando...",
        Key::Close => "Fechar",
        Key::Cancel => "Cancelar",
        Key::Save => "Salvar",
        Key::Reset => "Redefinir",
        Key::Error => "Erro",

        Key::Play => "Reproduzir",
        Key::Pause => "Pausar",
        Key::StepForward => "Avançar etapa",
        Key::StepBackward => "Voltar etapa",
        Key::Speed => "Velocidade",
        Key::Loop => "Repetição",

        Key::AboutSubtitle => "Explorador de dados N-dimensionais",
        Key::AboutDescription => {
            "Octant é um visualizador interativo para conjuntos de dados N-dimensionais com suporte nativo para armazenamento local e em nuvem, Zarr (v2/v3) e Icechunk. Construído em Rust com renderização acelerada por GPU via WGPU. Funciona em macOS, Linux e Windows."
        }
        Key::AboutHyperslabTitle => "Fatiamento de hiperblocos",
        Key::AboutHyperslabDesc => "Cache LRU assíncrono e pirâmides de multi-resolução.",
        Key::AboutStorageTitle => "Nativo em Zarr e Icechunk",
        Key::AboutStorageDesc => "Backends locais, S3, GCS, Azure e streaming HTTP.",
        Key::AboutShadersTitle => "Shaders WGPU acelerados por hardware",
        Key::AboutShadersDesc => {
            "Mapas 2D, esferas 3D, superfícies de elevação, volumes e perfis 1D."
        }
        Key::AboutLinksHeading => "Links e recursos",
        Key::AboutGoodFaithNotice => {
            "Octant reconhece obras de terceiros de boa fé. Caso algo esteja ausente ou incorreto, entre em contato para correção."
        }
        Key::AboutBrowseIcons => "Explorar ícones vetoriais nativos",
    })
}
