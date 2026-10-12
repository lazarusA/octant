use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "О программе Octant",
        AboutKey::IconsWindowTitle => "Встроенные векторные иконки",
        AboutKey::IconsTab => "Векторные иконки",
        AboutKey::Subtitle => "Обозреватель N-мерных данных",
        AboutKey::Description => {
            "Octant - интерактивный просмотрщик N-мерных наборов данных со встроенной поддержкой локальных и облачных объектных хранилищ, Zarr (v2/v3) и Icechunk. Написан на Rust, отрисовка ускорена на GPU через WGPU. Octant работает нативно на macOS, Linux и Windows."
        }
        AboutKey::HyperslabTitle => "Срезы гиперкубов",
        AboutKey::HyperslabDesc => "Асинхронный кэш фрагментов LRU и многомасштабные пирамиды.",
        AboutKey::StorageTitle => "Нативная поддержка Zarr и Icechunk",
        AboutKey::StorageDesc => "Локальные, S3, GCS, Azure и HTTP бэкенды потоковой передачи.",
        AboutKey::ShadersTitle => "Аппаратно ускоренные шейдеры WGPU",
        AboutKey::ShadersDesc => "2D-карты, 3D-сферы, поверхности высот, объёмы и 1D-профили.",
        AboutKey::LinksHeading => "Ссылки и ресурсы",
        AboutKey::DocsLink => "Документация Octant",
        AboutKey::GoodFaithNotice => {
            "Octant добросовестно указывает авторство сторонних работ. Если вы считаете, что что-то пропущено или указано неверно, сообщите нам, и мы исправим или удалим это."
        }
        AboutKey::BrowseIcons => "Просмотр векторных иконок",
        AboutKey::BrowseIconsHover => {
            "Просмотреть процедурные векторные иконки во всплывающем окне"
        }
        AboutKey::Licensed => "Лицензия MIT или Apache-2.0",
        AboutKey::OctantTeaser => {
            "Знаки x, y и z (+/-, +/-, +/-) делят пространство на восемь октантов. Исследуйте свои данные так же: выберите оси, вырежьте срез и рассмотрите одну область за раз."
        }
        AboutKey::OctantHover => "Нажмите, чтобы перейти к следующему октанту",
        AboutKey::ColormapCredits => "Авторы цветовых шкал",
        AboutKey::ColormapCreditsNote => {
            "Встроенные цветовые шкалы сохраняют лицензии своих авторов."
        }
        AboutKey::LicenseTexts => "Тексты лицензий",
        AboutKey::ThirdPartyLicenses => "Сторонние лицензии",
        AboutKey::ThirdPartyNote => {
            "Octant включает открытые crate-пакеты, шрифты и C-библиотеки (netCDF, HDF5), каждый под своей лицензией."
        }
        AboutKey::ViewOnGithub => "Открыть на GitHub",
        AboutKey::IconsSearchHint => "Поиск иконок...",
        AboutKey::IconsAll => "Все",
        AboutKey::IconsCopied => "Скопировано: {}",
        AboutKey::IconsNoMatch => "Подходящие векторные иконки не найдены.",
        AboutKey::IconsCategory => "Категория: {}",
        AboutKey::IconsClickToCopy => "Нажмите, чтобы скопировать идентификатор enum",
        AboutKey::IconsSizesTones => "Размеры и тона",
        AboutKey::ClearSearchHover => "Очистить поиск",
        AboutKey::FontLicensesNote => {
            "Noto Sans SC, JP и KR (текст на китайском, японском и корейском):"
        }
    })
}
