use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Про Octant",
        AboutKey::IconsWindowTitle => "Вбудовані векторні іконки",
        AboutKey::IconsTab => "Векторні іконки",
        AboutKey::Subtitle => "Переглядач N-вимірних даних",
        AboutKey::Description => {
            "Octant - інтерактивний переглядач N-вимірних наборів даних із вбудованою підтримкою локальних і хмарних об'єктних сховищ, Zarr (v2/v3) та Icechunk. Написаний на Rust з апаратно прискореним рендерингом через WGPU. Octant працює нативно на macOS, Linux та Windows."
        }
        AboutKey::HyperslabTitle => "Зрізи гіперкубів",
        AboutKey::HyperslabDesc => {
            "Асинхронний кеш фрагментів LRU та піраміди різної роздільної здатності."
        }
        AboutKey::StorageTitle => "Рідна підтримка Zarr та Icechunk",
        AboutKey::StorageDesc => "Локальні, S3, GCS, Azure та HTTP бекенди потокового передавання.",
        AboutKey::ShadersTitle => "Апаратно прискорені шейдери WGPU",
        AboutKey::ShadersDesc => "2D-карти, 3D-сфери, поверхні рельєфу, об'єми та 1D-профілі.",
        AboutKey::LinksHeading => "Посилання та ресурси",
        AboutKey::DocsLink => "Документація Octant",
        AboutKey::GoodFaithNotice => {
            "Octant добросовісно зазначає внесок сторонніх авторів. Якщо ви вважаєте, що щось пропущено або зазначено неправильно, повідомте нам, і ми це виправимо або вилучимо."
        }
        AboutKey::BrowseIcons => "Переглянути векторні іконки",
        AboutKey::BrowseIconsHover => "Переглянути процедурні векторні іконки у вікні",
        AboutKey::Licensed => "Ліцензія MIT або Apache-2.0",
        AboutKey::OctantTeaser => {
            "Знаки x, y та z (+/-, +/-, +/-) ділять простір на вісім октантів. Досліджуйте свої дані так само: оберіть осі, виріжте зріз і розгляньте одну область за раз."
        }
        AboutKey::OctantHover => "Натисніть, щоб перейти до наступного октанта",
        AboutKey::ColormapCredits => "Автори колірних шкал",
        AboutKey::ColormapCreditsNote => {
            "Вбудовані колірні шкали зберігають ліцензії своїх авторів."
        }
        AboutKey::LicenseTexts => "Тексти ліцензій",
        AboutKey::ThirdPartyLicenses => "Сторонні ліцензії",
        AboutKey::ThirdPartyNote => {
            "Octant містить відкриті crates, шрифти та бібліотеки C (netCDF, HDF5), кожна під власною ліцензією."
        }
        AboutKey::ViewOnGithub => "Переглянути на GitHub",
        AboutKey::IconsSearchHint => "Пошук іконок...",
        AboutKey::IconsAll => "Усі",
        AboutKey::IconsCopied => "Скопійовано {}",
        AboutKey::IconsNoMatch => "Відповідних векторних іконок не знайдено.",
        AboutKey::IconsCategory => "Категорія: {}",
        AboutKey::IconsClickToCopy => "Клацніть, щоб скопіювати ідентифікатор enum",
        AboutKey::IconsSizesTones => "Розміри та тони",
        AboutKey::ClearSearchHover => "Очистити пошук",
        AboutKey::FontLicensesNote => {
            "Noto Sans SC, JP і KR (текст китайською, японською та корейською):"
        }
    })
}
