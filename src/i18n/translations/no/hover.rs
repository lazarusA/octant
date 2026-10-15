use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Ingen data",
        HoverKey::MoreFields => "+{} til",
        HoverKey::ChannelN => "Kanal {}",
        HoverKey::SeriesLabel => "serie",
        HoverKey::FallbackVariable => "variabel",
        HoverKey::FallbackOverlay => "overlegg",
        HoverKey::DataValue => "Dataverdi",
        HoverKey::LongitudeDegrees => "Lengdegrad (grader)",
        HoverKey::LatitudeDegrees => "Breddegrad (grader)",
        HoverKey::AlongAxisNamed => "Langs {} ({})",
        HoverKey::AlongAxis => "Langs {}",
        HoverKey::AxisXLongitude => "X / lengdegrad",
        HoverKey::AxisYLatitude => "Y / breddegrad",
        HoverKey::AxisZDepth => "Z / dybde",
        HoverKey::DropDataset => {
            "Slipp et datasett for å visualisere det (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "etikett {}",
        HoverKey::Level => "nivå",
        HoverKey::LevelOfTotal => "{} av {}",
    })
}
