use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "선",
        PlotOptionsKey::LinesHover => "점을 잇는 연속된 선을 표시합니다",
        PlotOptionsKey::Scatter => "산점",
        PlotOptionsKey::ScatterHover => "각 점 위치에 산점 표식을 표시합니다",
        PlotOptionsKey::PointSize => "점 크기:",
        PlotOptionsKey::CustomColor => "사용자 색상",
        PlotOptionsKey::CustomColorHover => "컬러맵 대신 하나의 선/산점 색상을 사용합니다",
        PlotOptionsKey::LineColorHover => "선 / 산점 색상. 클릭하여 색상을 선택하세요.",
        PlotOptionsKey::LineProfile => "선 프로파일",
        PlotOptionsKey::AllLinesSeries => "모든 선 시리즈",
        PlotOptionsKey::ProfileIndex => "프로파일 인덱스",
        PlotOptionsKey::SingleProfile => "프로파일이 하나뿐입니다.",
        PlotOptionsKey::AspectRatio => "종횡비",
        PlotOptionsKey::AspectRatioHover => {
            "선택하면 2D 플롯이 행렬 데이터의 종횡비 (너비/높이)를 유지합니다. \
             선택하지 않으면 2D 플롯이 캔버스 전체를 채웁니다."
        }
        PlotOptionsKey::Algorithm => "알고리즘",
        PlotOptionsKey::AlgoDvr => "볼륨 레이마칭 (DVR)",
        PlotOptionsKey::AlgoMip => "최대 강도 (MIP)",
        PlotOptionsKey::AlgoMinIp => "최소 강도 (MinIP)",
        PlotOptionsKey::AlgoXray => "평균 투영 (X-ray)",
        PlotOptionsKey::AlgoLabelSurface => "범주형 레이블 표면",
        PlotOptionsKey::AlgoAbsorption => "흡수 RGBA",
        PlotOptionsKey::AlgoAdditive => "가산 RGBA",
        PlotOptionsKey::AlgoIndexed => "인덱스 RGBA",
        PlotOptionsKey::Quality => "품질",
        PlotOptionsKey::QualityHover => "광선마다 복셀당 샘플 수",
        PlotOptionsKey::Density => "밀도",
        PlotOptionsKey::Attenuation => "감쇠",
        PlotOptionsKey::ZScale => "Z 배율",
        PlotOptionsKey::ModeSmooth => "부드럽게",
        PlotOptionsKey::ModeBumpy => "울퉁불퉁",
        PlotOptionsKey::ModeSteps => "계단",
        PlotOptionsKey::ModeVoxel => "복셀",
        PlotOptionsKey::Height => "높이",
        PlotOptionsKey::FlipHeights => "높이 뒤집기",
        PlotOptionsKey::FlipHeightsHover => {
            "가장 높은 값이 바닥에 놓이고 낮은 값일수록 위로 솟습니다"
        }
        PlotOptionsKey::Size => "크기",
    })
}
