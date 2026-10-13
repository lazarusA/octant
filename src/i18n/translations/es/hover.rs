use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "Sin datos",
        HoverKey::MoreFields => "+{} más",
        HoverKey::ChannelN => "Canal {}",
        HoverKey::SeriesLabel => "serie",
        HoverKey::FallbackVariable => "variable",
        HoverKey::FallbackOverlay => "superposición",
        HoverKey::DataValue => "Valor",
        HoverKey::LongitudeDegrees => "Longitud (grados)",
        HoverKey::LatitudeDegrees => "Latitud (grados)",
        HoverKey::AlongAxisNamed => "A lo largo de {} ({})",
        HoverKey::AlongAxis => "A lo largo de {}",
        HoverKey::AxisXLongitude => "X / Longitud",
        HoverKey::AxisYLatitude => "Y / Latitud",
        HoverKey::AxisZDepth => "Z / Profundidad",
        HoverKey::DropDataset => {
            "Suelte un conjunto de datos para visualizarlo (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "etiqueta {}",
    })
}
