use crate::i18n::keys::PlaybackKey;

pub fn text(key: PlaybackKey) -> Option<&'static str> {
    Some(match key {
        PlaybackKey::Play => "Відтворити",
        PlaybackKey::Pause => "Пауза",
        PlaybackKey::Loop => "Цикл",
        PlaybackKey::FirstStep => "Перший крок",
        PlaybackKey::PreviousStep => "Попередній крок",
        PlaybackKey::NextStep => "Наступний крок",
        PlaybackKey::LastStep => "Останній крок",
        PlaybackKey::Crop => "Обрізати",
        PlaybackKey::CropHover => "Напрямні лінії обрізання (C)",
        PlaybackKey::PlaybackSpeed => "Швидкість відтворення",
        PlaybackKey::SaveExportHover => {
            "Зберегти / експортувати рисунок (Cmd+S - швидке збереження, Cmd+Shift+S - налаштування)"
        }
        PlaybackKey::CollapsedHint => "Відтворення (клацніть або потягніть, щоб розгорнути)",
        PlaybackKey::TimelineDetails => "Деталі шкали часу",
        PlaybackKey::TimelineDimension => "Вимір: {}",
        PlaybackKey::TimelineStep => "Крок: {} / {}",
        PlaybackKey::TimelineCurrent => "Поточне: {}",
        PlaybackKey::TimelineRange => "Діапазон: {} -> {}",
        PlaybackKey::TimelineStepSize => "Розмір кроку: {}",
        PlaybackKey::DefaultStepSize => "Крок: 1",
    })
}
