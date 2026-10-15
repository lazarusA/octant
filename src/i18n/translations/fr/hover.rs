use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Aucune donnée",
        HoverKey::MoreFields => "+{} de plus",
        HoverKey::ChannelN => "Canal {}",
        HoverKey::SeriesLabel => "série",
        HoverKey::FallbackVariable => "variable",
        HoverKey::FallbackOverlay => "superposition",
        HoverKey::DataValue => "Valeur",
        HoverKey::LongitudeDegrees => "Longitude (degrés)",
        HoverKey::LatitudeDegrees => "Latitude (degrés)",
        HoverKey::AlongAxisNamed => "Le long de {} ({})",
        HoverKey::AlongAxis => "Le long de {}",
        HoverKey::AxisXLongitude => "X / Longitude",
        HoverKey::AxisYLatitude => "Y / Latitude",
        HoverKey::AxisZDepth => "Z / Profondeur",
        HoverKey::DropDataset => {
            "Déposez un jeu de données à visualiser (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "étiquette {}",
        HoverKey::Level => "niveau",
        HoverKey::LevelOfTotal => "{} sur {}",
    })
}
