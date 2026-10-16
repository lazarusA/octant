use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Keine Daten",
        HoverKey::MoreFields => "+{} weitere",
        HoverKey::ChannelN => "Kanal {}",
        HoverKey::SeriesLabel => "Serie",
        HoverKey::FallbackVariable => "Variable",
        HoverKey::FallbackOverlay => "Überlagerung",
        HoverKey::DataValue => "Datenwert",
        HoverKey::LongitudeDegrees => "Längengrad (°)",
        HoverKey::LatitudeDegrees => "Breitengrad (°)",
        HoverKey::AlongAxisNamed => "Entlang {} ({})",
        HoverKey::AlongAxis => "Entlang {}",
        HoverKey::AxisXLongitude => "X / Längengrad",
        HoverKey::AxisYLatitude => "Y / Breitengrad",
        HoverKey::AxisZDepth => "Z / Tiefe",
        HoverKey::DropDataset => {
            "Datensatz zum Visualisieren ablegen (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "Label {}",
        HoverKey::Level => "Stufe",
        HoverKey::LevelOfTotal => "{} von {}",
    })
}
