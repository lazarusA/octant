use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "Группа",
        VariablesKey::DataType => "Тип",
        VariablesKey::Shape => "Форма",
        VariablesKey::Description => "Описание",
        VariablesKey::SearchVariables => "Поиск переменных...",
        VariablesKey::NoVariablesMatching => "Нет переменных, соответствующих '{}'",
        VariablesKey::CloseVariablesWindow => "Закрыть окно переменных",
        VariablesKey::NoMetadataLoaded => "Метаданные хранилища ещё не загружены.",
        VariablesKey::FetchStoreMetadata => "Загрузить метаданные хранилища",
        VariablesKey::NoVariablesFound => "В этом хранилище не найдено переменных.",
        VariablesKey::RefreshStoreMetadata => "Обновить метаданные хранилища",
        VariablesKey::TruncatedVariables => {
            "Показано {} из {} переменных в этой папке. Используйте поиск, чтобы найти все."
        }
        VariablesKey::TruncatedFolders => {
            "Показано {} из {} папок. Используйте поиск, чтобы найти все."
        }
        VariablesKey::NoVariableSelected => "Переменная не выбрана.",
        VariablesKey::DimensionSliders => "Ползунки измерений",
        VariablesKey::CloseDimensionPanel => "Закрыть панель измерений",
        VariablesKey::PlotData => "Построить",
        VariablesKey::PlotDataHover => "Загрузить эту переменную и построить график",
        VariablesKey::PlotDataOverlayHover => {
            "Загрузить эту переменную и добавить её как наложение поверх графика"
        }
        VariablesKey::AddOverlay => "Добавить наложение",
        VariablesKey::AddOverlayHover => {
            "Пока включено, 'Построить' добавляет эту переменную как наложение"
        }
        VariablesKey::InfoPath => "Путь:",
        VariablesKey::InfoUnits => "Единицы: {}",
        VariablesKey::InfoDescription => "Описание: {}",
        VariablesKey::InfoShape => "Форма: {}",
        VariablesKey::InfoDimensions => "Измерения: {}",
        VariablesKey::InfoTime => "Время: {} -> {}",
        VariablesKey::InfoResolution => "Разрешение: {}",
        VariablesKey::InfoSize => "Размер: {} MB",
        VariablesKey::InfoAttributes => "Атрибуты (.zattrs)",
        VariablesKey::Download => "Загрузка",
        VariablesKey::DownloadCells => "{} ({} ячеек)",
        VariablesKey::LargeSelectionPyramid => {
            "Большая 2D-выборка ({} ячеек, {} MB): включена автоматическая агрегация многомасштабной пирамидой."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "Сетки 3D-глобуса и 3D-поверхности отключены для этой большой выборки ({} ячеек, {} MB). 2D-плоскость и 1D-линия полностью доступны."
        }
        VariablesKey::LargeVolumeDisabled => {
            "3D-объём и облако точек отключены для этой выборки: размер объёма ({} MB) превышает лимит буфера хранения GPU в {} MB. 2D-плоскость, 1D-линия и 3D-глобус доступны."
        }
        VariablesKey::RoleNone => "Нет",
        VariablesKey::RoleAnimated => "Анимация",
        VariablesKey::RoleGrid => "Сетка",
        VariablesKey::Index => "Индекс:",
        VariablesKey::ChannelN => "Канал {}",
        VariablesKey::LevelN => "Уровень {}",
        VariablesKey::LevelAuto => "Авто",
        VariablesKey::LevelOf => "Уровень {} из {}",
        VariablesKey::AutoLevelOf => "Авто, уровень {} из {}",
    })
}
