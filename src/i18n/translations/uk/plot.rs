use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Теплова карта",
        PlotKey::Line => "1D-лінія",
        PlotKey::Surface => "Поверхня",
        PlotKey::Volume => "Об'єм",
        PlotKey::Sphere => "Сфера",
        PlotKey::PointCloud => "Хмара точок",
        PlotKey::CompositeTrueColor => "Справжні кольори",
        PlotKey::CompositeFalseColor => "Псевдокольори",
        PlotKey::CompositeRgb => "RGB-композит",
        PlotKey::CompositeOverlay => "Накладання каналів",
        PlotKey::AlignGeo => "На іншій сітці довготи/широти (ще не підтримується)",
        PlotKey::AlignIndexOnly => {
            "Та сама форма сітки, але жодна координата не збігається з графіком"
        }
        PlotKey::AlignIncompatible => "Не має спільної сітки з графіком",
        PlotKey::OverlayPlotFirst => "Спочатку побудуйте змінну",
        PlotKey::OverlayHeatmapOnly => "Накладання малюються поверх теплових карт",
        PlotKey::OverlayStackFull => "Не більше 4 накладань",
        PlotKey::OverlayAlreadyLayer => "Уже є шаром графіка",
        PlotKey::OverlayCantAdd => "Не вдалося додати накладання",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Виберіть проєкцію візуалізації",
        PlotKey::GridHealpixDiscrete => "HEALPix / дискретна",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Не підтримується для сітки {}",
        PlotKey::DisabledAggregation => "Вимкнено: увімкнено 2D-агрегацію",
        PlotKey::Requires3D => "Потрібні 3D-дані",
        PlotKey::DisabledGpuLimit => "Вимкнено: {} MB > ліміт GPU 128 MB",
        PlotKey::ScalarField => "Скалярне поле",
        PlotKey::LabelsHeatmapOnly => "Зображення міток малюються на тепловій карті",
    })
}
