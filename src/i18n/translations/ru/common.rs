use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "Набор данных",
        CommonKey::Variables => "Переменные",
        CommonKey::Dimensions => "Измерения",
        CommonKey::Settings => "Настройки",
        CommonKey::About => "О программе",
        CommonKey::Cache => "Кэш",
        CommonKey::Language => "Язык",
        CommonKey::Loading => "Загрузка...",
        CommonKey::CloseEsc => "Закрыть (Esc)",
        CommonKey::Cancel => "Отмена",
        CommonKey::Save => "Сохранить",
        CommonKey::Reset => "Сброс",
        CommonKey::Move => "Переместить",
        CommonKey::MoveHover => "Перетащите, чтобы переместить; двойной щелчок - сброс",
        CommonKey::Light => "Светлая",
        CommonKey::Dark => "Тёмная",
        CommonKey::SwitchToLightHover => "Переключить на светлую тему",
        CommonKey::SwitchToDarkHover => "Переключить на тёмную тему",
        CommonKey::ToggleHeroHover => {
            "Показать/скрыть стартовый экран (правый щелчок - О программе)"
        }
        CommonKey::AboutOctantHover => "О программе Octant",
        CommonKey::ToggleVariableControlsHover => "Показать/скрыть панель переменной",
        CommonKey::FetchingProgress => "Загрузка {} / {} ({}%)",
        CommonKey::Abort => "Прервать",
        CommonKey::AbortHover => "Прервать текущую передачу данных",
    })
}
