use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Dispensar",
        NoticesKey::CopyDetails => "Copiar detalhes",
        NoticesKey::RevealInFinder => "Mostrar no Finder",
        NoticesKey::OpenFolder => "Abrir pasta",
        NoticesKey::NoVariablesFound => "Nenhuma variável encontrada",
        NoticesKey::CouldntOpenDataset => "Não foi possível abrir o conjunto de dados",
        NoticesKey::UnsupportedFile => "Arquivo não suportado",
        NoticesKey::CouldntLoadData => "Não foi possível carregar os dados",
        NoticesKey::CoordinatesUnavailable => "Coordenadas indisponíveis",
        NoticesKey::CoordinatesFailedDetail => {
            "As coordenadas de '{}' não puderam ser carregadas ({}); os eixos mostram índices"
        }
        NoticesKey::CoastlinesUnavailable => "Litorais indisponíveis",
        NoticesKey::VolumeTooLargeForGpu => "Volume grande demais para a GPU",
        NoticesKey::VolumeLodOff => "Detalhe conforme a vista está desligado para este volume",
        NoticesKey::VolumeLodOffDetail => {
            "O nível representado precisa de {} blocos; o atlas comporta {} e reserva metade para os mais finos. O volume é desenhado com um único nível inteiro."
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "Os seus níveis não são múltiplos inteiros uns dos outros com células alinhadas, como os blocos exigem. O volume é desenhado com um nível inteiro."
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "Os seus valores inteiros (como ids de rótulos) abrangem um intervalo amplo demais para os blocos os distinguirem. O volume é desenhado com um nível inteiro."
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "Esta GPU não consegue escrever a partir do shader do volume, como os blocos exigem. O volume é desenhado com um nível inteiro."
        }
        NoticesKey::VolumeLodBrickFailed => "Parte do volume não pôde ser lida num nível mais fino",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "O tamanho do buffer de dados do volume ({} elementos / {} bytes) excede o limite máximo do buffer de armazenamento da GPU ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "O volume {}x{}x{} excede o limite de textura 3D da GPU ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Envio do volume ignorado",
        NoticesKey::VolumeUploadMismatchDetail => "{} valores para uma textura de {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "Dados grandes demais para o buffer da GPU",
        NoticesKey::BufferTooLargeDetail => {
            "{}: o tamanho dos dados ({} bytes) excede a capacidade do buffer da GPU ({} bytes); gravação ignorada"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: o intervalo no elemento {} (+{} elementos) excede a capacidade do buffer da GPU ({} bytes); gravação ignorada"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Gráfico de linhas grande demais para a GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "As linhas ocupam {} bytes; esta GPU vincula no máximo {}."
        }
        NoticesKey::FontLoadFailed => "Não foi possível carregar uma fonte",
        NoticesKey::FontLoadFailedDetail => "O texto em {} pode aparecer como quadrados: {}",
        NoticesKey::OmeMetadataNonConforming => "Os metadados OME-Zarr não estão em conformidade",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Ids de rótulo acima de 32 bits compartilham cores com ids menores"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Os metadados da versão {} de {} não seguem a especificação; mostrando o que foi possível ler."
        }
        NoticesKey::PlateWellStopped => "Poço {}: a leitura foi interrompida",
        NoticesKey::PlateWellNotJson => "Poço {}: os metadados não são JSON válido",
        NoticesKey::PlateWellNotOme => "Poço {}: não é um poço OME-Zarr",
        NoticesKey::PlateWellNoMetadata => "Poço {}: sem metadados de poço",
    })
}
