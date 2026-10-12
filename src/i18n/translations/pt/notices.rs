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
        NoticesKey::DataFontMissing => "Falta uma fonte para parte do texto do conjunto de dados",
        NoticesKey::DataFontMissingDetail => {
            "O texto em {} deste conjunto de dados pode aparecer como quadrados. Instale uma fonte como Noto Sans CJK."
        }
    })
}
