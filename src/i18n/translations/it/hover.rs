use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Nessun dato",
        HoverKey::MoreFields => "+{} altri",
        HoverKey::ChannelN => "Canale {}",
        HoverKey::SeriesLabel => "serie",
        HoverKey::FallbackVariable => "variabile",
        HoverKey::FallbackOverlay => "sovrapposizione",
        HoverKey::DataValue => "Valore del dato",
        HoverKey::LongitudeDegrees => "Longitudine (gradi)",
        HoverKey::LatitudeDegrees => "Latitudine (gradi)",
        HoverKey::AlongAxisNamed => "Lungo {} ({})",
        HoverKey::AlongAxis => "Lungo {}",
        HoverKey::AxisXLongitude => "X / Longitudine",
        HoverKey::AxisYLatitude => "Y / Latitudine",
        HoverKey::AxisZDepth => "Z / Profondità",
        HoverKey::DropDataset => {
            "Rilascia un set di dati da visualizzare (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "etichetta {}",
        HoverKey::Level => "livello",
        HoverKey::LevelOfTotal => "{} di {}",
    })
}
