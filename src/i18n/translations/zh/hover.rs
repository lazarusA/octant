use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "无数据",
        HoverKey::MoreFields => "+{} 项",
        HoverKey::ChannelN => "通道 {}",
        HoverKey::SeriesLabel => "系列",
        HoverKey::FallbackVariable => "变量",
        HoverKey::FallbackOverlay => "叠加层",
        HoverKey::DataValue => "数据值",
        HoverKey::LongitudeDegrees => "经度 (°)",
        HoverKey::LatitudeDegrees => "纬度 (°)",
        HoverKey::AlongAxisNamed => "沿 {}（{}）",
        HoverKey::AlongAxis => "沿 {}",
        HoverKey::AxisXLongitude => "X / 经度",
        HoverKey::AxisYLatitude => "Y / 纬度",
        HoverKey::AxisZDepth => "Z / 深度",
        HoverKey::DropDataset => "拖放数据集以可视化 (.nc, .h5, .zarr, .icechunk, .tif)",
    })
}
