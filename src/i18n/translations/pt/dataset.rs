use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Conjunto de dados ativo",
        DatasetKey::NoDatasetLoaded => "Nenhum conjunto de dados carregado",
        DatasetKey::MemoryCache => "Memória e cache",
        DatasetKey::SampleDatasets => "Exemplos",
        DatasetKey::OpenFolder => "Abrir pasta",
        DatasetKey::OpenFile => "Abrir arquivo",
        DatasetKey::DropFilesHere => "Arraste e solte um arquivo ou pasta",
        DatasetKey::SourceUrlOrPath => "URL de destino / caminho do arquivo",
        DatasetKey::LoadingMetadata => "Inspecionando metadados em segundo plano...",
        // Dataset panel
        DatasetKey::OpenSource => "Abrir fonte",
        DatasetKey::DatasetManager => "Gerenciador de conjuntos de dados",
        DatasetKey::CloseDatasetHover => "Fechar conjunto de dados",
        // Active dataset card
        DatasetKey::TargetUriHover => "URI de destino:\n{}",
        DatasetKey::VariableCountOne => "{} variável",
        DatasetKey::VariableCountMany => "{} variáveis",
        DatasetKey::InspectVariables => "Inspecionar variáveis",
        DatasetKey::Reload => "Recarregar",
        DatasetKey::BadgeVolume4D => "Volume 4D",
        DatasetKey::BadgeMatrix2D => "Matriz 2D",
        DatasetKey::BadgeStore => "Repositório",
        // Source intake
        DatasetKey::Load => "Carregar",
        DatasetKey::TargetPathHint => "https://... ou caminho (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Catálogo...",
        DatasetKey::Clear => "Limpar",
        DatasetKey::FormatLabel => "Formato:",
        DatasetKey::SampleSeasFire => "SeasFire: incêndios florestais globais (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Matriz procedural 2D (teste)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "Zarr remoto (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Zarr local (sistema de arquivos)",
        DatasetKey::FormatRemoteIcechunk => "Icechunk remoto (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Icechunk local (sistema de arquivos)",
        DatasetKey::FormatRemoteGeoTiff => "GeoTIFF / COG remoto (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "GeoTIFF local (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "NetCDF / HDF5 local (.nc/.h5)",
        DatasetKey::FormatVolume4D => "Volume 4D de referência (procedural)",
        DatasetKey::FormatMatrix2D => "Matriz procedural 2D (procedural)",
        DatasetKey::FormatRemoteGeoTiffShort => "GeoTIFF / COG remoto",
        DatasetKey::FormatLocalGeoTiffShort => "GeoTIFF local",
        DatasetKey::FormatLocalNetCdfShort => "NetCDF / HDF5 local",
        DatasetKey::FormatGrib => "GRIB2 local ou remoto (.grib2/.grb2)",
        DatasetKey::FormatGribShort => "GRIB2",
        DatasetKey::FormatVolume4DShort => "Volume 4D de referência",
        DatasetKey::FormatMatrix2DShort => "Matriz procedural 2D",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "Nenhum conjunto de dados registrado no gerenciador.",
        DatasetKey::LoadedCount => "Carregados ({}):",
        DatasetKey::ClearAll => "Limpar tudo",
        DatasetKey::ClearAllHover => "Remover todos os conjuntos de dados do gerenciador",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Remover este conjunto de dados",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} blocos, {}% de acertos",
        DatasetKey::CacheMenuTitle => "Cache de memória unificado para várias variáveis",
        DatasetKey::GlobalMemoryUsage => "Uso global de memória RAM:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Uso de memória por tipo:",
        DatasetKey::ResidentHyperslabs => "Hyperslabs residentes: {} MB ({} blocos)",
        DatasetKey::MultiResPyramid => "Pirâmide de multi-resolução: {} MB",
        DatasetKey::CacheHitsMisses => "Acertos: {} | Falhas: {} (taxa de acertos: {}%)",
        DatasetKey::PrefetchInFlight => "Pré-carregamento em segundo plano: {} em andamento",
        DatasetKey::AllBlocksCached => "Buffer pronto / todos os blocos em cache",
        DatasetKey::SummaryPerVariable => "Resumo por variável:",
        DatasetKey::NoVariablesCached => "(Nenhuma variável em cache na memória)",
        DatasetKey::VariableCacheSize => "({} MB, {} blocos)",
        DatasetKey::ClearVariableCacheHover => "Limpar o cache desta variável",
        DatasetKey::PrefetchCapacitySettings => "Pré-carregamento e capacidade",
        DatasetKey::EnablePrefetching => "Ativar pré-carregamento em segundo plano",
        DatasetKey::CacheLimitSuffix => " MB de limite",
        DatasetKey::WindowSizeSuffix => " Tamanho da janela (passos)",
        DatasetKey::PrefetchWorkersSuffix => " Workers de pré-carregamento (threads)",
        DatasetKey::FlushAllCaches => "Esvaziar e limpar todos os caches",
        // Catalog
        DatasetKey::CatalogTitle => "Catálogo de conjuntos de dados",
        DatasetKey::CatalogStoreCount => "{} repositórios",
        DatasetKey::CatalogBlurb => {
            "Conjuntos de dados selecionados: Zarr e Icechunk na nuvem e dados procedurais de referência. Selecione um conjunto de dados para carregá-lo."
        }
        DatasetKey::CatalogSearchHint => "Filtrar por nome, descrição ou URL...",
        DatasetKey::CatalogCategory => "Categoria:",
        DatasetKey::CategoryAll => "Todos",
        DatasetKey::CategoryProcedural => "Procedural",
        DatasetKey::NoCatalogMatches => "Nenhum conjunto de dados corresponde à sua pesquisa.",
        DatasetKey::ClearSearch => "Limpar pesquisa",
        DatasetKey::SelectAndLoadDataset => "Selecionar e carregar conjunto de dados",
        DatasetKey::SelectAndLoad => "Selecionar e carregar",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "Tipo não suportado",
        DatasetKey::DropReleaseToLoad => "Solte para carregar o conjunto de dados",
        DatasetKey::DropSupportedList => "Suportados: {}",
        // Hero landing
        DatasetKey::HeroSeasFireHover => {
            "Conjunto de dados global de incêndios florestais e clima, reorganizado em chunks (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A em cor verdadeira, COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Volume espaço-temporal 4D sintético",
        DatasetKey::HeroCatalogHover => "Explorar o catálogo de conjuntos de dados",
        DatasetKey::LoadSampleHover => "Carregar exemplo: {}",
        DatasetKey::HeroDropLong => {
            "Solte o conjunto de dados para carregar (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Solte o conjunto de dados (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Solte o conjunto de dados para carregar",
        DatasetKey::HeroUnsupportedLong => {
            "Tipo não suportado. Suportados: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Formato não suportado (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Formato não suportado",
        DatasetKey::HeroInputHint => "Solte um conjunto de dados, insira caminho ou URL (.zarr, .icechunk, .nc, .h5, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL ou caminho...",
        DatasetKey::ClearInputHover => "Limpar entrada",
        DatasetKey::LoadDatasetHover => "Carregar conjunto de dados e abrir variáveis",
        DatasetKey::HopOctantHover => "Clique para saltar de octante",
        DatasetKey::HeroLoadingSource => "carregando: {}",
        DatasetKey::HeroLoading => "carregando...",
        DatasetKey::HeroLoaded => "carregado: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "Caminhos de arquivos locais não podem ser lidos diretamente no navegador por causa da segurança do sandbox da web.\n\nAbra com \"Abrir pasta\" ou \"Abrir arquivo\", ou arraste para a janela.\n\nPara ver conjuntos de dados locais no navegador:\n1. Sirva sua pasta ou arquivo com um servidor HTTP local: `npx serve` ou `python3 -m http.server`\n2. Digite a URL: `http://localhost:8000/my_dataset`\n\nOu execute a versão nativa para desktop do Octant (`cargo run --release`)."
        }
        DatasetKey::LocalNoZarrMetadata => {
            "Esta pasta não contém metadados Zarr que o navegador consiga ler. Escolha a pasta do próprio armazenamento (a que contém zarr.json ou .zgroup), não uma pasta acima dela. O Safari oculta arquivos cujo nome começa com um ponto, então pastas Zarr v2 (.zgroup, .zarray) só abrem no Chrome, no Firefox ou no aplicativo para desktop do Octant. Pastas Zarr v3 abrem em qualquer navegador."
        }
        DatasetKey::NeedsDataWorkers => {
            "Os arquivos NetCDF e HDF5 são lidos pelos workers de dados, que não iniciaram neste navegador. Recarregue a página ou abra o arquivo no aplicativo de desktop do Octant."
        }
        DatasetKey::MatArraysLeftOut => {
            "{} arrays deste arquivo MATLAB não são mostrados: texto, arrays de células, arrays vazios e esparsos, números complexos e objetos não são suportados"
        }
        DatasetKey::VariablesTooLargeForBrowser => {
            "{} variáveis deste arquivo são grandes demais para ler no navegador: abra-o no aplicativo de desktop do Octant para vê-las"
        }
        DatasetKey::GribMessagesLeftOut => {
            "{} mensagens deste arquivo GRIB não são mostradas: a grade, o produto ou o empacotamento não são suportados, ou repetem o tempo e o nível de outra mensagem"
        }
        DatasetKey::Plate => "Placa",
        DatasetKey::PlateAcquisition => "Aquisição",
        DatasetKey::PlateAllAcquisitions => "Todas",
        DatasetKey::PlateFields => "Campos:",
        DatasetKey::PlatePickWell => "Escolha um poço para ver seus campos",
        DatasetKey::PlateLoadingFields => "Lendo os campos do poço",
        DatasetKey::PlateNoFields => "Nenhum campo nesta aquisição",
        DatasetKey::PlateWellUnreadable => {
            "Não foi possível ler os campos do poço. Clique para tentar de novo"
        }
        DatasetKey::CategoryMicroscopy => "Microscopia",
        DatasetKey::CategorySoil => "Solo",
        DatasetKey::BadgeDesktopOnly => "Apenas desktop",
        DatasetKey::DesktopOnlyHint => {
            "Este servidor não permite o acesso pelo navegador. Abra-o no aplicativo de desktop do Octant."
        }
    })
}
