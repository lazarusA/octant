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
        NoticesKey::NoDataAtStep => "Немає даних на цьому кроці",
        NoticesKey::NoDataAtStepDetail => "У '{}' немає значень на цьому кроці: у сховищі тут немає даних. Спробуйте інший крок.",
        NoticesKey::LargeChunks => "Великі фрагменти",
        NoticesKey::LargeChunksDetail => "'{}' зберігається фрагментами по {}: кожен крок завантажує щонайменше один фрагмент повністю, тому кроки завантажуються повільно.",
        NoticesKey::VolumeTooLargeForGpu => "Об'єм завеликий для GPU",
        NoticesKey::VolumeLodOff => "Деталізацію за видом вимкнено для цього об'єму",
        NoticesKey::VolumeLodOffDetail => {
            "Відображеному рівню потрібно {} блоків; атлас вміщує {} і лишає половину для детальніших. Об'єм малюється одним цілим рівнем."
        }
        NoticesKey::VolumeLodUnevenDetail => {
            "Його рівні не кратні один одному цілим числом із вирівняними комірками, як потребують блоки. Об'єм малюється одним цілим рівнем."
        }
        NoticesKey::VolumeLodWideIntegersDetail => {
            "Його цілі значення (наприклад, ідентифікатори міток) охоплюють надто широкий діапазон, щоб блоки їх розрізняли. Об'єм малюється одним цілим рівнем."
        }
        NoticesKey::VolumeLodUnsupportedDetail => {
            "Цей GPU не може записувати з шейдера об'єму, як потребують блоки. Об'єм малюється одним цілим рівнем."
        }
        NoticesKey::VolumeLodBrickFailed => "Частину об'єму не вдалося прочитати на детальнішому рівні",
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
        NoticesKey::OmeMetadataNonConforming => "Метадані OME-Zarr не відповідають специфікації",
        NoticesKey::LabelIdsBeyond32Bits => {
            "Ідентифікатори міток ширші за 32 біти мають спільні кольори з меншими"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "Метадані версії {} набору {} не відповідають специфікації; показано те, що вдалося прочитати."
        }
        NoticesKey::PlateWellStopped => "Лунка {}: читання перервано",
        NoticesKey::PlateWellNotJson => "Лунка {}: метадані не є коректним JSON",
        NoticesKey::PlateWellNotOme => "Лунка {}: це не лунка OME-Zarr",
        NoticesKey::PlateWellNoMetadata => "Лунка {}: немає метаданих лунки",
    })
}
