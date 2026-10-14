use crate::i18n::keys::HoverKey;

pub fn text(key: HoverKey) -> Option<&'static str> {
    Some(match key {
        HoverKey::NoData => "데이터 없음",
        HoverKey::MoreFields => "+{}개 더",
        HoverKey::ChannelN => "채널 {}",
        HoverKey::SeriesLabel => "시리즈",
        HoverKey::FallbackVariable => "변수",
        HoverKey::FallbackOverlay => "오버레이",
        HoverKey::DataValue => "데이터 값",
        HoverKey::LongitudeDegrees => "경도 (°)",
        HoverKey::LatitudeDegrees => "위도 (°)",
        HoverKey::AlongAxisNamed => "{} 방향 ({})",
        HoverKey::AlongAxis => "{} 방향",
        HoverKey::AxisXLongitude => "X / 경도",
        HoverKey::AxisYLatitude => "Y / 위도",
        HoverKey::AxisZDepth => "Z / 깊이",
        HoverKey::DropDataset => "데이터셋을 놓아 시각화 (.nc, .h5, .zarr, .icechunk, .tif)",
        HoverKey::LabelN => "레이블 {}",
    })
}
