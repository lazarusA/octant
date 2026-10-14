use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Воспроизвести",
        PlaybackKey::Pause => "Пауза",
        PlaybackKey::Loop => "Цикл",
        PlaybackKey::FirstStep => "Первый шаг",
        PlaybackKey::PreviousStep => "Предыдущий шаг",
        PlaybackKey::NextStep => "Следующий шаг",
        PlaybackKey::LastStep => "Последний шаг",
        PlaybackKey::Crop => "Обрезка",
        PlaybackKey::CropHover => "Направляющие обрезки (C)",
        PlaybackKey::PlaybackSpeed => "Скорость воспроизведения",
        PlaybackKey::PlaybackBuffer => "Буферизация перед воспроизведением",
        PlaybackKey::Buffering => "Буферизация {}/{}",
        PlaybackKey::SaveExportHover => {
            "Сохранить / экспортировать рисунок (Cmd+S - быстрое сохранение, Cmd+Shift+S - настройки)"
        }
        PlaybackKey::CollapsedHint => "Воспроизведение (нажмите или потяните, чтобы развернуть)",
        PlaybackKey::TimelineDetails => "Сведения о шкале времени",
        PlaybackKey::TimelineDimension => "Измерение: {}",
        PlaybackKey::TimelineStep => "Шаг: {} / {}",
        PlaybackKey::TimelineCurrent => "Текущее: {}",
        PlaybackKey::TimelineRange => "Диапазон: {} -> {}",
        PlaybackKey::TimelineStepSize => "Размер шага: {}",
        PlaybackKey::DefaultStepSize => "Шаг: 1",
    })
}
