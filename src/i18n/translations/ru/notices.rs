use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "Скрыть",
        NoticesKey::CopyDetails => "Копировать подробности",
        NoticesKey::RevealInFinder => "Показать в Finder",
        NoticesKey::OpenFolder => "Открыть папку",
        NoticesKey::NoVariablesFound => "Переменные не найдены",
        NoticesKey::CouldntOpenDataset => "Не удалось открыть набор данных",
        NoticesKey::UnsupportedFile => "Неподдерживаемый файл",
        NoticesKey::CouldntLoadData => "Не удалось загрузить данные",
        NoticesKey::CoordinatesUnavailable => "Координаты недоступны",
        NoticesKey::CoordinatesFailedDetail => {
            "Не удалось загрузить координаты '{}' ({}); на осях показаны индексы"
        }
        NoticesKey::CoastlinesUnavailable => "Береговые линии недоступны",
        NoticesKey::VolumeTooLargeForGpu => "Объём слишком велик для GPU",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "Размер буфера объёма ({} элементов / {} байт) превышает максимальный лимит буфера хранения GPU ({})"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "Объём {}x{}x{} превышает лимит 3D-текстуры GPU ({})"
        }
        NoticesKey::VolumeUploadSkipped => "Загрузка объёма пропущена",
        NoticesKey::VolumeUploadMismatchDetail => "{} значений для текстуры {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "Данные слишком велики для буфера GPU",
        NoticesKey::BufferTooLargeDetail => {
            "{}: размер данных ({} байт) превышает ёмкость буфера GPU ({} байт), запись пропущена"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: диапазон с элемента {} (+{} элементов) превышает ёмкость буфера GPU ({} байт), запись пропущена"
        }
        NoticesKey::LinePlotTooLargeForGpu => "Линейный график слишком велик для GPU",
        NoticesKey::LinePayloadTooLargeDetail => {
            "Линии занимают {} байт; этот GPU связывает не более {}."
        }
        NoticesKey::FontLoadFailed => "Не удалось загрузить шрифт",
        NoticesKey::FontLoadFailedDetail => "Текст на языке {} может отображаться квадратами: {}",
        NoticesKey::DataFontMissing => "Нет шрифта для части текста набора данных",
        NoticesKey::DataFontMissingDetail => {
            "Текст на языке {} в этом наборе данных может отображаться квадратами. Установите шрифт, например Noto Sans CJK."
        }
        NoticesKey::OmeMetadataNonConforming => "Метаданные OME-Zarr не соответствуют спецификации",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Идентификаторы меток длиннее 32 бит делят цвета с меньшими"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Метаданные версии {} набора {} не соответствуют спецификации; показано то, что удалось прочитать."
        }
    })
}
