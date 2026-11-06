use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Линии",
        PlotOptionsKey::LinesHover => "Показывать непрерывные линии, соединяющие точки",
        PlotOptionsKey::Scatter => "Точки",
        PlotOptionsKey::ScatterHover => "Показывать маркеры в каждой точке",
        PlotOptionsKey::PointSize => "Размер точки:",
        PlotOptionsKey::CustomColor => "Свой цвет",
        PlotOptionsKey::CustomColorHover => {
            "Использовать один цвет линий/точек вместо цветовой шкалы"
        }
        PlotOptionsKey::LineColorHover => "Цвет линий / точек. Нажмите, чтобы выбрать цвет.",
        PlotOptionsKey::LineProfile => "Профиль линии",
        PlotOptionsKey::AllLinesSeries => "Все серии линий",
        PlotOptionsKey::ProfileIndex => "Индекс профиля",
        PlotOptionsKey::SingleProfile => "Доступен один профиль.",
        PlotOptionsKey::AspectRatio => "Пропорции",
        PlotOptionsKey::AspectRatioHover => {
            "Если отмечено, 2D-графики сохраняют пропорции матрицы данных (ширина/высота). \
             Если не отмечено, 2D-графики занимают весь холст."
        }
        PlotOptionsKey::Algorithm => "Алгоритм",
        PlotOptionsKey::AlgoDvr => "Объёмная трассировка (DVR)",
        PlotOptionsKey::AlgoMip => "Максимальная интенсивность (MIP)",
        PlotOptionsKey::AlgoMinIp => "Минимальная интенсивность (MinIP)",
        PlotOptionsKey::AlgoXray => "Средняя проекция (рентген)",
        PlotOptionsKey::AlgoLabelSurface => "Поверхность категориальных меток",
        PlotOptionsKey::AlgoAbsorption => "Поглощение RGBA",
        PlotOptionsKey::AlgoAdditive => "Аддитивный RGBA",
        PlotOptionsKey::AlgoIndexed => "Индексированный RGBA",
        PlotOptionsKey::Quality => "Качество",
        PlotOptionsKey::QualityHover => "Число выборок на воксель вдоль каждого луча",
        PlotOptionsKey::Density => "Плотность",
        PlotOptionsKey::Attenuation => "Затухание",
        PlotOptionsKey::ZScale => "Масштаб Z",
        PlotOptionsKey::ModeSmooth => "Гладкий",
        PlotOptionsKey::ModeBumpy => "Рельефный",
        PlotOptionsKey::ModeSteps => "Ступени",
        PlotOptionsKey::ModeVoxel => "Воксели",
        PlotOptionsKey::Height => "Высота",
        PlotOptionsKey::FlipHeights => "Перевернуть высоты",
        PlotOptionsKey::ScaleByValue => "Размер по значению",
        PlotOptionsKey::FlipHeightsHover => {
            "Наибольшее значение лежит внизу, а меньшие поднимаются над ним"
        }
        PlotOptionsKey::MeshDetail => "Детализация",
        PlotOptionsKey::MeshDetailHover => {
            "Ячейки, которые рисуются на больших сетках: чем больше деталей, тем медленнее; подсказка всегда читает каждую ячейку; режим Каждая ячейка ограничен тем, что вмещает один буфер GPU"
        }
        PlotOptionsKey::DetailScreen => "Экран",
        PlotOptionsKey::DetailDouble => "2x экран",
        PlotOptionsKey::DetailQuadruple => "4x экран",
        PlotOptionsKey::DetailFull => "Каждая ячейка",
        PlotOptionsKey::Size => "Размер",
        PlotOptionsKey::Surface => "Поверхность",
        PlotOptionsKey::SurfaceRange => "Цветовой диапазон",
        PlotOptionsKey::SurfaceValue => "Значение",
        PlotOptionsKey::SurfaceHover => {
            "Где непрозрачный объём рисует поверхность: где значения входят в цветовой диапазон или где они достигают одного значения"
        }
    })
}
