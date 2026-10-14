use crate::i18n::keys::NoticesKey;

pub fn text(key: NoticesKey) -> Option<&'static str> {
    Some(match key {
        NoticesKey::DismissHover => "닫기",
        NoticesKey::CopyDetails => "세부 정보 복사",
        NoticesKey::RevealInFinder => "Finder에서 보기",
        NoticesKey::OpenFolder => "폴더 열기",
        NoticesKey::NoVariablesFound => "변수를 찾지 못함",
        NoticesKey::CouldntOpenDataset => "데이터셋을 열 수 없음",
        NoticesKey::UnsupportedFile => "지원하지 않는 파일",
        NoticesKey::CouldntLoadData => "데이터를 불러올 수 없음",
        NoticesKey::CoordinatesUnavailable => "좌표를 사용할 수 없음",
        NoticesKey::CoordinatesFailedDetail => {
            "'{}' 좌표를 불러오지 못했습니다 ({}). 축에 인덱스를 표시합니다"
        }
        NoticesKey::CoastlinesUnavailable => "해안선을 사용할 수 없음",
        NoticesKey::VolumeTooLargeForGpu => "볼륨이 GPU에 비해 너무 큼",
        NoticesKey::VolumeBufferTooLargeDetail => {
            "볼륨 데이터 버퍼 크기 (요소 {}개 / {} 바이트)가 GPU 저장 버퍼 최대 한도 ({})를 넘습니다"
        }
        NoticesKey::VolumeTextureTooLargeDetail => {
            "볼륨 {}x{}x{}: GPU 3D 텍스처 한도 ({})를 넘습니다"
        }
        NoticesKey::VolumeUploadSkipped => "볼륨 업로드를 건너뜀",
        NoticesKey::VolumeUploadMismatchDetail => "값 {}개, 대상 텍스처 {}x{}x{}",
        NoticesKey::DataTooLargeForGpuBuffer => "데이터가 GPU 버퍼에 비해 너무 큼",
        NoticesKey::BufferTooLargeDetail => {
            "{}: 데이터 크기 ({} 바이트)가 GPU 버퍼 용량 ({} 바이트)을 넘어 쓰기를 건너뜁니다"
        }
        NoticesKey::BufferRangeTooLargeDetail => {
            "{}: 요소 {}의 범위 (+{}개 요소)가 GPU 버퍼 용량 ({} 바이트)을 넘어 쓰기를 건너뜁니다"
        }
        NoticesKey::LinePlotTooLargeForGpu => "선 플롯이 GPU에 비해 너무 큼",
        NoticesKey::LinePayloadTooLargeDetail => {
            "선 데이터가 {} 바이트를 차지합니다. 이 GPU는 최대 {}까지 바인딩합니다."
        }
        NoticesKey::FontLoadFailed => "글꼴을 불러올 수 없습니다",
        NoticesKey::FontLoadFailedDetail => "{} 텍스트가 네모로 표시될 수 있습니다: {}",
        NoticesKey::DataFontMissing => "데이터셋의 일부 텍스트에 사용할 글꼴이 없습니다",
        NoticesKey::DataFontMissingDetail => {
            "이 데이터셋의 {} 텍스트가 네모로 표시될 수 있습니다. Noto Sans CJK 같은 글꼴을 설치하세요."
        }
        NoticesKey::OmeMetadataNonConforming => "OME-Zarr 메타데이터가 사양을 따르지 않습니다",
        NoticesKey::LabelIdsBeyond32Bits => {
            "32비트를 넘는 레이블 ID는 더 작은 ID와 색을 공유합니다"
        }
        NoticesKey::OmeMetadataNonConformingDetail => {
            "{} 버전 메타데이터({})가 사양을 따르지 않습니다. 읽을 수 있는 내용만 표시합니다."
        }
    })
}
