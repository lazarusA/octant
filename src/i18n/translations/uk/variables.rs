use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Група",
        VariablesKey::DataType => "Тип",
        VariablesKey::Shape => "Форма",
        VariablesKey::Description => "Опис",
        VariablesKey::SearchVariables => "Пошук змінних...",
        VariablesKey::NoVariablesMatching => "Немає змінних, що відповідають '{}'",
        VariablesKey::CloseVariablesWindow => "Закрити вікно змінних",
        VariablesKey::NoMetadataLoaded => "Метадані сховища ще не завантажено.",
        VariablesKey::FetchStoreMetadata => "Отримати / завантажити метадані сховища",
        VariablesKey::NoVariablesFound => "У цьому сховищі даних не знайдено змінних.",
        VariablesKey::RefreshStoreMetadata => "Оновити метадані сховища",
        VariablesKey::TruncatedVariables => {
            "Показано {} з {} змінних у цій папці. Скористайтеся пошуком, щоб знайти всі."
        }
        VariablesKey::TruncatedFolders => {
            "Показано {} з {} папок. Скористайтеся пошуком, щоб знайти всі."
        }
        VariablesKey::NoVariableSelected => "Змінну не вибрано.",
        VariablesKey::DimensionSliders => "Повзунки вимірів",
        VariablesKey::CloseDimensionPanel => "Закрити панель вимірів",
        VariablesKey::PlotData => "Побудувати",
        VariablesKey::PlotDataHover => "Отримати цю змінну та побудувати її графік",
        VariablesKey::PlotDataOverlayHover => {
            "Отримати цю змінну та додати її як накладання поверх графіка"
        }
        VariablesKey::AddOverlay => "Додати накладання",
        VariablesKey::AddOverlayHover => {
            "Коли увімкнено, 'Побудувати' додає цю змінну як накладання"
        }
        VariablesKey::InfoPath => "Шлях:",
        VariablesKey::InfoUnits => "Одиниці: {}",
        VariablesKey::InfoDescription => "Опис: {}",
        VariablesKey::InfoShape => "Форма: {}",
        VariablesKey::InfoDimensions => "Виміри: {}",
        VariablesKey::InfoTime => "Час: {} -> {}",
        VariablesKey::InfoResolution => "Роздільність: {}",
        VariablesKey::InfoSize => "Розмір: {} MB",
        VariablesKey::InfoAttributes => "Атрибути (.zattrs)",
        VariablesKey::Download => "Завантаження",
        VariablesKey::DownloadCells => "{} (комірок: {})",
        VariablesKey::LargeSelectionPyramid => {
            "Велика 2D-вибірка (комірок: {}, {} MB): увімкнено автоматичну агрегацію в піраміду кількох роздільностей."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "3D-глобус і 3D-поверхню вимкнено для цієї великої вибірки (комірок: {}, {} MB). 2D-площина та 1D-лінія залишаються доступними."
        }
        VariablesKey::LargeVolumeDisabled => {
            "3D-об'єм і хмару точок вимкнено для цієї вибірки: розмір об'єму ({} MB) перевищує ліміт буфера зберігання GPU 128 MB. 2D-площина, 1D-лінія та 3D-глобус залишаються доступними."
        }
        VariablesKey::RoleNone => "Немає",
        VariablesKey::RoleAnimated => "Анімація",
        VariablesKey::RoleGrid => "Сітка",
        VariablesKey::Index => "Індекс:",
        VariablesKey::ChannelN => "Канал {}",
    })
}
