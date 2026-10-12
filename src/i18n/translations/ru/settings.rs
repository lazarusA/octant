use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Цвета берутся из каналов композита.",
        SettingsKey::ReasonCustomColor => "Свой цвет рисует все точки одним цветом.",
        SettingsKey::ReasonAllSeries => "Все серии линий окрашивает каждую линию по её индексу.",
        SettingsKey::ReasonIndexed => "Индексированный RGBA окрашивает воксели по индексу метки.",
        SettingsKey::ReasonIndexedMissing => {
            "Индексированный RGBA пропускает отсутствующие воксели."
        }
        SettingsKey::ReasonClassicOpaque => {
            "Только Объёмная трассировка (DVR) рисует полупрозрачные цвета."
        }
        SettingsKey::ReasonNotTranslucent => {
            "Действует, если Непрозрачность ниже 1 или задана Кривая альфа."
        }
        SettingsKey::ReasonCompositeOpaque => "Композиты рисуются непрозрачными.",
        SettingsKey::ReasonLabelOpaque => {
            "Поверхность категориальных меток всегда рисуется непрозрачной."
        }
        SettingsKey::ReasonAlwaysBlends => "Этот алгоритм всегда смешивает вдоль луча.",
        SettingsKey::CloseSettingsHover => "Закрыть настройки",
        SettingsKey::SectionOverlays => "Наложения",
        SettingsKey::SectionResolution => "Разрешение",
        SettingsKey::SectionView => "Вид",
        SettingsKey::Layers => "Слои",
        SettingsKey::BaseLayerHover => "Базовый слой: определяет холст, оси и вид",
        SettingsKey::BaseTag => "база",
        SettingsKey::Color => "Цвет",
        SettingsKey::HideLayerHover => "Скрыть слой",
        SettingsKey::ShowLayerHover => "Показать слой",
        SettingsKey::RemoveLayerHover => "Удалить слой",
        SettingsKey::MoveDownHover => "Переместить вниз",
        SettingsKey::MoveUpHover => "Переместить вверх",
        SettingsKey::NoVariable => "(нет переменной)",
        SettingsKey::ColorbarLabel => "Подпись легенды",
        SettingsKey::ResetLabelHover => "Сбросить подпись легенды к умолчанию",
        SettingsKey::ColorRange => "Диапазон цвета",
        SettingsKey::Categorical => "Категориальная",
        SettingsKey::CategoricalHover => {
            "Дискретная легенда (уникальные значения определяются автоматически, иначе 10 равных интервалов)."
        }
        SettingsKey::LockBoundsHover => {
            "Зафиксировать мин./макс., чтобы цвета не менялись между временными шагами."
        }
        SettingsKey::ResetBoundsHover => {
            "Сбросить границы к мин. и макс. текущего среза/набора данных"
        }
        SettingsKey::ResetAllColorbar => "Сбросить настройки легенды",
        SettingsKey::ResetAllColorbarHover => {
            "Сбросить подпись и диапазон легенды к значениям по умолчанию"
        }
        SettingsKey::RangeMin => "Мин.:",
        SettingsKey::RangeMax => "Макс.:",
        SettingsKey::NanColor => "Цвет NaN",
        SettingsKey::NanColorHover => "Если не отмечено, значения NaN/Inf прозрачны.",
        SettingsKey::NanColorPickerHover => "Цвет NaN. Нажмите, чтобы выбрать цвет.",
        SettingsKey::LowClip => "Нижнее отсечение",
        SettingsKey::LowClipHover => "Значения < cmin окрашиваются этим цветом.",
        SettingsKey::LowClipPickerHover => {
            "Цвет нижнего отсечения (< Мин.). Нажмите, чтобы выбрать цвет."
        }
        SettingsKey::HighClip => "Верхнее отсечение",
        SettingsKey::HighClipHover => "Значения > cmax окрашиваются этим цветом.",
        SettingsKey::HighClipPickerHover => {
            "Цвет верхнего отсечения (> Макс.). Нажмите, чтобы выбрать цвет."
        }
        SettingsKey::Scale => "Шкала",
        SettingsKey::ScaleLinear => "Линейная",
        SettingsKey::ScaleLog => "Логарифмическая",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Корень",
        SettingsKey::ScaleExponential => "Экспоненциальная",
        SettingsKey::ScaleLogHover => "Логарифмическая шкала (неотрицательные данные)",
        SettingsKey::ScaleLogDisabledHover => {
            "Отключено: требуется min >= 0. Для отрицательных данных используйте Symlog."
        }
        SettingsKey::ScaleParam => "Параметр:",
        SettingsKey::Opacity => "Непрозрачность",
        SettingsKey::OpacityHover => {
            "Непрозрачность значений цветовой шкалы. Цвета NaN и отсечения сохраняют свою альфу. \
             Двойной щелчок - сброс."
        }
        SettingsKey::AlphaCurve => "Кривая альфа",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  или  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Альфа вдоль легенды, от начала до конца. Значения распределяются \
             равномерно (Ступенчатая: по одному равному интервалу); точки position:alpha \
             размещаются вдоль шкалы. Пусто - без кривой."
        }
        SettingsKey::ClearCurveHover => "Очистить кривую",
        SettingsKey::Presets => "Шаблоны",
        SettingsKey::InterpStep => "Ступенчатая",
        SettingsKey::InterpLinear => "Линейная",
        SettingsKey::PresetRampUp => "Нарастание",
        SettingsKey::PresetRampDown => "Спад",
        SettingsKey::PresetPeak => "Пик",
        SettingsKey::PresetSink => "Провал",
        SettingsKey::PresetHideLow => "Скрыть низкие",
        SettingsKey::PresetHideCenter => "Скрыть центр",
        SettingsKey::Transparency => "Прозрачность",
        SettingsKey::TransparencyHover => {
            "При полупрозрачных цветах (Непрозрачность или Кривая альфа) рисовать каждый слой, \
             а не скрывать дальние за ближайшим."
        }
        SettingsKey::Lighting => "Освещение",
        SettingsKey::LightingHover => {
            "Затенять выборки по градиенту, чтобы фронты и края обретали форму"
        }
        SettingsKey::Coastlines => "Береговые линии",
        SettingsKey::CoastlinesHover => {
            "Накладывать береговые линии Natural Earth на географические графики."
        }
        SettingsKey::ResetCoastlineColor => "сброс",
        SettingsKey::ResetCoastlineColorHover => "Сбросить цвет береговых линий к цвету темы",
        SettingsKey::CoastlineColorHover => "Цвет береговых линий. Нажмите, чтобы выбрать.",
        SettingsKey::CoastlineResolution => "Разрешение:",
        SettingsKey::Lod110mHover => "110 m - быстро, всегда доступно (встроено)",
        SettingsKey::Lod50mHover => "50 m - средняя детализация (загружается в фоне)",
        SettingsKey::Lod10mHover => "10 m - высокая детализация (загружается в фоне)",
        SettingsKey::LineWidth => "Толщина линии",
        SettingsKey::CropToDomain => "Обрезать по области данных",
        SettingsKey::CropToDomainHover => {
            "Обрезать береговые линии строго по пространственной границе активного набора данных."
        }
        SettingsKey::AutoRotate => "Автовращение",
        SettingsKey::ResetView => "Сбросить вид",
        SettingsKey::ResetViewHover => "Сбросить ориентацию 3D-камеры",
        SettingsKey::HoverCard => "Карточка значений",
        SettingsKey::HideHoverCardHover => "Скрыть карточку значений",
        SettingsKey::ShowHoverCardHover => "Показать карточку значений",
        SettingsKey::Export => "Экспорт",
        SettingsKey::ExportFormat => "Формат:",
        SettingsKey::ExportFolder => "Папка:",
        SettingsKey::OpenSaveDialog => "Диалог сохранения",
        SettingsKey::CropTool => "Обрезка",
        SettingsKey::Aggregation => "2D-агрегация",
        SettingsKey::AggregationMandatoryHover => {
            "Обязательно: массив превышает лимит одного буфера GPU (128 MB / 33,5 млн ячеек). \
             Агрегацию нельзя отключить."
        }
        SettingsKey::AggregationHover => {
            "Прореживание многомасштабной пирамидой и передискретизация под область просмотра."
        }
        SettingsKey::AggregationMode => "Режим:",
        SettingsKey::AggMean => "Среднее (box-фильтр)",
        SettingsKey::AggMax => "Максимум (пики)",
        SettingsKey::AggMin => "Минимум (провалы)",
        SettingsKey::AggNearest => "Ближайший сосед",
        SettingsKey::AggMeanHover => "Усредняет подпиксели в ячейках 2x2, игнорируя NaN.",
        SettingsKey::AggMaxHover => "Сохраняет экстремально высокие значения.",
        SettingsKey::AggMinHover => "Сохраняет экстремально низкие значения.",
        SettingsKey::AggNearestHover => "Быстрый выбор ближайшей точки без интерполяции.",
        SettingsKey::CmykAutoMapped => "Каналы назначены автоматически: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "Наложение каналов",
        SettingsKey::MultiChannelOverlayHover => {
            "Складывает несколько каналов, каждый со своим цветовым оттенком."
        }
        SettingsKey::CmykComposite => "CMYK-композит",
        SettingsKey::CmykCompositeHover => {
            "Сводит 4 канала: голубой, пурпурный, жёлтый, чёрный (CMYK) в RGB в естественных цветах."
        }
        SettingsKey::RgbComposite => "RGB-композит",
        SettingsKey::RgbCompositeHover => "Сводит 3 выбранных канала в RGB в естественных цветах.",
        SettingsKey::Channels => "Каналы:",
        SettingsKey::ChannelsAll => "Все",
        SettingsKey::ChannelsNone => "Нет",
        SettingsKey::ChannelTintHover => "Нажмите, чтобы настроить оттенок канала",
        SettingsKey::BandNumber => "Канал {}",
    })
}
