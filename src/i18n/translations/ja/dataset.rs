use crate::i18n::keys::DatasetKey;

pub fn text(key: DatasetKey) -> Option<&'static str> {
    Some(match key {
        DatasetKey::ActiveDataset => "使用中のデータセット",
        DatasetKey::NoDatasetLoaded => "データセットが読み込まれていません",
        DatasetKey::MemoryCache => "メモリとキャッシュ",
        DatasetKey::SampleDatasets => "サンプル",
        DatasetKey::DropFilesHere => "ファイルまたはフォルダーをドラッグ&ドロップ",
        DatasetKey::SourceUrlOrPath => "URL / ファイルパス",
        DatasetKey::LoadingMetadata => "バックグラウンドでストアのメタデータを確認中...",
        // Dataset panel
        DatasetKey::OpenSource => "ソースを開く",
        DatasetKey::DatasetManager => "データセット管理",
        DatasetKey::CloseDatasetHover => "データセットを閉じる",
        // Active dataset card
        DatasetKey::TargetUriHover => "対象URI:\n{}",
        DatasetKey::VariableCountOne => "{} 個の変数",
        DatasetKey::VariableCountMany => "{} 個の変数",
        DatasetKey::InspectVariables => "変数を確認",
        DatasetKey::Reload => "再読み込み",
        DatasetKey::BadgeVolume4D => "4Dボリューム",
        DatasetKey::BadgeMatrix2D => "2D行列",
        DatasetKey::BadgeStore => "ストア",
        // Source intake
        DatasetKey::Load => "読み込み",
        DatasetKey::TargetPathHint => "https://... またはパス (.zarr, .nc, .tif)",
        DatasetKey::CatalogEllipsis => "カタログ...",
        DatasetKey::Clear => "クリア",
        DatasetKey::FormatLabel => "形式:",
        DatasetKey::SampleSeasFire => "SeasFire 世界の山火事 (Zarr)",
        DatasetKey::SampleSentinel2 => "Sentinel-2 L2A COG (GeoTIFF)",
        DatasetKey::SampleMatrixTest => "2D手続き型行列 (テスト)",
        // Store formats (menu entries, then the shorter selected texts)
        DatasetKey::FormatRemoteZarr => "リモート Zarr (HTTP/S3)",
        DatasetKey::FormatLocalZarr => "ローカル Zarr (ファイルシステム)",
        DatasetKey::FormatRemoteIcechunk => "リモート Icechunk (HTTP/S3)",
        DatasetKey::FormatLocalIcechunk => "ローカル Icechunk (ファイルシステム)",
        DatasetKey::FormatRemoteGeoTiff => "リモート GeoTIFF / COG (HTTP/S3)",
        DatasetKey::FormatLocalGeoTiff => "ローカル GeoTIFF (.tif/.tiff/.cog)",
        DatasetKey::FormatLocalNetCdf => "ローカル NetCDF / HDF5 (.nc/.h5)",
        DatasetKey::FormatVolume4D => "4D既知解ボリューム (手続き型)",
        DatasetKey::FormatMatrix2D => "2D手続き型行列 (手続き型)",
        DatasetKey::FormatRemoteGeoTiffShort => "リモート GeoTIFF / COG",
        DatasetKey::FormatLocalGeoTiffShort => "ローカル GeoTIFF",
        DatasetKey::FormatLocalNetCdfShort => "ローカル NetCDF / HDF5",
        DatasetKey::FormatVolume4DShort => "4D既知解ボリューム",
        DatasetKey::FormatMatrix2DShort => "2D手続き型行列",
        // Dataset manager
        DatasetKey::NoDatasetsRegistered => "登録されているデータセットはありません。",
        DatasetKey::LoadedCount => "読み込み済み ({}):",
        DatasetKey::ClearAll => "すべてクリア",
        DatasetKey::ClearAllHover => "すべてのデータセットを管理から削除",
        DatasetKey::DatasetRowHover => "{}\nURI: {}",
        DatasetKey::RemoveDatasetHover => "このデータセットを削除",
        // Cache statistics
        DatasetKey::CacheUsage => "{} MB / {} MB",
        DatasetKey::CacheBlocksHitRate => "{} ブロック | ヒット率 {}%",
        DatasetKey::CacheMenuTitle => "全変数共通のメモリキャッシュ",
        DatasetKey::GlobalMemoryUsage => "RAMメモリ使用量 (全体):",
        DatasetKey::CacheUsagePercent => "{} MB / {} MB ({}%)",
        DatasetKey::MemoryBreakdown => "種類別のメモリ内訳:",
        DatasetKey::ResidentHyperslabs => "常駐ハイパースラブ: {} MB ({} ブロック)",
        DatasetKey::MultiResPyramid => "多解像度ピラミッド: {} MB",
        DatasetKey::CacheHitsMisses => "キャッシュヒット: {} | ミス: {} (ヒット率: {}%)",
        DatasetKey::PrefetchInFlight => "バックグラウンド先読み: {} 件実行中",
        DatasetKey::AllBlocksCached => "バッファ準備完了 / 全ブロックをキャッシュ済み",
        DatasetKey::SummaryPerVariable => "変数ごとの概要:",
        DatasetKey::NoVariablesCached => "(メモリにキャッシュされた変数はありません)",
        DatasetKey::VariableCacheSize => "({} MB、{} ブロック)",
        DatasetKey::ClearVariableCacheHover => "この変数のキャッシュをクリア",
        DatasetKey::PrefetchCapacitySettings => "先読みと容量の設定",
        DatasetKey::EnablePrefetching => "バックグラウンド先読みを有効にする",
        DatasetKey::CacheLimitSuffix => " MB 上限",
        DatasetKey::WindowSizeSuffix => " ウィンドウサイズ (ステップ)",
        DatasetKey::PrefetchWorkersSuffix => " 先読みワーカー (スレッド)",
        DatasetKey::FlushAllCaches => "すべてのキャッシュを破棄",
        // Catalog
        DatasetKey::CatalogTitle => "データセットカタログ",
        DatasetKey::CatalogStoreCount => "{} 個のストア",
        DatasetKey::CatalogBlurb => {
            "厳選したクラウドの Zarr、Icechunk、手続き型の正解データセットです。データセットを選ぶと読み込みます。"
        }
        DatasetKey::CatalogSearchHint => "名前、説明、URLで絞り込み...",
        DatasetKey::CatalogCategory => "カテゴリ:",
        DatasetKey::CategoryAll => "すべて",
        DatasetKey::CategoryProcedural => "手続き型",
        DatasetKey::NoCatalogMatches => "検索に一致するデータセットはありません。",
        DatasetKey::ClearSearch => "検索をクリア",
        DatasetKey::SelectAndLoadDataset => "データセットを選択して読み込み",
        DatasetKey::SelectAndLoad => "選択して読み込み",
        // Drop zone
        DatasetKey::DropTypeNotSupported => "非対応の形式です",
        DatasetKey::DropReleaseToLoad => "離すとデータセットを読み込みます",
        DatasetKey::DropSupportedList => "対応形式: {}",
        // Hero landing
        DatasetKey::HeroTryPrefix => "例:",
        DatasetKey::HeroSeasFireHover => "世界の山火事と気候の再チャンク済みデータセット (Zarr)",
        DatasetKey::HeroSentinel2Hover => "Sentinel-2 L2A トゥルーカラー COG (AWS S3)",
        DatasetKey::HeroVolume4DHover => "合成された4D時空間ボリューム",
        DatasetKey::HeroCatalogHover => "データセットカタログを見る",
        DatasetKey::LoadSampleHover => "サンプルを読み込み: {}",
        DatasetKey::HeroHintLong => {
            "URLやローカルパスを貼り付けるか、ファイルをどこにでもドラッグ&ドロップ"
        }
        DatasetKey::HeroHintMedium => "URLやパスを貼り付けるか、ファイルをドラッグ&ドロップ",
        DatasetKey::HeroHintShort => "URLを貼り付けるかファイルをドロップ",
        DatasetKey::HeroDropLong => {
            "ドロップしてデータセットを読み込み (.nc, .h5, .zarr, .icechunk, .tif)"
        }
        DatasetKey::HeroDropMedium => "データセットをドロップ (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroDropShort => "ドロップして読み込み",
        DatasetKey::HeroUnsupportedLong => {
            "非対応の形式です。対応形式: .nc, .h5, .zarr, .icechunk, .tif"
        }
        DatasetKey::HeroUnsupportedMedium => "非対応の形式 (.nc, .zarr, .icechunk, .tif)",
        DatasetKey::HeroUnsupportedShort => "非対応の形式",
        DatasetKey::HeroInputHintLong => {
            "https://... またはパス (.zarr, .icechunk, .nc, .h5, .tiff, ...)"
        }
        DatasetKey::HeroInputHintMedium => "https://... またはパス (.zarr, .nc, .tiff, ...)",
        DatasetKey::HeroInputHintShort => "URLまたはパス...",
        DatasetKey::ClearInputHover => "入力をクリア",
        DatasetKey::LoadDatasetHover => "データセットを読み込んで変数を開く",
        DatasetKey::HopOctantHover => "クリックで次のオクタントへ",
        DatasetKey::HeroLoadingSource => "読み込み中: {}",
        DatasetKey::HeroLoading => "読み込み中...",
        DatasetKey::HeroLoaded => "読み込み完了: {}",
        // Errors
        DatasetKey::BrowserLocalPaths => {
            "ブラウザーではWebのサンドボックスによる制限のため、ローカルファイルのパスを直接読み込めません。\n\nブラウザーでローカルのデータセットを表示するには:\n1. ローカルHTTPサーバーでフォルダーまたはファイルを配信します: `npx serve` または `python3 -m http.server`\n2. URLを入力します: `http://localhost:8000/my_dataset`\n\nまたは、Octantのネイティブデスクトップ版を実行してください (`cargo run --release`)。"
        }
    })
}
