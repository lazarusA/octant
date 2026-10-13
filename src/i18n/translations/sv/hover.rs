use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Inga data",
        HoverKey::MoreFields => "+{} till",
        HoverKey::ChannelN => "Kanal {}",
        HoverKey::SeriesLabel => "serie",
        HoverKey::FallbackVariable => "variabel",
        HoverKey::FallbackOverlay => "överlägg",
        HoverKey::DataValue => "Datavärde",
        HoverKey::LongitudeDegrees => "Longitud (grader)",
        HoverKey::LatitudeDegrees => "Latitud (grader)",
        HoverKey::AlongAxisNamed => "Längs {} ({})",
        HoverKey::AlongAxis => "Längs {}",
        HoverKey::AxisXLongitude => "X / longitud",
        HoverKey::AxisYLatitude => "Y / latitud",
        HoverKey::AxisZDepth => "Z / djup",
        HoverKey::DropDataset => {
            "Släpp ett dataset för att visualisera (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "etikett {}",
    })
}
