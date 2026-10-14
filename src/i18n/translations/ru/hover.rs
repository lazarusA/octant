use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Нет данных",
        HoverKey::MoreFields => "+{} ещё",
        HoverKey::ChannelN => "Канал {}",
        HoverKey::SeriesLabel => "серия",
        HoverKey::FallbackVariable => "переменная",
        HoverKey::FallbackOverlay => "наложение",
        HoverKey::DataValue => "Значение",
        HoverKey::LongitudeDegrees => "Долгота (град.)",
        HoverKey::LatitudeDegrees => "Широта (град.)",
        HoverKey::AlongAxisNamed => "Вдоль {} ({})",
        HoverKey::AlongAxis => "Вдоль {}",
        HoverKey::AxisXLongitude => "X / долгота",
        HoverKey::AxisYLatitude => "Y / широта",
        HoverKey::AxisZDepth => "Z / глубина",
        HoverKey::DropDataset => {
            "Отпустите набор данных для визуализации (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "метка {}",
    })
}
