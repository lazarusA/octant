use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> &'static str {
    match key {
        HoverKey::NoData => "No data",
        HoverKey::MoreFields => "+{} more",
        HoverKey::ChannelN => "Channel {}",
        HoverKey::SeriesLabel => "series",
        HoverKey::FallbackVariable => "variable",
        HoverKey::FallbackOverlay => "overlay",
        HoverKey::DataValue => "Data Value",
        HoverKey::LongitudeDegrees => "Longitude (°)",
        HoverKey::LatitudeDegrees => "Latitude (°)",
        HoverKey::AlongAxisNamed => "Along {} ({})",
        HoverKey::AlongAxis => "Along {}",
        HoverKey::AxisXLongitude => "X / Longitude",
        HoverKey::AxisYLatitude => "Y / Latitude",
        HoverKey::AxisZDepth => "Z / Depth",
        HoverKey::DropDataset => "Drop dataset to visualize (.nc, .h5, .zarr, .icechunk, .tif)",
        HoverKey::LabelN => "label {}",
    }
}
