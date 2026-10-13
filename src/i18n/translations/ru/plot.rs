use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Тепловая карта",
        PlotKey::Line => "1D-линия",
        PlotKey::Surface => "Поверхность",
        PlotKey::Volume => "Объём",
        PlotKey::Sphere => "Сфера",
        PlotKey::PointCloud => "Облако точек",
        PlotKey::CompositeTrueColor => "Естественные цвета",
        PlotKey::CompositeFalseColor => "Псевдоцвета",
        PlotKey::CompositeRgb => "RGB-композит",
        PlotKey::CompositeOverlay => "Наложение каналов",
        PlotKey::AlignGeo => "На другой сетке долготы/широты (пока не поддерживается)",
        PlotKey::AlignIndexOnly => {
            "Та же форма сетки, но ни одна координата не совпадает с графиком"
        }
        PlotKey::AlignIncompatible => "Нет общей сетки с графиком",
        PlotKey::OverlayPlotFirst => "Сначала постройте переменную",
        PlotKey::OverlayHeatmapOnly => "Наложения рисуются поверх тепловых карт",
        PlotKey::OverlayStackFull => "Не более 4 наложений",
        PlotKey::OverlayAlreadyLayer => "Уже является слоем графика",
        PlotKey::OverlayCantAdd => "Не удалось добавить наложение",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Выберите проекцию визуализации",
        PlotKey::GridHealpixDiscrete => "HEALPix / дискретная",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Не поддерживается для сетки {}",
        PlotKey::DisabledAggregation => "Отключено: включена 2D-агрегация",
        PlotKey::Requires3D => "Нужны 3D-данные",
        PlotKey::DisabledGpuLimit => "Отключено: {} MB > лимит GPU 128 MB",
        PlotKey::ScalarField => "Скалярное поле",
        PlotKey::LabelsHeatmapOnly => "Изображения меток рисуются на тепловой карте",
    })
}
