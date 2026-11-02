use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Лінії",
        PlotOptionsKey::LinesHover => "Показувати суцільні лінії, що з'єднують точки",
        PlotOptionsKey::Scatter => "Точки",
        PlotOptionsKey::ScatterHover => "Показувати маркери в кожній точці",
        PlotOptionsKey::PointSize => "Розмір точки:",
        PlotOptionsKey::CustomColor => "Власний колір",
        PlotOptionsKey::CustomColorHover => {
            "Використовувати один колір ліній/точок замість колірної шкали"
        }
        PlotOptionsKey::LineColorHover => "Колір ліній / точок. Клацніть, щоб вибрати колір.",
        PlotOptionsKey::LineProfile => "Профіль лінії",
        PlotOptionsKey::AllLinesSeries => "Усі серії ліній",
        PlotOptionsKey::ProfileIndex => "Індекс профілю",
        PlotOptionsKey::SingleProfile => "Доступний лише один профіль.",
        PlotOptionsKey::AspectRatio => "Пропорції",
        PlotOptionsKey::AspectRatioHover => {
            "Якщо позначено, 2D-графіки зберігають пропорції матриці даних (ширина/висота). \
             Якщо ні, 2D-графіки розтягуються на все полотно."
        }
        PlotOptionsKey::Algorithm => "Алгоритм",
        PlotOptionsKey::AlgoDvr => "Об'ємне трасування (DVR)",
        PlotOptionsKey::AlgoMip => "Максимальна інтенсивність (MIP)",
        PlotOptionsKey::AlgoMinIp => "Мінімальна інтенсивність (MinIP)",
        PlotOptionsKey::AlgoXray => "Середня проєкція (рентген)",
        PlotOptionsKey::AlgoLabelSurface => "Поверхня категоріальних міток",
        PlotOptionsKey::AlgoAbsorption => "Поглинання RGBA",
        PlotOptionsKey::AlgoAdditive => "Адитивний RGBA",
        PlotOptionsKey::AlgoIndexed => "Індексований RGBA",
        PlotOptionsKey::Quality => "Якість",
        PlotOptionsKey::QualityHover => "Вибірок на воксель уздовж кожного променя",
        PlotOptionsKey::Density => "Щільність",
        PlotOptionsKey::Attenuation => "Згасання",
        PlotOptionsKey::ZScale => "Масштаб Z",
        PlotOptionsKey::ModeSmooth => "Гладкий",
        PlotOptionsKey::ModeBumpy => "Рельєфний",
        PlotOptionsKey::ModeSteps => "Сходинки",
        PlotOptionsKey::ModeVoxel => "Воксель",
        PlotOptionsKey::Height => "Висота",
        PlotOptionsKey::FlipHeights => "Перевернути висоти",
        PlotOptionsKey::FlipHeightsHover => {
            "Найбільше значення лежить пласко, а менші піднімаються над ним"
        }
        PlotOptionsKey::MeshDetail => "Деталізація",
        PlotOptionsKey::MeshDetailHover => {
            "Комірки, що малюються на великих сітках: чим більше деталей, тим повільніше; підказка завжди читає кожну комірку; режим Кожна комірка обмежений тим, що вміщує один буфер GPU"
        }
        PlotOptionsKey::DetailScreen => "Екран",
        PlotOptionsKey::DetailDouble => "2x екран",
        PlotOptionsKey::DetailQuadruple => "4x екран",
        PlotOptionsKey::DetailFull => "Кожна комірка",
        PlotOptionsKey::Size => "Розмір",
    })
}
