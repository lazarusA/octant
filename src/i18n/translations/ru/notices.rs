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
        NoticesKey::NoDataAtStep => "Нет данных на этом шаге",
        NoticesKey::NoDataAtStepDetail => "У '{}' нет значений на этом шаге: в хранилище здесь нет данных. Попробуйте другой шаг.",
        NoticesKey::LargeChunks => "Крупные фрагменты",
        NoticesKey::LargeChunksDetail => "'{}' хранится фрагментами по {}: каждый шаг загружает как минимум один фрагмент целиком, поэтому шаги загружаются медленно.",
        NoticesKey::VolumeTooLargeForGpu => "Объём слишком велик для GPU",
        NoticesKey::VolumeLodOff => "Детализация по виду выключена для этого объёма",
        NoticesKey::VolumeLodOffDetail => {
            "Отображаемому уровню нужно {} блоков; атлас вмещает {} и оставляет половину для более детальных. Объём рисуется одним целым уровнем."
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "Его уровни не кратны друг другу целым числом с выровненными ячейками, как требуют блоки. Объём рисуется одним целым уровнем."
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "Его целые значения (например, идентификаторы меток) охватывают слишком широкий диапазон, чтобы блоки их различали. Объём рисуется одним целым уровнем."
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "Этот GPU не может записывать из шейдера объёма, как требуют блоки. Объём рисуется одним целым уровнем."
        }
        NoticesKey::VolumeLodBrickFailed => "Часть объёма не удалось прочитать на более детальном уровне",
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
        NoticesKey::OmeMetadataNonConforming => "Метаданные OME-Zarr не соответствуют спецификации",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Идентификаторы меток шире 32 бит имеют общие цвета с меньшими"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Метаданные версии {} набора {} не соответствуют спецификации; показано то, что удалось прочитать."
        }
        NoticesKey::PlateWellStopped => "Лунка {}: чтение прервано",
        NoticesKey::PlateWellNotJson => "Лунка {}: метаданные не являются корректным JSON",
        NoticesKey::PlateWellNotOme => "Лунка {}: это не лунка OME-Zarr",
        NoticesKey::PlateWellNoMetadata => "Лунка {}: нет метаданных лунки",
    })
}
