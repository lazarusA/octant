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
        PlaybackKey::CropHover => "Напрямні лінії обрізання",
        PlaybackKey::PlaybackSpeed => "Швидкість відтворення",
        PlaybackKey::PlaybackBuffer => "Буферизація перед відтворенням",
        PlaybackKey::Buffering => "Буферизація {}/{}",
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
        PlaybackKey::VoiceListen => "Почати слухати",
        PlaybackKey::VoiceStop => "Припинити слухати",
        PlaybackKey::VoiceListening => "Слухаю...",
        PlaybackKey::VoiceMicOff => "Мікрофон вимкнено",
        PlaybackKey::VoiceMicError => "Мікрофон недоступний",
        PlaybackKey::VoiceSilent => "Мікрофон передає лише тишу",
        PlaybackKey::VoiceSilentHint => {
            "Дозвольте доступ до мікрофона застосунку, що запустив Octant, у налаштуваннях конфіденційності системи та перезапустіть його."
        }
        PlaybackKey::VoiceSttFailed => "Не вдалося розпізнати мовлення",
        PlaybackKey::VoiceCloseHover => "Закрити голосове введення",
    })
}
