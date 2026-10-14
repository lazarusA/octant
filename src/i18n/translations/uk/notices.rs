use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Закрити",
        NoticesKey::CopyDetails => "Копіювати деталі",
        NoticesKey::RevealInFinder => "Показати у Finder",
        NoticesKey::OpenFolder => "Відкрити папку",
        NoticesKey::NoVariablesFound => "Змінних не знайдено",
        NoticesKey::CouldntOpenDataset => "Не вдалося відкрити набір даних",
        NoticesKey::UnsupportedFile => "Непідтримуваний файл",
        NoticesKey::CouldntLoadData => "Не вдалося завантажити дані",
        NoticesKey::CoordinatesUnavailable => "Координати недоступні",
        NoticesKey::CoordinatesFailedDetail => {
            "Координати '{}' не вдалося завантажити ({}); осі показують індекси"
        }
        NoticesKey::CoastlinesUnavailable => "Берегові лінії недоступні",
        NoticesKey::VolumeTooLargeForGpu => "Об'єм завеликий для GPU",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "Розмір буфера даних об'єму ({} елементів / {} байтів) перевищує максимальний ліміт буфера зберігання GPU ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Об'єм {}x{}x{} перевищує ліміт 3D-текстури GPU ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Передавання об'єму пропущено",
        NoticesKey::VolumeUploadMismatchDetail => "{} значень для текстури {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "Дані завеликі для буфера GPU",
        NoticesKey::BufferTooLargeDetail => {
            "{}: розмір даних ({} байтів) перевищує місткість буфера GPU ({} байтів), запис пропущено"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: діапазон з елемента {} (+{} елементів) перевищує місткість буфера GPU ({} байтів), запис пропущено"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Лінійний графік завеликий для GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Лінії займають {} байтів; цей GPU прив'язує щонайбільше {}."
        }
        NoticesKey::FontLoadFailed => "Не вдалося завантажити шрифт",
        NoticesKey::FontLoadFailedDetail => "Текст мовою {} може відображатися квадратами: {}",
        NoticesKey::DataFontMissing => "Немає шрифту для частини тексту набору даних",
        NoticesKey::DataFontMissingDetail => {
            "Текст мовою {} у цьому наборі даних може відображатися квадратами. Установіть шрифт, наприклад Noto Sans CJK."
        }
        NoticesKey::OmeMetadataNonConforming => "Метадані OME-Zarr не відповідають специфікації",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Ідентифікатори міток ширші за 32 біти мають спільні кольори з меншими"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Метадані версії {} набору {} не відповідають специфікації; показано те, що вдалося прочитати."
        }
    })
}
