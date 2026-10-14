use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "그룹",
        VariablesKey::DataType => "자료형",
        VariablesKey::Shape => "형태",
        VariablesKey::Description => "설명",
        VariablesKey::SearchVariables => "변수 검색...",
        VariablesKey::NoVariablesMatching => "일치하는 변수 없음: '{}'",
        VariablesKey::CloseVariablesWindow => "변수 창 닫기",
        VariablesKey::NoMetadataLoaded => "아직 저장소 메타데이터를 불러오지 않았습니다.",
        VariablesKey::FetchStoreMetadata => "저장소 메타데이터 가져오기",
        VariablesKey::NoVariablesFound => "이 데이터셋 저장소에서 변수를 찾지 못했습니다.",
        VariablesKey::RefreshStoreMetadata => "저장소 메타데이터 새로 고침",
        VariablesKey::TruncatedVariables => {
            "이 폴더의 변수 {}/{}개를 표시합니다. 모두 찾으려면 검색을 사용하세요."
        }
        VariablesKey::TruncatedFolders => {
            "폴더 {}/{}개를 표시합니다. 모두 찾으려면 검색을 사용하세요."
        }
        VariablesKey::NoVariableSelected => "선택한 변수가 없습니다.",
        VariablesKey::DimensionSliders => "차원 슬라이더",
        VariablesKey::CloseDimensionPanel => "차원 패널 닫기",
        VariablesKey::PlotData => "데이터 그리기",
        VariablesKey::PlotDataHover => "이 변수를 가져와 그립니다",
        VariablesKey::PlotDataOverlayHover => "이 변수를 가져와 플롯 위에 오버레이로 추가합니다",
        VariablesKey::AddOverlay => "오버레이 추가",
        VariablesKey::AddOverlayHover => {
            "켜져 있으면 데이터 그리기가 이 변수를 오버레이로 추가합니다"
        }
        VariablesKey::InfoPath => "경로:",
        VariablesKey::InfoUnits => "단위: {}",
        VariablesKey::InfoDescription => "설명: {}",
        VariablesKey::InfoShape => "형태: {}",
        VariablesKey::InfoDimensions => "차원: {}",
        VariablesKey::InfoTime => "시간: {} -> {}",
        VariablesKey::InfoResolution => "해상도: {}",
        VariablesKey::InfoSize => "크기: {} MB",
        VariablesKey::InfoAttributes => "속성 (.zattrs)",
        VariablesKey::Download => "다운로드",
        VariablesKey::DownloadCells => "{} (셀 {}개)",
        VariablesKey::LargeSelectionPyramid => {
            "큰 2D 선택 영역 (셀 {}개, {} MB): 다중 해상도 피라미드 집계가 자동으로 사용됩니다."
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "이 큰 선택 영역 (셀 {}개, {} MB)에서는 3D 구체와 3D 표면 메시를 사용할 수 없습니다. 2D 평면과 1D 선 플롯은 그대로 사용할 수 있습니다."
        }
        VariablesKey::LargeVolumeDisabled => {
            "이 선택 영역에서는 3D 볼륨과 포인트 클라우드를 사용할 수 없습니다: 볼륨 크기 ({} MB)가 {} MB GPU 저장 버퍼 한도를 넘습니다. 2D 평면, 1D 선, 3D 구체는 그대로 사용할 수 있습니다."
        }
        VariablesKey::RoleNone => "없음",
        VariablesKey::RoleAnimated => "애니메이션",
        VariablesKey::RoleGrid => "격자",
        VariablesKey::Index => "인덱스:",
        VariablesKey::ChannelN => "채널 {}",
    })
}
