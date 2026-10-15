use crate::i18n::keys::SettingsKey;

pub fn text(key: SettingsKey) -> Option<&'static str> {
    Some(match key {
        SettingsKey::ReasonComposite => "色は合成チャンネルから決まります。",
        SettingsKey::ReasonCustomColor => "「カスタム色」はすべての点を1色で描画します。",
        SettingsKey::ReasonAllSeries => "「全ライン系列」は各線をその番号で色分けします。",
        SettingsKey::ReasonIndexed => "「インデックス RGBA」はボクセルをラベル番号で色分けします。",
        SettingsKey::ReasonIndexedMissing => "「インデックス RGBA」は欠損ボクセルを描画しません。",
        SettingsKey::ReasonClassicOpaque => {
            "半透明の色を描画できるのは「ボリュームレイマーチング (DVR)」だけです。"
        }
        SettingsKey::ReasonNotTranslucent => {
            "「不透明度」が1未満か「アルファカーブ」があるときに有効になります。"
        }
        SettingsKey::ReasonCompositeOpaque => "合成は不透明で描画されます。",
        SettingsKey::ReasonLabelOpaque => "「カテゴリラベル表面」は常に不透明で描画されます。",
        SettingsKey::ReasonAlwaysBlends => "このアルゴリズムは常にレイに沿って合成します。",
        SettingsKey::CloseSettingsHover => "設定を閉じる",
        SettingsKey::SectionOverlays => "オーバーレイ",
        SettingsKey::SectionResolution => "解像度",
        SettingsKey::SectionView => "表示",
        SettingsKey::Layers => "レイヤー",
        SettingsKey::BaseLayerHover => "ベースレイヤー: キャンバス、軸、表示を決めます",
        SettingsKey::BaseTag => "ベース",
        SettingsKey::Color => "色",
        SettingsKey::HideLayerHover => "レイヤーを非表示",
        SettingsKey::ShowLayerHover => "レイヤーを表示",
        SettingsKey::RemoveLayerHover => "レイヤーを削除",
        SettingsKey::MoveDownHover => "下へ移動",
        SettingsKey::MoveUpHover => "上へ移動",
        SettingsKey::NoVariable => "(変数なし)",
        SettingsKey::ColorbarLabel => "カラーバーのラベル",
        SettingsKey::ResetLabelHover => "カラーバーのラベルを既定値に戻す",
        SettingsKey::ColorRange => "色の範囲",
        SettingsKey::Categorical => "カテゴリ",
        SettingsKey::CategoricalHover => {
            "離散カラーバー (一意な値を自動検出、または10個の等間隔ビン)。"
        }
        SettingsKey::LockBoundsHover => {
            "最小値/最大値を固定し、タイムステップ間で色の対応を変えないようにします。"
        }
        SettingsKey::ResetBoundsHover => "範囲を現在のスライス/データセットの最小値と最大値に戻す",
        SettingsKey::ResetAllColorbar => "カラーバーをすべて既定値に戻す",
        SettingsKey::ResetAllColorbarHover => "カラーバーのラベルと範囲の両方を既定値に戻します",
        SettingsKey::RangeMin => "最小:",
        SettingsKey::RangeMax => "最大:",
        SettingsKey::NanColor => "NaN の色",
        SettingsKey::NanColorHover => "オフにすると、NaN/Inf の値は透明で描画されます。",
        SettingsKey::NanColorPickerHover => "NaN の色。クリックして色を選択します。",
        SettingsKey::LowClip => "下限クリップ",
        SettingsKey::LowClipHover => "cmin 未満の値をこの色にクリップします。",
        SettingsKey::LowClipPickerHover => {
            "下限クリップの色 (< 最小)。クリックして色を選択します。"
        }
        SettingsKey::HighClip => "上限クリップ",
        SettingsKey::HighClipHover => "cmax を超える値をこの色にクリップします。",
        SettingsKey::HighClipPickerHover => {
            "上限クリップの色 (> 最大)。クリックして色を選択します。"
        }
        SettingsKey::Scale => "スケール",
        SettingsKey::ScaleLinear => "線形",
        SettingsKey::ScaleLog => "対数",
        SettingsKey::ScaleSymlog => "対称対数",
        SettingsKey::ScaleSqrt => "平方根",
        SettingsKey::ScaleExponential => "指数",
        SettingsKey::ScaleLogHover => "対数スケール (非負のデータ)",
        SettingsKey::ScaleLogDisabledHover => {
            "無効: 最小値 >= 0 が必要です。負のデータには対称対数を使ってください。"
        }
        SettingsKey::ScaleParam => "パラメータ:",
        SettingsKey::Opacity => "不透明度",
        SettingsKey::OpacityHover => {
            "カラーマップで色付けした値の不透明度。NaN とクリップの色は独自のアルファを保ちます。ダブルクリックでリセットします。"
        }
        SettingsKey::AlphaCurve => "アルファカーブ",
        SettingsKey::AlphaCurveHint => "0.1, 0.4, 0.3  または  0:0, 0.5:1, 1:0",
        SettingsKey::AlphaCurveHover => {
            "カラーバーの始点から終点までのアルファ。値は等間隔に並びます (ステップ: 値ごとに等しいビン1つ)。position:alpha の指定点はバーに沿って配置されます。空欄ならカーブなし。"
        }
        SettingsKey::ClearCurveHover => "カーブをクリア",
        SettingsKey::Presets => "プリセット",
        SettingsKey::InterpStep => "ステップ",
        SettingsKey::InterpLinear => "線形",
        SettingsKey::PresetRampUp => "上り坂",
        SettingsKey::PresetRampDown => "下り坂",
        SettingsKey::PresetPeak => "山",
        SettingsKey::PresetSink => "谷",
        SettingsKey::PresetHideLow => "低い値を隠す",
        SettingsKey::PresetHideCenter => "中央を隠す",
        SettingsKey::Transparency => "透過",
        SettingsKey::TransparencyHover => {
            "半透明の色 (不透明度またはアルファカーブ) のとき、最も手前の層で背後を隠さず、すべての層を描画します。"
        }
        SettingsKey::Lighting => "ライティング",
        SettingsKey::LightingHover => "勾配でサンプルに陰影を付け、前面や輪郭の形を際立たせます",
        SettingsKey::Coastlines => "海岸線",
        SettingsKey::CoastlinesHover => "地理プロットに Natural Earth の海岸線を重ねます。",
        SettingsKey::ResetCoastlineColor => "リセット",
        SettingsKey::ResetCoastlineColorHover => "海岸線の色をテーマの既定値に戻す",
        SettingsKey::CoastlineColorHover => "海岸線の色。クリックして選択します。",
        SettingsKey::CoastlineResolution => "解像度:",
        SettingsKey::Lod110mHover => "110 m - 高速、常に利用可能 (内蔵)",
        SettingsKey::Lod50mHover => "50 m - 中程度の詳細 (バックグラウンドで読み込み)",
        SettingsKey::Lod10mHover => "10 m - 高詳細 (バックグラウンドで読み込み)",
        SettingsKey::LineWidth => "線の太さ",
        SettingsKey::CropToDomain => "データ領域で切り抜く",
        SettingsKey::CropToDomainHover => {
            "海岸線を使用中のデータセットの空間範囲に厳密に切り抜きます。"
        }
        SettingsKey::AutoRotate => "自動回転",
        SettingsKey::ResetView => "表示をリセット",
        SettingsKey::ResetViewHover => "3Dカメラの向きをリセット",
        SettingsKey::HoverCard => "ホバーカード",
        SettingsKey::HideHoverCardHover => "ホバーカードを隠す",
        SettingsKey::ShowHoverCardHover => "ホバーカードを表示",
        SettingsKey::Export => "エクスポート",
        SettingsKey::ExportFormat => "形式:",
        SettingsKey::ExportFolder => "フォルダー:",
        SettingsKey::OpenSaveDialog => "保存ダイアログを開く",
        SettingsKey::CropTool => "切り抜きツール",
        SettingsKey::Aggregation => "2D集約",
        SettingsKey::AggregationMandatoryHover => {
            "必須: 配列が単一GPUバッファの上限 ({} MB) を超えています。集約は無効にできません。"
        }
        SettingsKey::AggregationHover => {
            "多解像度ピラミッドによる縮小とビューポートの再サンプリング。"
        }
        SettingsKey::AggregationMode => "モード:",
        SettingsKey::AggMean => "平均 (ボックスフィルター)",
        SettingsKey::AggMax => "最大 (ピークを保持)",
        SettingsKey::AggMin => "最小 (谷を保持)",
        SettingsKey::AggNearest => "最近傍",
        SettingsKey::AggMeanHover => "2x2 セル内のサブピクセルを平均します (NaN は無視)。",
        SettingsKey::AggMaxHover => "極端に高い値を保持します。",
        SettingsKey::AggMinHover => "極端に低い値を保持します。",
        SettingsKey::AggNearestHover => "補間せずに最も近い点を高速に選びます。",
        SettingsKey::ResolutionLevel => "解像度レベル",
        SettingsKey::ResolutionLevelHover => {
            "プロットが使う解像度レベルです。自動では画面に収まる最も細かいレベルを使います。"
        }
        SettingsKey::CmykAutoMapped => "自動割り当てチャンネル: C (1)、M (2)、Y (3)、K (4)",
        SettingsKey::MultiChannelOverlay => "マルチチャンネル重ね合わせ",
        SettingsKey::MultiChannelOverlayHover => {
            "複数のチャンネルを加算で重ね、それぞれ固有の色合いで描画します。"
        }
        SettingsKey::CmykComposite => "CMYK 合成",
        SettingsKey::CmykCompositeHover => {
            "シアン、マゼンタ、イエロー、ブラック (CMYK) の4チャンネルをトゥルーカラー RGB に合成します。"
        }
        SettingsKey::RgbComposite => "RGB 合成",
        SettingsKey::RgbCompositeHover => "選択した3チャンネルをトゥルーカラー RGB に合成します。",
        SettingsKey::Channels => "チャンネル:",
        SettingsKey::ChannelsAll => "すべて",
        SettingsKey::ChannelsNone => "なし",
        SettingsKey::ChannelTintHover => "クリックしてチャンネルの色合いを変更",
        SettingsKey::BandNumber => "バンド {}",
        SettingsKey::ScaleBar => "スケールバー",
        SettingsKey::ShowScaleBarHover => "スケールバーを表示",
        SettingsKey::HideScaleBarHover => "スケールバーを隠す",
        SettingsKey::ChannelsAuto => "自動",
        SettingsKey::ChannelsAutoHover => {
            "各チャンネルの表示範囲を表示中のデータに合わせる（値の 0.1% から 99.9%）"
        }
        SettingsKey::ChannelsAutoUnavailable => "自動コントラストは 2D スライスで機能します",
        SettingsKey::ChannelSettingsHover => "表示範囲、カーブ、反転",
        SettingsKey::ChannelWindow => "表示範囲",
        SettingsKey::ChannelWindowHover => {
            "開始から終了までの値が、黒からチャンネル本来の色までに対応します"
        }
        SettingsKey::ChannelInvert => "反転",
        SettingsKey::ChannelCurve => "カーブ",
        SettingsKey::CurveLinear => "線形",
        SettingsKey::CurveLogarithmic => "対数",
        SettingsKey::CurveExponential => "指数",
        SettingsKey::CurvePolynomial => "多項式",
        SettingsKey::ChannelCoefficientHover => "カーブの係数",
        SettingsKey::LabelsFill => "塗りつぶし",
        SettingsKey::LabelsFillHover => "各ラベル全体を塗る",
        SettingsKey::LabelsOutline => "輪郭",
        SettingsKey::LabelsOutlineHover => "ラベル間の境界だけを描く",
    })
}
