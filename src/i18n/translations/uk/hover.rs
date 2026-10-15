use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Немає даних",
        HoverKey::MoreFields => "+{} ще",
        HoverKey::ChannelN => "Канал {}",
        HoverKey::SeriesLabel => "серія",
        HoverKey::FallbackVariable => "змінна",
        HoverKey::FallbackOverlay => "накладання",
        HoverKey::DataValue => "Значення",
        HoverKey::LongitudeDegrees => "Довгота (град.)",
        HoverKey::LatitudeDegrees => "Широта (град.)",
        HoverKey::AlongAxisNamed => "Уздовж {} ({})",
        HoverKey::AlongAxis => "Уздовж {}",
        HoverKey::AxisXLongitude => "X / довгота",
        HoverKey::AxisYLatitude => "Y / широта",
        HoverKey::AxisZDepth => "Z / глибина",
        HoverKey::DropDataset => {
            "Відпустіть набір даних для візуалізації (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "мітка {}",
        HoverKey::Level => "рівень",
        HoverKey::LevelOfTotal => "{} з {}",
    })
}
