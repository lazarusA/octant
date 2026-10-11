//! Russian (Русский) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Набор данных",
        Key::Variables => "Переменные",
        Key::Dimensions => "Измерения",
        Key::Plot => "График",
        Key::Colormaps => "Цветовые шкалы",
        Key::Settings => "Настройки",
        Key::About => "О программе",

        Key::InspectMetadata => "Проверить метаданные",
        Key::ActiveDatasets => "Активные наборы данных",
        Key::NoActiveDatasets => "Нет активных наборов. Откройте набор данных для начала.",
        Key::CacheUsage => "Использование кэша",
        Key::ClearCache => "Очистить кэш",
        Key::OpenLocal => "Открыть локальный файл",
        Key::CloudStore => "Облачное хранилище",
        Key::SampleDatasets => "Примеры наборов данных",
        Key::DropFilesHere => "Перетащите файлы или папки сюда",
        Key::SourceUrlOrPath => "URI хранилища или локальный путь",
        Key::Inspect => "Проверить",
        Key::LoadingMetadata => "Проверка метаданных в фоновом режиме...",

        Key::Group => "Группа:",
        Key::DataType => "Тип",
        Key::Shape => "Форма",
        Key::Description => "Описание",
        Key::SearchVariables => "Поиск переменных...",
        Key::NoVariablesMatching => "Переменных по запросу не найдено",
        Key::ShowingItemsOfTotal => "Отображение элементов из общего числа",

        Key::Ready => "Готово",
        Key::Loading => "Загрузка...",
        Key::Close => "Закрыть",
        Key::Cancel => "Отмена",
        Key::Save => "Сохранить",
        Key::Reset => "Сброс",
        Key::Error => "Ошибка",

        Key::Play => "Воспроизвести",
        Key::Pause => "Пауза",
        Key::StepForward => "Шаг вперед",
        Key::StepBackward => "Шаг назад",
        Key::Speed => "Скорость",
        Key::Loop => "Цикл",

        Key::AboutSubtitle => "Обозреватель N-мерных данных",
        Key::AboutDescription => {
            "Octant — интерактивный просмотрщик N-мерных наборов данных с поддержкой локального и облачного хранения, Zarr (v2/v3) и Icechunk. Разработан на Rust с ускорением на GPU через WGPU. Работает на macOS, Linux и Windows."
        }
        Key::AboutHyperslabTitle => "Срезы гиперкубов",
        Key::AboutHyperslabDesc => "Асинхронный кэш фрагментов LRU и многомасштабные пирамиды.",
        Key::AboutStorageTitle => "Нативная поддержка Zarr и Icechunk",
        Key::AboutStorageDesc => "Локальные, S3, GCS, Azure и HTTP бэкенды потоковой передачи.",
        Key::AboutShadersTitle => "Аппаратно ускоренные шейдеры WGPU",
        Key::AboutShadersDesc => "2D-карты, 3D-сферы, поверхности высот, объемы и 1D-профили.",
        Key::AboutLinksHeading => "Ссылки и ресурсы",
        Key::AboutGoodFaithNotice => {
            "Octant добросовестно признает вклад сторонних авторов. Если что-то пропущено или указано неверно, сообщите нам для исправления."
        }
        Key::AboutBrowseIcons => "Просмотр векторных пиктограмм",
    })
}
