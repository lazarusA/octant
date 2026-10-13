use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "Активный набор данных",
        DatasetKey::NoDatasetLoaded => "Набор данных не загружен",
        DatasetKey::MemoryCache => "Память и кэш",
        DatasetKey::SampleDatasets => "Примеры",
        DatasetKey::DropFilesHere => "Перетащите файл или папку",
        DatasetKey::SourceUrlOrPath => "URL / путь к файлу",
        DatasetKey::LoadingMetadata => "Чтение метаданных хранилища в фоновом режиме...",
        DatasetKey::OpenSource => "Открыть источник",
        DatasetKey::DatasetManager => "Менеджер наборов данных",
        DatasetKey::CloseDatasetHover => "Закрыть набор данных",
        DatasetKey::TargetUriHover => "URI:\n{}",
        DatasetKey::VariableCountOne => "{} переменная",
        DatasetKey::VariableCountMany => "Переменных: {}",
        DatasetKey::InspectVariables => "Обзор переменных",
        DatasetKey::Reload => "Перезагрузить",
        DatasetKey::BadgeVolume4D => "4D-объём",
        DatasetKey::BadgeMatrix2D => "2D-матрица",
        DatasetKey::BadgeStore => "Хранилище",
        DatasetKey::Load => "Загрузить",
        DatasetKey::TargetPathHint => "https://... или путь (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "Каталог...",
        DatasetKey::Clear => "Очистить",
        DatasetKey::FormatLabel => "Формат:",
        DatasetKey::SampleSeasFire => "SeasFire: глобальные лесные пожары (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "Процедурная 2D-матрица (тест)",
        DatasetKey::FormatRemoteZarr => "Удалённый Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "Локальный Zarr (файловая система)",
        DatasetKey::FormatRemoteIcechunk => "Удалённый Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "Локальный Icechunk (файловая система)",
        DatasetKey::FormatRemoteGeoTiff => "Удалённый GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "Локальный GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "Локальный NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "Эталонный 4D-объём (процедурный)",
        DatasetKey::FormatMatrix2D => "Процедурная 2D-матрица (процедурная)",
        DatasetKey::FormatRemoteGeoTiffShort => "Удалённый GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "Локальный GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "Локальный NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "Эталонный 4D-объём",
        DatasetKey::FormatMatrix2DShort => "Процедурная 2D-матрица",
        DatasetKey::NoDatasetsRegistered => "В менеджере нет зарегистрированных наборов данных.",
        DatasetKey::LoadedCount => "Загружено ({}):",
        DatasetKey::ClearAll => "Очистить всё",
        DatasetKey::ClearAllHover => "Удалить все наборы данных из менеджера",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "Удалить этот набор данных",
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "Блоков: {}, попаданий: {}%",
        DatasetKey::CacheMenuTitle => "Общий кэш памяти для всех переменных",
        DatasetKey::GlobalMemoryUsage => "Общее использование RAM:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "Память по типам:",
        DatasetKey::ResidentHyperslabs => "Гиперсрезы в памяти: {} MB (блоков: {})",
        DatasetKey::MultiResPyramid => "Многомасштабная пирамида: {} MB",
        DatasetKey::CacheHitsMisses => "Попадания: {} | Промахи: {} (доля попаданий: {}%)",
        DatasetKey::PrefetchInFlight => "Фоновая предзагрузка: {} в процессе",
        DatasetKey::AllBlocksCached => "Буфер прогрет / все блоки в кэше",
        DatasetKey::SummaryPerVariable => "Сводка по переменным:",
        DatasetKey::NoVariablesCached => "(Нет переменных в кэше памяти)",
        DatasetKey::VariableCacheSize => "({} MB, блоков: {})",
        DatasetKey::ClearVariableCacheHover => "Очистить кэш этой переменной",
        DatasetKey::PrefetchCapacitySettings => "Предзагрузка и объём",
        DatasetKey::EnablePrefetching => "Включить фоновую предзагрузку",
        DatasetKey::CacheLimitSuffix => " MB - лимит",
        DatasetKey::WindowSizeSuffix => " - размер окна (шаги)",
        DatasetKey::PrefetchWorkersSuffix => " - потоки предзагрузки",
        DatasetKey::FlushAllCaches => "Сбросить и очистить все кэши",
        DatasetKey::CatalogTitle => "Каталог наборов данных",
        DatasetKey::CatalogStoreCount => "Хранилищ: {}",
        DatasetKey::CatalogBlurb => {
            "Подборка облачных наборов Zarr, Icechunk и процедурных эталонных данных. Выберите набор данных для загрузки."
        }
        DatasetKey::CatalogSearchHint => "Фильтр по названию, описанию или URL...",
        DatasetKey::CatalogCategory => "Категория:",
        DatasetKey::CategoryAll => "Все",
        DatasetKey::CategoryProcedural => "Процедурные",
        DatasetKey::NoCatalogMatches => "Наборы данных по вашему запросу не найдены.",
        DatasetKey::ClearSearch => "Очистить поиск",
        DatasetKey::SelectAndLoadDataset => "Выбрать и загрузить набор данных",
        DatasetKey::SelectAndLoad => "Выбрать и загрузить",
        DatasetKey::DropTypeNotSupported => "Тип не поддерживается",
        DatasetKey::DropReleaseToLoad => "Отпустите, чтобы загрузить набор данных",
        DatasetKey::DropSupportedList => "Поддерживаются: {}",
        DatasetKey::HeroTryPrefix => "попробуйте:",
        DatasetKey::HeroSeasFireHover => {
            "Глобальные данные о пожарах и климате, перефрагментированные (Zarr)"
        }
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A COG в естественных цветах (AWS S3)",
        DatasetKey::HeroVolume4DHover => "Синтетический пространственно-временной 4D-объём",
        DatasetKey::HeroCatalogHover => "Открыть каталог наборов данных",
        DatasetKey::LoadSampleHover => "Загрузить пример: {}",
        DatasetKey::HeroHintLong => "URL / локальный путь / перетащите файлы куда угодно",
        DatasetKey::HeroHintMedium => "URL / путь / перетащите файлы",
        DatasetKey::HeroHintShort => "URL / перетащите файлы",
        DatasetKey::HeroDropLong => {
            "Отпустите набор данных для загрузки (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "Отпустите набор данных (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "Отпустите для загрузки",
        DatasetKey::HeroUnsupportedLong => {
            "Тип не поддерживается. Поддерживаются: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => {
            "Формат не поддерживается (.nc, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroUnsupportedShort => "Формат не поддерживается",
        DatasetKey::HeroInputHintLong => {
            "https://... или путь (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... или путь (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL или путь...",
        DatasetKey::ClearInputHover => "Очистить поле",
        DatasetKey::LoadDatasetHover => "Загрузить набор данных и открыть переменные",
        DatasetKey::HopOctantHover => "Нажмите, чтобы перейти в другой октант",
        DatasetKey::HeroLoadingSource => "загрузка: {}",
        DatasetKey::HeroLoading => "загрузка...",
        DatasetKey::HeroLoaded => "загружено: {}",
        DatasetKey::BrowserLocalPaths => {
            "Браузер не может читать локальные файлы напрямую из-за ограничений безопасности веб-песочницы.\n\nЧтобы открыть локальные наборы данных в браузере:\n1. Раздайте папку или файл через локальный HTTP-сервер: `npx serve` или `python3 -m http.server`\n2. Введите URL: `http://localhost:8000/my_dataset`\n\nИли запустите настольную версию Octant (`cargo run --release`)."
        }
        DatasetKey::Plate => "Планшет",
        DatasetKey::PlateAcquisition => "Съемка",
        DatasetKey::PlateAllAcquisitions => "Все",
        DatasetKey::PlateFields => "Поля:",
        DatasetKey::PlatePickWell => "Выберите лунку, чтобы увидеть ее поля",
        DatasetKey::PlateLoadingFields => "Чтение полей лунки",
        DatasetKey::PlateNoFields => "В этой съемке нет полей",
        DatasetKey::CategoryMicroscopy => "Микроскопия",
    })
}
