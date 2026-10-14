use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "활성 데이터셋",
        DatasetKey::NoDatasetLoaded => "불러온 데이터셋이 없습니다",
        DatasetKey::MemoryCache => "메모리 및 캐시",
        DatasetKey::SampleDatasets => "샘플",
        DatasetKey::DropFilesHere => "파일 또는 폴더를 끌어다 놓으세요",
        DatasetKey::SourceUrlOrPath => "대상 URL / 파일 경로",
        DatasetKey::LoadingMetadata => "백그라운드에서 저장소 메타데이터를 확인하는 중...",
        // Dataset panel
        DatasetKey::OpenSource => "소스 열기",
        DatasetKey::DatasetManager => "데이터셋 관리자",
        DatasetKey::CloseDatasetHover => "데이터셋 닫기",
        // Active dataset card
        DatasetKey::TargetUriHover => "대상 URI:\n{}",
        DatasetKey::VariableCountOne => "변수 {}개",
        DatasetKey::VariableCountMany => "변수 {}개",
        DatasetKey::InspectVariables => "변수 살펴보기",
        DatasetKey::Reload => "다시 불러오기",
        DatasetKey::BadgeVolume4D => "4D 볼륨",
        DatasetKey::BadgeMatrix2D => "2D 행렬",
        DatasetKey::BadgeStore => "저장소",
        // Source intake
        DatasetKey::Load => "불러오기",
        DatasetKey::TargetPathHint => "https://... 또는 경로 (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "카탈로그...",
        DatasetKey::Clear => "지우기",
        DatasetKey::FormatLabel => "형식:",
        DatasetKey::SampleSeasFire => "SeasFire 전 지구 산불 (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "2D 절차적 행렬 (테스트)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "원격 Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "로컬 Zarr (파일 시스템)",
        DatasetKey::FormatRemoteIcechunk => "원격 Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "로컬 Icechunk (파일 시스템)",
        DatasetKey::FormatRemoteGeoTiff => "원격 GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "로컬 GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "로컬 NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "4D 참값 기준 볼륨 (절차적)",
        DatasetKey::FormatMatrix2D => "2D 절차적 행렬 (절차적)",
        DatasetKey::FormatRemoteGeoTiffShort => "원격 GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "로컬 GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "로컬 NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "4D 참값 기준 볼륨",
        DatasetKey::FormatMatrix2DShort => "2D 절차적 행렬",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "관리자에 등록된 데이터셋이 없습니다.",
        DatasetKey::LoadedCount => "불러옴 ({}):",
        DatasetKey::ClearAll => "모두 지우기",
        DatasetKey::ClearAllHover => "관리자에서 모든 데이터셋을 제거합니다",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "이 데이터셋 제거",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "블록 {}개 | 적중률 {}%",
        DatasetKey::CacheMenuTitle => "통합 다중 변수 메모리 캐시",
        DatasetKey::GlobalMemoryUsage => "전체 RAM 사용량:",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "종류별 메모리 사용량:",
        DatasetKey::ResidentHyperslabs => "상주 하이퍼슬랩: {} MB (블록 {}개)",
        DatasetKey::MultiResPyramid => "다중 해상도 피라미드: {} MB",
        DatasetKey::CacheHitsMisses => "캐시 적중: {} | 미스: {} (적중률: {}%)",
        DatasetKey::PrefetchInFlight => "백그라운드 미리 가져오기: {}개 진행 중",
        DatasetKey::AllBlocksCached => "버퍼 준비됨 / 모든 블록 캐시됨",
        DatasetKey::SummaryPerVariable => "변수별 요약:",
        DatasetKey::NoVariablesCached => "(메모리에 캐시된 변수가 없습니다)",
        DatasetKey::VariableCacheSize => "({} MB, 블록 {}개)",
        DatasetKey::ClearVariableCacheHover => "이 변수의 캐시 지우기",
        DatasetKey::PrefetchCapacitySettings => "미리 가져오기 및 용량 설정",
        DatasetKey::EnablePrefetching => "백그라운드 미리 가져오기 사용",
        DatasetKey::CacheLimitSuffix => " MB 한도",
        DatasetKey::WindowSizeSuffix => " 프리페치 범위 (스텝)",
        DatasetKey::PrefetchWorkersSuffix => " 미리 가져오기 작업자 (스레드)",
        DatasetKey::FlushAllCaches => "모든 캐시 비우기",
        // Catalog
        DatasetKey::CatalogTitle => "데이터셋 카탈로그",
        DatasetKey::CatalogStoreCount => "저장소 {}개",
        DatasetKey::CatalogBlurb => {
            "엄선한 클라우드 Zarr, Icechunk 및 절차적 기준 데이터셋입니다. 데이터셋을 선택하면 불러옵니다."
        }
        DatasetKey::CatalogSearchHint => "이름, 설명 또는 URL로 필터...",
        DatasetKey::CatalogCategory => "분류:",
        DatasetKey::CategoryAll => "전체",
        DatasetKey::CategoryProcedural => "절차적",
        DatasetKey::NoCatalogMatches => "검색과 일치하는 데이터셋이 없습니다.",
        DatasetKey::ClearSearch => "검색 지우기",
        DatasetKey::SelectAndLoadDataset => "데이터셋 선택 및 불러오기",
        DatasetKey::SelectAndLoad => "선택 및 불러오기",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "지원하지 않는 형식",
        DatasetKey::DropReleaseToLoad => "놓으면 데이터셋을 불러옵니다",
        DatasetKey::DropSupportedList => "지원 형식: {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "예시:",
        DatasetKey::HeroSeasFireHover => "전 지구 산불 및 기후 재청크 데이터셋 (Zarr)",
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A 트루 컬러 COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "합성 4D 시공간 볼륨",
        DatasetKey::HeroCatalogHover => "데이터셋 카탈로그 둘러보기",
        DatasetKey::LoadSampleHover => "샘플 불러오기: {}",
        DatasetKey::HeroHintLong => "URL / 로컬 경로 / 파일을 아무 곳에나 끌어다 놓기",
        DatasetKey::HeroHintMedium => "URL / 경로 / 파일 끌어다 놓기",
        DatasetKey::HeroHintShort => "URL / 파일 놓기",
        DatasetKey::HeroDropLong => "데이터셋을 놓아 불러오기 (.nc, .h5, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropMedium => "데이터셋 놓기 (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "데이터셋을 놓아 불러오기",
        DatasetKey::HeroUnsupportedLong => {
            "지원하지 않는 형식입니다. 지원 형식: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "지원하지 않는 형식 (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "지원하지 않는 형식",
        DatasetKey::HeroInputHintLong => {
            "https://... 또는 경로 (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... 또는 경로 (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL 또는 경로...",
        DatasetKey::ClearInputHover => "입력 지우기",
        DatasetKey::LoadDatasetHover => "데이터셋을 불러오고 변수 열기",
        DatasetKey::HopOctantHover => "클릭하여 다른 옥탄트로 이동",
        DatasetKey::HeroLoadingSource => "불러오는 중: {}",
        DatasetKey::HeroLoading => "불러오는 중...",
        DatasetKey::HeroLoaded => "불러옴: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "웹 샌드박스 보안 때문에 브라우저에서는 로컬 파일 경로를 직접 읽을 수 없습니다.\n\n브라우저에서 로컬 데이터셋을 보려면:\n1. 로컬 HTTP 서버로 폴더나 파일을 제공하세요: `npx serve` 또는 `python3 -m http.server`\n2. URL을 입력하세요: `http://localhost:8000/my_dataset`\n\n또는 Octant 데스크톱 버전을 실행하세요 (`cargo run --release`)."
        }
        DatasetKey::Plate => "플레이트",
        DatasetKey::PlateAcquisition => "획득",
        DatasetKey::PlateAllAcquisitions => "전체",
        DatasetKey::PlateFields => "시야:",
        DatasetKey::PlatePickWell => "웰을 선택하면 시야가 표시됩니다",
        DatasetKey::PlateLoadingFields => "웰의 시야를 읽는 중",
        DatasetKey::PlateNoFields => "이 획득에는 시야가 없습니다",
        DatasetKey::PlateWellUnreadable => "웰의 시야를 읽을 수 없습니다. 클릭하여 다시 시도",
        DatasetKey::CategoryMicroscopy => "현미경",
    })
}
