use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "データなし",
        HoverKey::MoreFields => "他 +{} 件",
        HoverKey::ChannelN => "チャンネル {}",
        HoverKey::SeriesLabel => "系列",
        HoverKey::FallbackVariable => "変数",
        HoverKey::FallbackOverlay => "オーバーレイ",
        HoverKey::DataValue => "データ値",
        HoverKey::LongitudeDegrees => "経度 (°)",
        HoverKey::LatitudeDegrees => "緯度 (°)",
        HoverKey::AlongAxisNamed => "{} 方向 ({})",
        HoverKey::AlongAxis => "{} 方向",
        HoverKey::AxisXLongitude => "X / 経度",
        HoverKey::AxisYLatitude => "Y / 緯度",
        HoverKey::AxisZDepth => "Z / 深さ",
        HoverKey::DropDataset => {
            "データセットをドロップして可視化 (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        HoverKey::LabelN => "ラベル {}",
    })
}
