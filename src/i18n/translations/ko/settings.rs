use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "색상은 합성 채널에서 가져옵니다.",
        SettingsKey::ReasonCustomColor => "사용자 색상은 모든 점을 한 색으로 그립니다.",
        SettingsKey::ReasonAllSeries => "모든 선 시리즈는 각 선을 인덱스에 따라 색칠합니다.",
        SettingsKey::ReasonIndexed => "인덱스 RGBA는 복셀을 레이블 인덱스에 따라 색칠합니다.",
        SettingsKey::ReasonIndexedMissing => "인덱스 RGBA는 누락된 복셀을 건너뜁니다.",
        SettingsKey::ReasonClassicOpaque => "볼륨 레이마칭 (DVR)만 반투명 색상을 그립니다.",
        SettingsKey::ReasonNotTranslucent => {
            "불투명도가 1보다 작거나 알파 곡선이 있을 때 적용됩니다."
        }
        SettingsKey::ReasonCompositeOpaque => "합성은 불투명하게 그려집니다.",
        SettingsKey::ReasonLabelOpaque => "범주형 레이블 표면은 항상 불투명하게 그려집니다.",
        SettingsKey::ReasonAlwaysBlends => "이 알고리즘은 항상 광선을 따라 혼합합니다.",
        SettingsKey::CloseSettingsHover => "설정 닫기",
        SettingsKey::SectionOverlays => "오버레이",
        SettingsKey::SectionResolution => "해상도",
        SettingsKey::SectionView => "보기",
        SettingsKey::Layers => "레이어",
        SettingsKey::BaseLayerHover => "기본 레이어: 캔버스, 축, 보기를 결정합니다",
        SettingsKey::BaseTag => "기본",
        SettingsKey::Color => "색상",
        SettingsKey::HideLayerHover => "레이어 숨기기",
        SettingsKey::ShowLayerHover => "레이어 표시",
        SettingsKey::RemoveLayerHover => "레이어 제거",
        SettingsKey::MoveDownHover => "아래로 이동",
        SettingsKey::MoveUpHover => "위로 이동",
        SettingsKey::NoVariable => "(변수 없음)",
        SettingsKey::ColorbarLabel => "컬러바 레이블",
        SettingsKey::ResetLabelHover => "컬러바 레이블을 기본값으로 초기화",
        SettingsKey::ColorRange => "색상 범위",
        SettingsKey::Categorical => "범주형",
        SettingsKey::CategoricalHover => {
            "이산 컬러바 (고유값 또는 같은 간격의 구간 10개를 자동으로 감지)."
        }
        SettingsKey::LockBoundsHover => {
            "최소/최대를 고정하여 시간 스텝이 바뀌어도 색상 매핑을 유지합니다."
        }
        SettingsKey::ResetBoundsHover => {
            "범위를 현재 슬라이스/데이터셋의 기본 최솟값과 최댓값으로 초기화"
        }
        SettingsKey::ResetAllColorbar => "컬러바 기본값 모두 초기화",
        SettingsKey::ResetAllColorbarHover => "컬러바 레이블과 범위를 모두 기본값으로 초기화",
        SettingsKey::RangeMin => "최소:",
        SettingsKey::RangeMax => "최대:",
        SettingsKey::NanColor => "NaN 색상",
        SettingsKey::NanColorHover => "선택하지 않으면 NaN/Inf 값은 투명하게 표시됩니다.",
        SettingsKey::NanColorPickerHover => "NaN 색상. 클릭하여 색상을 선택하세요.",
        SettingsKey::LowClip => "하한 클립",
        SettingsKey::LowClipHover => "cmin보다 작은 값은 이 색상으로 표시됩니다.",
        SettingsKey::LowClipPickerHover => "하한 클립 색상 (< 최소). 클릭하여 색상을 선택하세요.",
        SettingsKey::HighClip => "상한 클립",
        SettingsKey::HighClipHover => "cmax보다 큰 값은 이 색상으로 표시됩니다.",
        SettingsKey::HighClipPickerHover => "상한 클립 색상 (> 최대). 클릭하여 색상을 선택하세요.",
        SettingsKey::Scale => "척도",
        SettingsKey::ScaleLinear => "선형",
        SettingsKey::ScaleLog => "로그",
        SettingsKey::ScaleSymlog => "대칭 로그",
        SettingsKey::ScaleSqrt => "제곱근",
        SettingsKey::ScaleExponential => "지수",
        SettingsKey::ScaleLogHover => "로그 척도 (음이 아닌 데이터)",
        SettingsKey::ScaleLogDisabledHover => {
            "사용 불가: 최솟값이 0 이상이어야 합니다. 음수 데이터에는 대칭 로그를 사용하세요."
        }
        SettingsKey::ScaleParam => "매개변수:",
        SettingsKey::Opacity => "불투명도",
        SettingsKey::OpacityHover => {
            "컬러맵으로 표시한 값의 불투명도입니다. NaN 및 클립 색상은 자체 알파를 유지합니다. \
             더블클릭하면 초기화됩니다."
        }
        SettingsKey::AlphaCurve => "알파 곡선",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  또는  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "컬러바의 시작부터 끝까지의 알파입니다. 값은 같은 간격으로 놓이고 \
             (계단: 값마다 같은 구간 하나), position:alpha 지점은 \
             바를 따라 배치됩니다. 비워 두면 사용하지 않습니다."
        }
        SettingsKey::ClearCurveHover => "곡선 지우기",
        SettingsKey::Presets => "프리셋",
        SettingsKey::InterpStep => "계단",
        SettingsKey::InterpLinear => "선형",
        SettingsKey::PresetRampUp => "증가",
        SettingsKey::PresetRampDown => "감소",
        SettingsKey::PresetPeak => "봉우리",
        SettingsKey::PresetSink => "골",
        SettingsKey::PresetHideLow => "낮은 값 숨기기",
        SettingsKey::PresetHideCenter => "가운데 숨기기",
        SettingsKey::Transparency => "투명도",
        SettingsKey::TransparencyHover => {
            "반투명 색상 (불투명도 또는 알파 곡선)을 쓸 때, 가장 가까운 면이 \
             뒤의 면을 가리지 않고 모든 면을 그립니다."
        }
        SettingsKey::Lighting => "조명",
        SettingsKey::LightingHover => {
            "샘플을 기울기에 따라 음영 처리하여 앞면과 가장자리에 입체감을 줍니다"
        }
        SettingsKey::Coastlines => "해안선",
        SettingsKey::CoastlinesHover => "지리 플롯 위에 Natural Earth 해안선을 겹쳐 표시합니다.",
        SettingsKey::ResetCoastlineColor => "초기화",
        SettingsKey::ResetCoastlineColorHover => "해안선 색상을 테마 기본값으로 초기화",
        SettingsKey::CoastlineColorHover => "해안선 색상. 클릭하여 선택하세요.",
        SettingsKey::CoastlineResolution => "해상도:",
        SettingsKey::Lod110mHover => "110 m - 빠름, 항상 사용 가능 (내장)",
        SettingsKey::Lod50mHover => "50 m - 중간 상세도 (백그라운드에서 불러옴)",
        SettingsKey::Lod10mHover => "10 m - 높은 상세도 (백그라운드에서 불러옴)",
        SettingsKey::LineWidth => "선 두께",
        SettingsKey::CropToDomain => "데이터 영역으로 자르기",
        SettingsKey::CropToDomainHover => "해안선을 활성 데이터셋의 공간 경계에 맞춰 자릅니다.",
        SettingsKey::AutoRotate => "자동 회전",
        SettingsKey::ResetView => "보기 초기화",
        SettingsKey::ResetViewHover => "3D 카메라 방향 초기화",
        SettingsKey::HoverCard => "호버 카드",
        SettingsKey::HideHoverCardHover => "호버 카드 숨기기",
        SettingsKey::ShowHoverCardHover => "호버 카드 표시",
        SettingsKey::Export => "내보내기",
        SettingsKey::ExportFormat => "형식:",
        SettingsKey::ExportFolder => "폴더:",
        SettingsKey::OpenSaveDialog => "저장 대화상자 열기",
        SettingsKey::CropTool => "자르기 도구",
        SettingsKey::Aggregation => "2D 집계",
        SettingsKey::AggregationMandatoryHover => {
            "필수: 배열이 단일 GPU 버퍼 한도 ({} MB)를 넘습니다. \
             집계를 끌 수 없습니다."
        }
        SettingsKey::AggregationHover => "다중 해상도 피라미드 다운샘플링 및 뷰포트 리샘플링.",
        SettingsKey::AggregationMode => "모드:",
        SettingsKey::AggMean => "평균 (박스 필터)",
        SettingsKey::AggMax => "최대 (피크 보존)",
        SettingsKey::AggMin => "최소 (최저값 보존)",
        SettingsKey::AggNearest => "최근접 이웃",
        SettingsKey::AggMeanHover => "2x2 셀의 하위 픽셀을 평균하며 NaN은 무시합니다.",
        SettingsKey::AggMaxHover => "극단적으로 높은 값을 보존합니다.",
        SettingsKey::AggMinHover => "극단적으로 낮은 값을 보존합니다.",
        SettingsKey::AggNearestHover => "보간 없이 가장 가까운 점을 빠르게 선택합니다.",
        SettingsKey::ResolutionLevel => "해상도 단계",
        SettingsKey::ResolutionLevelHover => {
            "플롯이 읽는 해상도 단계입니다. 자동은 화면에 맞는 가장 세밀한 단계를 읽습니다."
        }
        SettingsKey::CmykAutoMapped => "자동 매핑된 채널: C (1), M (2), Y (3), K (4)",
        SettingsKey::MultiChannelOverlay => "다중 채널 오버레이",
        SettingsKey::MultiChannelOverlayHover => {
            "여러 채널을 가산 방식으로 겹치며, 각 채널은 고유한 색조로 그려집니다."
        }
        SettingsKey::CmykComposite => "CMYK 합성",
        SettingsKey::CmykCompositeHover => {
            "4채널 시안, 마젠타, 노랑, 검정 (CMYK)을 트루 컬러 RGB로 합성합니다."
        }
        SettingsKey::RgbComposite => "RGB 합성",
        SettingsKey::RgbCompositeHover => "선택한 3개 채널을 트루 컬러 RGB로 합성합니다.",
        SettingsKey::Channels => "채널:",
        SettingsKey::ChannelsAll => "전체",
        SettingsKey::ChannelsNone => "없음",
        SettingsKey::ChannelTintHover => "클릭하여 채널 색조를 지정하세요",
        SettingsKey::BandNumber => "밴드 {}",
        SettingsKey::ScaleBar => "스케일 바",
        SettingsKey::ShowScaleBarHover => "스케일 바 표시",
        SettingsKey::HideScaleBarHover => "스케일 바 숨기기",
        SettingsKey::ChannelsAuto => "자동",
        SettingsKey::ChannelsAutoHover => {
            "각 채널의 표시 범위를 표시된 데이터에 맞춤(값의 0.1%에서 99.9%)"
        }
        SettingsKey::ChannelsAutoUnavailable => "자동 대비는 2D 단면에서 작동합니다",
        SettingsKey::ChannelSettingsHover => "표시 범위, 곡선 및 반전",
        SettingsKey::ChannelWindow => "표시 범위",
        SettingsKey::ChannelWindowHover => {
            "시작부터 끝까지의 값이 검정에서 채널 고유의 색까지 대응합니다"
        }
        SettingsKey::ChannelInvert => "반전",
        SettingsKey::ChannelCurve => "곡선",
        SettingsKey::CurveLinear => "선형",
        SettingsKey::CurveLogarithmic => "로그",
        SettingsKey::CurveExponential => "지수",
        SettingsKey::CurvePolynomial => "다항식",
        SettingsKey::ChannelCoefficientHover => "곡선 계수",
        SettingsKey::LabelsFill => "채우기",
        SettingsKey::LabelsFillHover => "각 레이블 전체에 색 칠하기",
        SettingsKey::LabelsOutline => "윤곽선",
        SettingsKey::LabelsOutlineHover => "레이블 사이 경계만 그리기",
    })
}
