use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "Кольори беруться з каналів композиту.",
        SettingsKey::ReasonCustomColor => "Власний колір малює всі точки одним кольором.",
        SettingsKey::ReasonAllSeries => "Режим Усі серії ліній фарбує кожну лінію за її індексом.",
        SettingsKey::ReasonIndexed => "Індексований RGBA фарбує воксели за індексом мітки.",
        SettingsKey::ReasonIndexedMissing => "Індексований RGBA пропускає відсутні воксели.",
        SettingsKey::ReasonClassicOpaque => {
            "Лише Об'ємне трасування (DVR) малює напівпрозорі кольори."
        }
        SettingsKey::ReasonNotTranslucent => {
            "Діє, коли Непрозорість менша за 1 або задано Криву альфа."
        }
        SettingsKey::ReasonCompositeOpaque => "Композити малюються непрозорими.",
        SettingsKey::ReasonLabelOpaque => {
            "Поверхня категоріальних міток завжди малюється непрозорою."
        }
        SettingsKey::ReasonAlwaysBlends => "Цей алгоритм завжди змішує вздовж променя.",
        SettingsKey::CloseSettingsHover => "Закрити налаштування",
        SettingsKey::SectionOverlays => "Накладання",
        SettingsKey::SectionResolution => "Роздільність",
        SettingsKey::SectionView => "Вигляд",
        SettingsKey::Layers => "Шари",
        SettingsKey::BaseLayerHover => "Базовий шар: визначає полотно, осі та вигляд",
        SettingsKey::BaseTag => "база",
        SettingsKey::Color => "Колір",
        SettingsKey::HideLayerHover => "Сховати шар",
        SettingsKey::ShowLayerHover => "Показати шар",
        SettingsKey::RemoveLayerHover => "Видалити шар",
        SettingsKey::MoveDownHover => "Перемістити нижче",
        SettingsKey::MoveUpHover => "Перемістити вище",
        SettingsKey::NoVariable => "(немає змінної)",
        SettingsKey::ColorbarLabel => "Підпис кольорової смуги",
        SettingsKey::ResetLabelHover => "Скинути підпис кольорової смуги до типового",
        SettingsKey::ColorRange => "Діапазон кольорів",
        SettingsKey::Categorical => "Категоріальна",
        SettingsKey::CategoricalHover => {
            "Дискретна кольорова смуга (автовизначення унікальних значень або 10 рівних інтервалів)."
        }
        SettingsKey::LockBoundsHover => {
            "Зафіксувати мін./макс., щоб відображення кольорів не змінювалося між кроками часу."
        }
        SettingsKey::ResetBoundsHover => {
            "Скинути межі до типових мін. і макс. поточного зрізу/набору даних"
        }
        SettingsKey::ResetAllColorbar => "Скинути кольорову смугу",
        SettingsKey::ResetAllColorbarHover => {
            "Скинути підпис і діапазон кольорової смуги до типових значень"
        }
        SettingsKey::RangeMin => "Мін.:",
        SettingsKey::RangeMax => "Макс.:",
        SettingsKey::NanColor => "Колір NaN",
        SettingsKey::NanColorHover => "Якщо не позначено, значення NaN/Inf прозорі.",
        SettingsKey::NanColorPickerHover => "Колір NaN. Клацніть, щоб вибрати колір.",
        SettingsKey::LowClip => "Нижнє відсікання",
        SettingsKey::LowClipHover => "Значення < cmin відсікаються до цього кольору.",
        SettingsKey::LowClipPickerHover => {
            "Колір нижнього відсікання (< Мін.). Клацніть, щоб вибрати колір."
        }
        SettingsKey::HighClip => "Верхнє відсікання",
        SettingsKey::HighClipHover => "Значення > cmax відсікаються до цього кольору.",
        SettingsKey::HighClipPickerHover => {
            "Колір верхнього відсікання (> Макс.). Клацніть, щоб вибрати колір."
        }
        SettingsKey::Scale => "Шкала",
        SettingsKey::ScaleLinear => "Лінійна",
        SettingsKey::ScaleLog => "Логарифмічна",
        SettingsKey::ScaleSymlog => "Symlog",
        SettingsKey::ScaleSqrt => "Корінь",
        SettingsKey::ScaleExponential => "Експоненційна",
        SettingsKey::ScaleLogHover => "Логарифмічна шкала (невід'ємні дані)",
        SettingsKey::ScaleLogDisabledHover => {
            "Вимкнено: потрібно min >= 0. Для від'ємних даних використовуйте Symlog."
        }
        SettingsKey::ScaleParam => "Параметр:",
        SettingsKey::Opacity => "Непрозорість",
        SettingsKey::OpacityHover => {
            "Непрозорість значень за колірною шкалою. Кольори NaN і відсікання зберігають власну альфу. \
             Двічі клацніть, щоб скинути."
        }
        SettingsKey::AlphaCurve => "Крива альфа",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  або  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "Альфа вздовж кольорової смуги, від її початку до кінця. Значення розподілені \
             рівномірно (Крок: по одному рівному інтервалу); точки position:alpha \
             розміщуються вздовж смуги. Порожньо - без кривої."
        }
        SettingsKey::ClearCurveHover => "Очистити криву",
        SettingsKey::Presets => "Шаблони",
        SettingsKey::InterpStep => "Крок",
        SettingsKey::InterpLinear => "Лінійна",
        SettingsKey::PresetRampUp => "Наростання",
        SettingsKey::PresetRampDown => "Спадання",
        SettingsKey::PresetPeak => "Пік",
        SettingsKey::PresetSink => "Западина",
        SettingsKey::PresetHideLow => "Сховати низькі",
        SettingsKey::PresetHideCenter => "Сховати центр",
        SettingsKey::Transparency => "Прозорість",
        SettingsKey::TransparencyHover => {
            "З напівпрозорими кольорами (Непрозорість або Крива альфа) малювати кожен шар, \
             а не дозволяти найближчому приховувати ті, що за ним."
        }
        SettingsKey::Lighting => "Освітлення",
        SettingsKey::LightingHover => {
            "Затінювати вибірки за градієнтом, щоб фронти й краї набували форми"
        }
        SettingsKey::Coastlines => "Берегові лінії",
        SettingsKey::CoastlinesHover => {
            "Накладати берегові лінії Natural Earth на географічні графіки."
        }
        SettingsKey::ResetCoastlineColor => "скинути",
        SettingsKey::ResetCoastlineColorHover => {
            "Скинути колір берегових ліній до типового для теми"
        }
        SettingsKey::CoastlineColorHover => "Колір берегових ліній. Клацніть, щоб вибрати.",
        SettingsKey::CoastlineResolution => "Роздільність:",
        SettingsKey::Lod110mHover => "110 m - швидко, завжди доступно (вбудовано)",
        SettingsKey::Lod50mHover => "50 m - середня деталізація (завантажується у фоні)",
        SettingsKey::Lod10mHover => "10 m - висока деталізація (завантажується у фоні)",
        SettingsKey::LineWidth => "Товщина лінії",
        SettingsKey::CropToDomain => "Обрізати до області даних",
        SettingsKey::CropToDomainHover => {
            "Обрізати берегові лінії строго за просторовою межею активного набору даних."
        }
        SettingsKey::AutoRotate => "Автообертання",
        SettingsKey::ResetView => "Скинути вигляд",
        SettingsKey::ResetViewHover => "Скинути орієнтацію 3D-камери",
        SettingsKey::HoverCard => "Картка наведення",
        SettingsKey::HideHoverCardHover => "Сховати картку наведення",
        SettingsKey::ShowHoverCardHover => "Показати картку наведення",
        SettingsKey::Export => "Експорт",
        SettingsKey::ExportFormat => "Формат:",
        SettingsKey::ExportFolder => "Папка:",
        SettingsKey::OpenSaveDialog => "Відкрити діалог збереження",
        SettingsKey::CropTool => "Обрізання",
        SettingsKey::Aggregation => "2D-агрегація",
        SettingsKey::AggregationMandatoryHover => {
            "Обов'язково: масив перевищує ліміт одного буфера GPU (128 MB / 33.5M комірок). \
             Агрегацію не можна вимкнути."
        }
        SettingsKey::AggregationHover => {
            "Зменшення роздільності пірамідою та перевибірка під область перегляду."
        }
        SettingsKey::AggregationMode => "Режим:",
        SettingsKey::AggMean => "Середнє (блоковий фільтр)",
        SettingsKey::AggMax => "Макс. (зберігає піки)",
        SettingsKey::AggMin => "Мін. (зберігає западини)",
        SettingsKey::AggNearest => "Найближчий сусід",
        SettingsKey::AggMeanHover => "Усереднює підпікселі в комірках 2x2, ігноруючи NaN.",
        SettingsKey::AggMaxHover => "Зберігає екстремально високі значення.",
        SettingsKey::AggMinHover => "Зберігає екстремально низькі значення.",
        SettingsKey::AggNearestHover => "Швидкий вибір найближчої точки без інтерполяції.",
        SettingsKey::CmykAutoMapped => "Автопризначені канали: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "Багатоканальне накладання",
        SettingsKey::MultiChannelOverlayHover => {
            "Накладає кілька каналів адитивно, кожен зі своїм унікальним відтінком."
        }
        SettingsKey::CmykComposite => "CMYK-композит",
        SettingsKey::CmykCompositeHover => {
            "Поєднує 4 канали: блакитний, пурпуровий, жовтий і чорний (CMYK) у RGB у справжніх кольорах."
        }
        SettingsKey::RgbComposite => "RGB-композит",
        SettingsKey::RgbCompositeHover => "Поєднує 3 вибрані канали в RGB у справжніх кольорах.",
        SettingsKey::Channels => "Канали:",
        SettingsKey::ChannelsAll => "Усі",
        SettingsKey::ChannelsNone => "Жодного",
        SettingsKey::ChannelTintHover => "Клацніть, щоб налаштувати відтінок каналу",
        SettingsKey::BandNumber => "Канал {}",
    })
}
