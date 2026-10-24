use crate::i18n::keys::KeymapKey;

pub fn text(key: KeymapKey) -> Option<&'static str> {
    Some(match key {
        KeymapKey::Title => "Сочетания клавиш",
        KeymapKey::CloseHover => "Закрыть сочетания клавиш (Esc)",
        KeymapKey::ScopeGlobal => "Везде",
        KeymapKey::ScopePanel => "В панели",
        KeymapKey::ScopeFree => "Вне текстовых полей",
        KeymapKey::ScopeCanvas => "На графике",
        KeymapKey::ScopePlayback => "Воспроизведение",
        KeymapKey::Arrows => "Стрелки",
        KeymapKey::Hint => "{} ({})",
        KeymapKey::QuickSave => "Сохранить рисунок",
        KeymapKey::ExportDialog => "Экспортировать рисунок с настройками",
        KeymapKey::ToggleCrop => "Показать/скрыть рамку обрезки",
        KeymapKey::GoToDataset => "Перейти к панели набора данных",
        KeymapKey::GoToVariables => "Перейти к панели переменных",
        KeymapKey::GoToDimensions => "Перейти к панели измерений",
        KeymapKey::GoToSettings => "Перейти к настройкам",
        KeymapKey::SearchVariables => "Искать переменные",
        KeymapKey::ClosePanel => "Закрыть панель",
        KeymapKey::NextRegion => "Перейти к следующей области",
        KeymapKey::PreviousRegion => "Перейти к предыдущей области",
        KeymapKey::FocusTopBar => "Перейти к верхней панели",
        KeymapKey::PlayPause => "Воспроизвести или приостановить",
        KeymapKey::StepPrev => "Предыдущий шаг",
        KeymapKey::StepNext => "Следующий шаг",
        KeymapKey::FirstStep => "Первый шаг",
        KeymapKey::LastStep => "Последний шаг",
        KeymapKey::ToggleLoop => "Включить/выключить повтор",
        KeymapKey::Move => {
            "Сдвинуть (2D) или повернуть (3D) график; переместить зонд или рамку обрезки"
        }
        KeymapKey::MoveAlt => {
            "Сдвинуть дальше (2D) или сдвинуть (3D); изменить размер рамки обрезки"
        }
        KeymapKey::ZoomIn => "Приблизить",
        KeymapKey::ZoomOut => "Отдалить",
        KeymapKey::ResetView => "Сбросить вид",
        KeymapKey::ToggleProbe => "Считывать значения стрелками",
        KeymapKey::LeaveProbe => "Прекратить считывание",
        KeymapKey::ShowShortcuts => "Показать сочетания клавиш",
        KeymapKey::ToggleHero => "Показать начальный экран или снова график",
        KeymapKey::LeaveHero => "Вернуться к графику",
        KeymapKey::ScopeHero => "На начальном экране",
        KeymapKey::PlotTypePicker => "Выбрать тип графика",
        KeymapKey::TypeCommand => "Ввести голосовую команду",
        KeymapKey::PlotStaged => "Построить выбранную переменную",
        KeymapKey::GoToColorbar => "Перейти к следующей цветовой шкале",
    })
}
