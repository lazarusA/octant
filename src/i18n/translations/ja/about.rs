use crate::i18n::keys::AboutKey;

pub fn text(key: AboutKey) -> Option<&'static str> {
    Some(match key {
        AboutKey::WindowTitle => "Octantについて",
        AboutKey::IconsWindowTitle => "ネイティブベクターアイコン",
        AboutKey::IconsTab => "ベクターアイコン",
        AboutKey::Subtitle => "N次元データビューアー",
        AboutKey::Description => {
            "Octant は N 次元データセットのための対話型ビューアーで、ローカルとクラウドのオブジェクトストレージ、Zarr (v2/v3)、Icechunk にネイティブ対応しています。Rust で開発され、WGPU による GPU 高速化レンダリングを備えています。Octant は macOS、Linux、Windows でネイティブに動作します。"
        }
        AboutKey::HyperslabTitle => "ハイパースラブ切り出し",
        AboutKey::HyperslabDesc => "非同期 LRU チャンクキャッシュと多解像度ピラミッド。",
        AboutKey::StorageTitle => "Zarr と Icechunk にネイティブ対応",
        AboutKey::StorageDesc => "ローカル、S3、GCS、Azure、HTTP ストリーミングのバックエンド。",
        AboutKey::ShadersTitle => "ハードウェア高速化 WGPU シェーダー",
        AboutKey::ShadersDesc => "2D平面地図、3D球体、標高面、ボリューム、1Dプロファイル。",
        AboutKey::LinksHeading => "リンクとリソース",
        AboutKey::DocsLink => "Octant ドキュメント",
        AboutKey::GoodFaithNotice => {
            "Octant は第三者の成果を誠意をもってクレジットしています。記載漏れや誤った帰属があると思われる場合はお知らせください。修正または削除いたします。"
        }
        AboutKey::BrowseIcons => "ネイティブベクターアイコンを見る",
        AboutKey::BrowseIconsHover => "手続き型ベクターアイコンをポップオーバーで見る",
        AboutKey::Licensed => "MIT または Apache-2.0 ライセンス",
        AboutKey::OctantTeaser => {
            "x、y、z の符号 (+/-, +/-, +/-) は空間を8つのオクタントに分けます。データも同じように探索しましょう。軸を選び、スラブを切り出し、一度に1つの領域を見てください。"
        }
        AboutKey::OctantHover => "クリックで次のオクタントへ",
        AboutKey::ColormapCredits => "カラーマップのクレジット",
        AboutKey::ColormapCreditsNote => "同梱のカラーマップには作者のライセンスが適用されます。",
        AboutKey::LicenseTexts => "ライセンス文",
        AboutKey::ThirdPartyLicenses => "サードパーティライセンス",
        AboutKey::ThirdPartyNote => {
            "Octant にはオープンソースのクレート、フォント、C ライブラリ (netCDF、HDF5) が含まれ、それぞれ独自のライセンスに従います。"
        }
        AboutKey::ViewOnGithub => "GitHub で見る",
        AboutKey::IconsSearchHint => "アイコンを検索...",
        AboutKey::IconsAll => "すべて",
        AboutKey::IconsCopied => "{} をコピーしました",
        AboutKey::IconsNoMatch => "一致するベクターアイコンはありません。",
        AboutKey::IconsCategory => "カテゴリ: {}",
        AboutKey::IconsClickToCopy => "クリックで列挙子の識別子をコピー",
        AboutKey::IconsSizesTones => "サイズと色調",
        AboutKey::ClearSearchHover => "検索をクリア",
        AboutKey::FontLicensesNote => "Noto Sans SC、JP、KR（中国語・日本語・韓国語のテキスト）：",
    })
}
