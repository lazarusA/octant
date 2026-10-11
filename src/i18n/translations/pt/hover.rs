use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Sem dados",
        HoverKey::MoreFields => "+{} mais",
        HoverKey::ChannelN => "Canal {}",
        HoverKey::SeriesLabel => "série",
        HoverKey::FallbackVariable => "variável",
        HoverKey::FallbackOverlay => "sobreposição",
        HoverKey::DataValue => "Valor do dado",
        HoverKey::LongitudeDegrees => "Longitude (graus)",
        HoverKey::LatitudeDegrees => "Latitude (graus)",
        HoverKey::AlongAxisNamed => "Ao longo de {} ({})",
        HoverKey::AlongAxis => "Ao longo de {}",
        HoverKey::AxisXLongitude => "X / Longitude",
        HoverKey::AxisYLatitude => "Y / Latitude",
        HoverKey::AxisZDepth => "Z / Profundidade",
        HoverKey::DropDataset => {
            "Solte um conjunto de dados para visualizar (.nc, .h5, .zarr, .icechunk, .tif)"
        }
    })
}
