use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "활성 데이터셋",
        DatasetKey::NoDatasetLoaded => "불러온 데이터셋이 없습니다",
        DatasetKey::MemoryCache => "메모리 및 캐시",
        DatasetKey::SampleDatasets => "샘플",
        DatasetKey::OpenFolder => "폴더 열기",
        DatasetKey::OpenFile => "파일 열기",
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
        DatasetKey::FormatGrib => "로컬 또는 원격 GRIB2 (.grib2/.grb2)",
        DatasetKey::FormatGribShort => "GRIB2",
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
        DatasetKey::HeroSeasFireHover => "전 지구 산불 및 기후 재청크 데이터셋 (Zarr)",
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A 트루 컬러 COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "합성 4D 시공간 볼륨",
        DatasetKey::HeroCatalogHover => "데이터셋 카탈로그 둘러보기",
        DatasetKey::LoadSampleHover => "샘플 불러오기: {}",
        DatasetKey::HeroDropLong => "데이터셋을 놓아 불러오기 (.nc, .h5, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropMedium => "데이터셋 놓기 (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "데이터셋을 놓아 불러오기",
        DatasetKey::HeroUnsupportedLong => {
            "지원하지 않는 형식입니다. 지원 형식: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "지원하지 않는 형식 (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "지원하지 않는 형식",
        DatasetKey::HeroInputHint => "데이터셋을 놓거나 경로 또는 URL 입력 (.zarr, .icechunk, .nc, .h5, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URL 또는 경로...",
        DatasetKey::ClearInputHover => "입력 지우기",
        DatasetKey::LoadDatasetHover => "데이터셋을 불러오고 변수 열기",
        DatasetKey::HopOctantHover => "클릭하여 다른 옥탄트로 이동",
        DatasetKey::HeroLoadingSource => "불러오는 중: {}",
        DatasetKey::HeroLoading => "불러오는 중...",
        DatasetKey::HeroLoaded => "불러옴: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "웹 샌드박스 보안 때문에 브라우저에서는 로컬 파일 경로를 직접 읽을 수 없습니다.\n\n'폴더 열기' 또는 '파일 열기'로 열거나 창에 끌어다 놓으세요.\n\n브라우저에서 로컬 데이터셋을 보려면:\n1. 로컬 HTTP 서버로 폴더나 파일을 제공하세요: `npx serve` 또는 `python3 -m http.server`\n2. URL을 입력하세요: `http://localhost:8000/my_dataset`\n\n또는 Octant 데스크톱 버전을 실행하세요 (`cargo run --release`)."
        }
        DatasetKey::LocalNoZarrMetadata => {
            "이 폴더에는 브라우저가 읽을 수 있는 Zarr 메타데이터가 없습니다. 상위 폴더가 아니라 데이터 자체의 폴더(zarr.json 또는 .zgroup이 있는 폴더)를 선택하세요. Safari는 이름이 점으로 시작하는 파일을 숨기므로 Zarr v2 폴더(.zgroup, .zarray)는 Chrome, Firefox 또는 Octant 데스크톱 앱에서만 열립니다. Zarr v3 폴더는 모든 브라우저에서 열립니다."
        }
        DatasetKey::NeedsDataWorkers => {
            "NetCDF와 HDF5 파일은 데이터 워커가 읽는데, 이 브라우저에서 워커가 시작되지 않았습니다. 페이지를 새로 고치거나 Octant 데스크톱 앱에서 파일을 여세요."
        }
        DatasetKey::MatArraysLeftOut => {
            "이 MATLAB 파일의 배열 {}개는 표시되지 않습니다: 텍스트, 셀 배열, 빈 배열과 희소 배열, 복소수, 객체는 지원되지 않습니다"
        }
        DatasetKey::VariablesTooLargeForBrowser => {
            "이 파일의 변수 {}개는 브라우저에서 읽기에 너무 큽니다: 보려면 Octant 데스크톱 앱에서 여세요"
        }
        DatasetKey::GribMessagesLeftOut => {
            "이 GRIB 파일의 메시지 {}개는 표시되지 않습니다: 격자, 제품 또는 압축 방식이 지원되지 않거나 다른 메시지와 시간 및 레벨이 겹칩니다"
        }
        DatasetKey::Plate => "플레이트",
        DatasetKey::PlateAcquisition => "획득",
        DatasetKey::PlateAllAcquisitions => "전체",
        DatasetKey::PlateFields => "시야:",
        DatasetKey::PlatePickWell => "웰을 선택하면 시야가 표시됩니다",
        DatasetKey::PlateLoadingFields => "웰의 시야를 읽는 중",
        DatasetKey::PlateNoFields => "이 획득에는 시야가 없습니다",
        DatasetKey::PlateWellUnreadable => "웰의 시야를 읽을 수 없습니다. 클릭하여 다시 시도",
        DatasetKey::CategoryMicroscopy => "현미경 이미지",
        DatasetKey::CategorySoil => "토양",
        DatasetKey::BadgeDesktopOnly => "데스크톱 전용",
        DatasetKey::DesktopOnlyHint => {
            "이 서버는 브라우저 접근을 허용하지 않습니다. Octant 데스크톱 앱에서 여세요."
        }
    })
}
