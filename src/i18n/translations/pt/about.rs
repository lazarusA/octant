use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Sobre o Octant",
        AboutKey::IconsWindowTitle => "Ícones vetoriais nativos",
        AboutKey::IconsTab => "Ícones vetoriais",
        AboutKey::Subtitle => "Explorador de dados N-dimensionais",
        AboutKey::Description => {
            "O Octant é um visualizador interativo para conjuntos de dados N-dimensionais com suporte nativo para armazenamento de objetos local e em nuvem, Zarr (v2/v3) e Icechunk. Construído em Rust com renderização acelerada por GPU via WGPU. O Octant roda nativamente em macOS, Linux e Windows."
        }
        AboutKey::HyperslabTitle => "Fatiamento de hiperblocos",
        AboutKey::HyperslabDesc => "Cache LRU assíncrono de chunks e pirâmides de multi-resolução.",
        AboutKey::StorageTitle => "Nativo em Zarr e Icechunk",
        AboutKey::StorageDesc => "Backends locais, S3, GCS, Azure e streaming HTTP.",
        AboutKey::ShadersTitle => "Shaders WGPU acelerados por hardware",
        AboutKey::ShadersDesc => {
            "Mapas planos 2D, esferas 3D, superfícies de elevação, volumes e perfis 1D."
        }
        AboutKey::LinksHeading => "Links e recursos",
        AboutKey::DocsLink => "Documentação do Octant",
        AboutKey::GoodFaithNotice => {
            "O Octant dá crédito a obras de terceiros de boa fé. Se você acredita que algo está faltando ou foi atribuído incorretamente, avise-nos e iremos corrigir ou remover."
        }
        AboutKey::BrowseIcons => "Explorar ícones vetoriais nativos",
        AboutKey::BrowseIconsHover => "Explorar ícones vetoriais procedurais em uma janela",
        AboutKey::Licensed => "Licenciado sob MIT ou Apache-2.0",
        AboutKey::OctantTeaser => {
            "Os sinais de x, y e z (+/-, +/-, +/-) dividem o espaço em oito octantes. Explore seus dados da mesma forma: escolha os eixos, corte uma fatia e observe uma região de cada vez."
        }
        AboutKey::OctantHover => "Clique para ir ao próximo octante",
        AboutKey::ColormapCredits => "Créditos dos mapas de cores",
        AboutKey::ColormapCreditsNote => {
            "Os mapas de cores incluídos mantêm as licenças de seus autores."
        }
        AboutKey::LicenseTexts => "Textos das licenças",
        AboutKey::ThirdPartyLicenses => "Licenças de terceiros",
        AboutKey::ThirdPartyNote => {
            "O Octant inclui crates, fontes e bibliotecas C de código aberto (netCDF, HDF5), cada um sob sua própria licença."
        }
        AboutKey::ViewOnGithub => "Ver no GitHub",
        AboutKey::IconsSearchHint => "Pesquisar ícones...",
        AboutKey::IconsAll => "Todos",
        AboutKey::IconsCopied => "{} copiado",
        AboutKey::IconsNoMatch => "Nenhum ícone vetorial encontrado.",
        AboutKey::IconsCategory => "Categoria: {}",
        AboutKey::IconsClickToCopy => "Clique para copiar o identificador do enum",
        AboutKey::IconsSizesTones => "Tamanhos e tons",
        AboutKey::ClearSearchHover => "Limpar pesquisa",
    })
}
