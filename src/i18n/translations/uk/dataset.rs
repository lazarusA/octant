use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Активний набір даних",
        DatasetKey::NoDatasetLoaded => "Жоден набір даних не завантажено",
        DatasetKey::MemoryCache => "Пам'ять і кеш",
        DatasetKey::SampleDatasets => "Зразки",
        DatasetKey::DropFilesHere => "Перетягніть файл або каталог",
        DatasetKey::SourceUrlOrPath => "Цільовий URL / шлях до файлу",
        DatasetKey::LoadingMetadata => "Перевірка метаданих сховища у фоновому режимі...",
        DatasetKey::OpenSource => "Відкрити джерело",
        DatasetKey::DatasetManager => "Менеджер наборів даних",
        DatasetKey::CloseDatasetHover => "Закрити набір даних",
        DatasetKey::TargetUriHover => "Цільовий URI:\n{}",
        DatasetKey::VariableCountOne => "Змінних: {}",
        DatasetKey::VariableCountMany => "Змінних: {}",
        DatasetKey::InspectVariables => "Переглянути змінні",
        DatasetKey::Reload => "Перезавантажити",
        DatasetKey::BadgeVolume4D => "4D-об'єм",
        DatasetKey::BadgeMatrix2D => "2D-матриця",
        DatasetKey::BadgeStore => "Сховище",
        DatasetKey::Load => "Завантажити",
        DatasetKey::TargetPathHint => "https://... або шлях (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Каталог...",
        DatasetKey::Clear => "Очистити",
        DatasetKey::FormatLabel => "Формат:",
        DatasetKey::SampleSeasFire => "SeasFire: глобальні пожежі (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "2D процедурна матриця (тест)",
        DatasetKey::FormatRemoteZarr => "Віддалений Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Локальний Zarr (файлова система)",
        DatasetKey::FormatRemoteIcechunk => "Віддалений Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Локальний Icechunk (файлова система)",
        DatasetKey::FormatRemoteGeoTiff => "Віддалений GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "Локальний GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "Локальний NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "4D еталонний об'єм (процедурний)",
        DatasetKey::FormatMatrix2D => "2D процедурна матриця (процедурна)",
        DatasetKey::FormatRemoteGeoTiffShort => "Віддалений GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "Локальний GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "Локальний NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "4D еталонний об'єм",
        DatasetKey::FormatMatrix2DShort => "2D процедурна матриця",
        DatasetKey::NoDatasetsRegistered => "У менеджері немає зареєстрованих наборів даних.",
        DatasetKey::LoadedCount => "Завантажено ({}):",
        DatasetKey::ClearAll => "Очистити все",
        DatasetKey::ClearAllHover => "Видалити всі набори даних з менеджера",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Видалити цей набір даних",
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} блоків, {}% влучань",
        DatasetKey::CacheMenuTitle => "Єдиний кеш пам'яті для всіх змінних",
        DatasetKey::GlobalMemoryUsage => "Загальне використання RAM:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Розподіл пам'яті за типом:",
        DatasetKey::ResidentHyperslabs => "Гіперзрізи в пам'яті: {} MB ({} блоків)",
        DatasetKey::MultiResPyramid => "Піраміда роздільностей: {} MB",
        DatasetKey::CacheHitsMisses => "Влучання кешу: {} | Промахи: {} (частка влучань: {}%)",
        DatasetKey::PrefetchInFlight => "Фонове попереднє завантаження: {} у процесі",
        DatasetKey::AllBlocksCached => "Буфер прогрітий / усі блоки в кеші",
        DatasetKey::SummaryPerVariable => "Підсумок за змінними:",
        DatasetKey::NoVariablesCached => "(Наразі жодна змінна не кешована в пам'яті)",
        DatasetKey::VariableCacheSize => "({} MB, {} блоків)",
        DatasetKey::ClearVariableCacheHover => "Очистити кеш цієї змінної",
        DatasetKey::PrefetchCapacitySettings => "Попереднє завантаження та обсяг",
        DatasetKey::EnablePrefetching => "Увімкнути фонове попереднє завантаження",
        DatasetKey::CacheLimitSuffix => " MB ліміт",
        DatasetKey::WindowSizeSuffix => " розмір вікна (кроки)",
        DatasetKey::PrefetchWorkersSuffix => " обробники завантаження (потоки)",
        DatasetKey::FlushAllCaches => "Скинути та очистити всі кеші",
        DatasetKey::CatalogTitle => "Каталог наборів даних",
        DatasetKey::CatalogStoreCount => "Сховищ: {}",
        DatasetKey::CatalogBlurb => {
            "Добірка хмарних Zarr, Icechunk та процедурних еталонних наборів даних. Виберіть будь-який набір, щоб завантажити."
        }
        DatasetKey::CatalogSearchHint => "Фільтр за назвою, описом або URL...",
        DatasetKey::CatalogCategory => "Категорія:",
        DatasetKey::CategoryAll => "Усі",
        DatasetKey::CategoryProcedural => "Процедурні",
        DatasetKey::NoCatalogMatches => "Не знайдено наборів даних за вашим запитом.",
        DatasetKey::ClearSearch => "Очистити пошук",
        DatasetKey::SelectAndLoadDataset => "Вибрати та завантажити набір",
        DatasetKey::SelectAndLoad => "Вибрати та завантажити",
        DatasetKey::DropTypeNotSupported => "Тип не підтримується",
        DatasetKey::DropReleaseToLoad => "Відпустіть, щоб завантажити набір даних",
        DatasetKey::DropSupportedList => "Підтримуються: {}",
        DatasetKey::HeroTryPrefix => "спробуйте:",
        DatasetKey::HeroSeasFireHover => {
            "Глобальний набір даних про пожежі та клімат із перерозбиттям на фрагменти (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A COG у справжніх кольорах (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Синтетичний 4D просторово-часовий об'єм",
        DatasetKey::HeroCatalogHover => "Переглянути каталог наборів даних",
        DatasetKey::LoadSampleHover => "Завантажити зразок: {}",
        DatasetKey::HeroHintLong => "вставте URL, локальний шлях або перетягніть файли будь-куди",
        DatasetKey::HeroHintMedium => "вставте URL, шлях або перетягніть файли",
        DatasetKey::HeroHintShort => "вставте URL або перетягніть файли",
        DatasetKey::HeroDropLong => {
            "Відпустіть набір даних, щоб завантажити (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Відпустіть набір даних (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Відпустіть, щоб завантажити",
        DatasetKey::HeroUnsupportedLong => {
            "Непідтримуваний тип. Підтримуються: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "Непідтримуваний формат (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "Непідтримуваний формат",
        DatasetKey::HeroInputHintLong => {
            "https://... або шлях (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... або шлях (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL або шлях...",
        DatasetKey::ClearInputHover => "Очистити поле",
        DatasetKey::LoadDatasetHover => "Завантажити набір даних і відкрити змінні",
        DatasetKey::HopOctantHover => "Клацніть, щоб перейти до іншого октанта",
        DatasetKey::HeroLoadingSource => "завантаження: {}",
        DatasetKey::HeroLoading => "завантаження...",
        DatasetKey::HeroLoaded => "завантажено: {}",
        DatasetKey::BrowserLocalPaths => {
            "Браузер не може читати локальні шляхи до файлів напряму через обмеження безпеки веб-пісочниці.\n\nЩоб переглянути локальні набори даних у браузері:\n1. Роздайте каталог або файл через локальний HTTP-сервер: `npx serve` або `python3 -m http.server`\n2. Введіть URL: `http://localhost:8000/my_dataset`\n\nАбо запустіть нативну настільну версію Octant (`cargo run --release`)."
        }
    })
}
