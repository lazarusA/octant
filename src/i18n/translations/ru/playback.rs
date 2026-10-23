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
        PlaybackKey::CropHover => "Направляющие обрезки",
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
        PlaybackKey::VoiceListen => "Начать слушать",
        PlaybackKey::VoiceStop => "Перестать слушать",
        PlaybackKey::VoiceListening => "Слушаю...",
        PlaybackKey::VoiceMicOff => "Микрофон выключен",
        PlaybackKey::VoiceMicError => "Микрофон недоступен",
        PlaybackKey::VoiceSilent => "Микрофон передаёт только тишину",
        PlaybackKey::VoiceSilentHint => {
            "Разрешите доступ к микрофону приложению, запустившему Octant, в настройках конфиденциальности системы и перезапустите его."
        }
        PlaybackKey::VoiceSttFailed => "Не удалось распознать речь",
        PlaybackKey::VoiceCloseHover => "Закрыть голосовой ввод",
    })
}
