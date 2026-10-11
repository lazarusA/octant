//! Ukrainian (Українська) translations for Octant.

use crate::i18n::keys::Key;

pub fn translate(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::Dataset => "Набір даних",
        Key::Variables => "Змінні",
        Key::Dimensions => "Виміри",
        Key::Plot => "Графік",
        Key::Colormaps => "Колірні шкали",
        Key::Settings => "Налаштування",
        Key::About => "Про програму",

        Key::InspectMetadata => "Перевірити метадані",
        Key::ActiveDatasets => "Активні набори даних",
        Key::NoActiveDatasets => "Немає активних наборів. Відкрийте набір даних для початку.",
        Key::CacheUsage => "Використання кешу",
        Key::ClearCache => "Очистити кеш",
        Key::OpenLocal => "Відкрити локальний файл",
        Key::CloudStore => "Хмарне сховище",
        Key::SampleDatasets => "Зразки даних",
        Key::DropFilesHere => "Перетягніть файли або папки сюди",
        Key::SourceUrlOrPath => "URI сховища або локальний шлях",
        Key::Inspect => "Перевірити",
        Key::LoadingMetadata => "Перевірка метаданих у фоновому режимі...",

        Key::Group => "Група:",
        Key::DataType => "Тип",
        Key::Shape => "Форма",
        Key::Description => "Опис",
        Key::SearchVariables => "Пошук змінних...",
        Key::NoVariablesMatching => "Змінних за запитом не знайдено",
        Key::ShowingItemsOfTotal => "Відображення елементів із загальної кількості",

        Key::Ready => "Готово",
        Key::Loading => "Завантаження...",
        Key::Close => "Закрити",
        Key::Cancel => "Скасувати",
        Key::Save => "Зберегти",
        Key::Reset => "Скинути",
        Key::Error => "Помилка",

        Key::Play => "Відтворити",
        Key::Pause => "Пауза",
        Key::StepForward => "Крок уперед",
        Key::StepBackward => "Крок назад",
        Key::Speed => "Швидкість",
        Key::Loop => "Цикл",

        Key::AboutSubtitle => "Провідник N-вимірних даних",
        Key::AboutDescription => {
            "Octant — інтерактивний переглядач N-вимірних наборів даних із підтримкою локального та хмарного сховища, Zarr (v2/v3) та Icechunk. Розроблено на Rust з апаратним рендерингом через WGPU. Працює на macOS, Linux та Windows."
        }
        Key::AboutHyperslabTitle => "Зрізи гіперкубів",
        Key::AboutHyperslabDesc => {
            "Асинхронний кеш фрагментів LRU та піраміди різної роздільної здатності."
        }
        Key::AboutStorageTitle => "Рідна підтримка Zarr та Icechunk",
        Key::AboutStorageDesc => "Локальні, S3, GCS, Azure та HTTP бекенди потокового передавання.",
        Key::AboutShadersTitle => "Апаратно прискорені шейдери WGPU",
        Key::AboutShadersDesc => "2D-карти, 3D-сфери, поверхні рельєфу, об'єми та 1D-профілі.",
        Key::AboutLinksHeading => "Посилання та ресурси",
        Key::AboutGoodFaithNotice => {
            "Octant добросовісно визнає внесок сторонніх авторів. Якщо щось пропущено або невірно зазначено, повідомте нам для виправлення."
        }
        Key::AboutBrowseIcons => "Переглянути векторні піктограми",
    })
}
