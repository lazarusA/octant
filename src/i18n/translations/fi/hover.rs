use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Ei dataa",
        HoverKey::MoreFields => "+{} lisää",
        HoverKey::ChannelN => "Kanava {}",
        HoverKey::SeriesLabel => "sarja",
        HoverKey::FallbackVariable => "muuttuja",
        HoverKey::FallbackOverlay => "päällystaso",
        HoverKey::DataValue => "Data-arvo",
        HoverKey::LongitudeDegrees => "Pituusaste (astetta)",
        HoverKey::LatitudeDegrees => "Leveysaste (astetta)",
        HoverKey::AlongAxisNamed => "Akselia {} pitkin ({})",
        HoverKey::AlongAxis => "Akselia {} pitkin",
        HoverKey::AxisXLongitude => "X / pituusaste",
        HoverKey::AxisYLatitude => "Y / leveysaste",
        HoverKey::AxisZDepth => "Z / syvyys",
        HoverKey::DropDataset => {
            "Pudota aineisto visualisoitavaksi (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "tunniste {}",
    })
}
