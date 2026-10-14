use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "히트맵",
        PlotKey::Line => "1D 선",
        PlotKey::Surface => "표면",
        PlotKey::Volume => "볼륨",
        PlotKey::Sphere => "구체",
        PlotKey::PointCloud => "포인트 클라우드",
        PlotKey::CompositeTrueColor => "트루 컬러",
        PlotKey::CompositeFalseColor => "폴스 컬러",
        PlotKey::CompositeRgb => "RGB 합성",
        PlotKey::CompositeOverlay => "채널 오버레이",
        PlotKey::AlignGeo => "다른 경도/위도 격자에 있음 (아직 지원하지 않음)",
        PlotKey::AlignIndexOnly => "격자 형태는 같지만 플롯과 일치하는 좌표가 없음",
        PlotKey::AlignIncompatible => "플롯과 공유하는 격자가 없음",
        PlotKey::OverlayPlotFirst => "먼저 변수를 그리세요",
        PlotKey::OverlayHeatmapOnly => "오버레이는 히트맵 위에 그려집니다",
        PlotKey::OverlayStackFull => "오버레이는 최대 4개입니다",
        PlotKey::OverlayAlreadyLayer => "이미 플롯의 레이어입니다",
        PlotKey::OverlayCantAdd => "오버레이를 추가할 수 없음",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "시각화 투영 선택",
        PlotKey::GridHealpixDiscrete => "HEALPix / 이산",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "{} 격자에서는 지원하지 않음",
        PlotKey::DisabledAggregation => "사용 불가: 2D 집계가 켜져 있음",
        PlotKey::Requires3D => "3D 데이터 필요",
        PlotKey::DisabledGpuLimit => "사용 불가: {} MB > 128 MB GPU 한도",
        PlotKey::ScalarField => "스칼라장",
        PlotKey::LabelsHeatmapOnly => "레이블 이미지는 히트맵에만 그려집니다",
    })
}
