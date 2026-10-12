use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Octant 정보",
        AboutKey::IconsWindowTitle => "네이티브 벡터 아이콘",
        AboutKey::IconsTab => "벡터 아이콘",
        AboutKey::Subtitle => "N차원 데이터 뷰어",
        AboutKey::Description => {
            "Octant는 로컬 및 클라우드 객체 저장소, Zarr (v2/v3), Icechunk를 기본 지원하는 N차원 데이터셋용 대화형 뷰어입니다. Rust로 만들었으며 WGPU를 통한 GPU 가속 렌더링을 사용합니다. Octant는 macOS, Linux, Windows에서 네이티브로 실행됩니다."
        }
        AboutKey::HyperslabTitle => "하이퍼슬랩 슬라이싱",
        AboutKey::HyperslabDesc => "비동기 LRU 청크 캐시와 다중 해상도 피라미드.",
        AboutKey::StorageTitle => "Zarr 및 Icechunk 기본 지원",
        AboutKey::StorageDesc => "로컬, S3, GCS, Azure, HTTP 스트리밍 백엔드.",
        AboutKey::ShadersTitle => "하드웨어 가속 WGPU 셰이더",
        AboutKey::ShadersDesc => "2D 평면 지도, 3D 구체, 고도 표면, 볼륨 및 1D 프로파일.",
        AboutKey::LinksHeading => "링크 및 자료",
        AboutKey::DocsLink => "Octant 문서",
        AboutKey::GoodFaithNotice => {
            "Octant는 제3자의 작업을 선의로 표기합니다. 누락되었거나 잘못 표기된 부분이 있다고 생각되면 알려 주세요. 수정하거나 삭제하겠습니다."
        }
        AboutKey::BrowseIcons => "네이티브 벡터 아이콘 둘러보기",
        AboutKey::BrowseIconsHover => "팝오버에서 절차적 벡터 아이콘을 살펴봅니다",
        AboutKey::Licensed => "MIT 또는 Apache-2.0 라이선스",
        AboutKey::OctantTeaser => {
            "x, y, z의 부호 (+/-, +/-, +/-)는 공간을 여덟 개의 옥탄트로 나눕니다. 데이터도 같은 방식으로 탐색하세요: 축을 고르고, 슬랩을 잘라, 한 번에 한 영역씩 살펴보세요."
        }
        AboutKey::OctantHover => "클릭하여 다음 옥탄트로 이동",
        AboutKey::ColormapCredits => "컬러맵 출처",
        AboutKey::ColormapCreditsNote => "내장 컬러맵은 제작자의 라이선스를 따릅니다.",
        AboutKey::LicenseTexts => "라이선스 전문",
        AboutKey::ThirdPartyLicenses => "제3자 라이선스",
        AboutKey::ThirdPartyNote => {
            "Octant에는 오픈 소스 크레이트, 글꼴, C 라이브러리 (netCDF, HDF5)가 포함되어 있으며, 각각 고유한 라이선스를 따릅니다."
        }
        AboutKey::ViewOnGithub => "GitHub에서 보기",
        AboutKey::IconsSearchHint => "아이콘 검색...",
        AboutKey::IconsAll => "전체",
        AboutKey::IconsCopied => "복사함: {}",
        AboutKey::IconsNoMatch => "일치하는 벡터 아이콘이 없습니다.",
        AboutKey::IconsCategory => "분류: {}",
        AboutKey::IconsClickToCopy => "클릭하여 enum 식별자 복사",
        AboutKey::IconsSizesTones => "크기 및 색조",
        AboutKey::ClearSearchHover => "검색 지우기",
        AboutKey::FontLicensesNote => "Noto Sans SC, JP, KR (중국어, 일본어, 한국어 텍스트):",
    })
}
