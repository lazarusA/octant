use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Набір даних",
        CommonKey::Variables => "Змінні",
        CommonKey::Dimensions => "Виміри",
        CommonKey::Settings => "Налаштування",
        CommonKey::About => "Про програму",
        CommonKey::Cache => "Кеш",
        CommonKey::Language => "Мова",
        CommonKey::Loading => "Завантаження...",
        CommonKey::CloseEsc => "Закрити (Esc)",
        CommonKey::Cancel => "Скасувати",
        CommonKey::Save => "Зберегти",
        CommonKey::Reset => "Скинути",
        CommonKey::Move => "Перемістити",
        CommonKey::MoveHover => "Перетягніть, щоб перемістити; двічі клацніть, щоб скинути",
        CommonKey::Light => "Світла",
        CommonKey::Dark => "Темна",
        CommonKey::SwitchToLightHover => "Перейти до світлої теми",
        CommonKey::SwitchToDarkHover => "Перейти до темної теми",
        CommonKey::ToggleHeroHover => {
            "Показати/сховати стартовий екран (правий клік - Про програму)"
        }
        CommonKey::AboutOctantHover => "Про Octant",
        CommonKey::ToggleVariableControlsHover => "Показати/сховати панель змінної",
        CommonKey::FetchingProgress => "Отримання {} / {} ({}%)",
        CommonKey::Abort => "Перервати",
        CommonKey::AbortHover => "Перервати поточне передавання даних",
    })
}
