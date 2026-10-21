use crate::i18n::keys::VariablesKey;

pub fn text(key: VariablesKey) -> Option<&'static str> {
    Some(match key {
        VariablesKey::Group => "グループ",
        VariablesKey::DataType => "型",
        VariablesKey::Shape => "形状",
        VariablesKey::Description => "説明",
        VariablesKey::SearchVariables => "変数を検索...",
        VariablesKey::NoVariablesMatching => "'{}' に一致する変数はありません",
        VariablesKey::CloseVariablesWindow => "変数ウィンドウを閉じる",
        VariablesKey::NoMetadataLoaded => "ストアのメタデータはまだ読み込まれていません。",
        VariablesKey::FetchStoreMetadata => "ストアのメタデータを取得",
        VariablesKey::NoVariablesFound => "このデータセットのストアに変数が見つかりません。",
        VariablesKey::RefreshStoreMetadata => "ストアのメタデータを更新",
        VariablesKey::TruncatedVariables => {
            "このフォルダーの変数を {} 件表示しています (全 {} 件)。すべて見つけるには検索を使ってください。"
        }
        VariablesKey::TruncatedFolders => {
            "フォルダーを {} 件表示しています (全 {} 件)。すべて見つけるには検索を使ってください。"
        }
        VariablesKey::NoVariableSelected => "変数が選択されていません。",
        VariablesKey::DimensionSliders => "次元スライダー",
        VariablesKey::CloseDimensionPanel => "次元パネルを閉じる",
        VariablesKey::PlotData => "プロット",
        VariablesKey::PlotDataHover => "この変数を取得してプロットします",
        VariablesKey::PlotDataOverlayHover => {
            "この変数を取得して、プロットにオーバーレイとして追加します"
        }
        VariablesKey::AddOverlay => "オーバーレイ追加",
        VariablesKey::AddOverlayHover => {
            "オンの間、プロットはこの変数をオーバーレイとして追加します"
        }
        VariablesKey::InfoPath => "パス:",
        VariablesKey::InfoUnits => "単位: {}",
        VariablesKey::InfoDescription => "説明: {}",
        VariablesKey::InfoShape => "形状: {}",
        VariablesKey::InfoDimensions => "次元: {}",
        VariablesKey::InfoTime => "時間: {} -> {}",
        VariablesKey::InfoResolution => "解像度: {}",
        VariablesKey::InfoSize => "サイズ: {} MB",
        VariablesKey::InfoCrs => "座標参照系: {}",
        VariablesKey::InfoCrsUnsupported => "座標参照系: {}（未対応）",
        VariablesKey::InfoAttributes => "属性 (.zattrs)",
        VariablesKey::Download => "ダウンロード",
        VariablesKey::DownloadCells => "{} ({} セル)",
        VariablesKey::LargeSelectionPyramid => {
            "大きな2D選択範囲 ({} セル、{} MB): 多解像度ピラミッドによる自動集約が有効です。"
        }
        VariablesKey::LargeSelectionNoMeshes => {
            "この大きな選択範囲 ({} セル、{} MB) では3D球体と3Dサーフェスのメッシュは無効です。2D平面と1Dラインのプロットは引き続き使えます。"
        }
        VariablesKey::LargeVolumeDisabled => {
            "この選択範囲では3Dボリュームと点群は無効です: ボリュームサイズ ({} MB) が{} MBのGPUストレージバッファ上限を超えています。2D平面、1Dライン、3D球体は引き続き使えます。"
        }
        VariablesKey::RoleNone => "なし",
        VariablesKey::RoleAnimated => "アニメーション",
        VariablesKey::RoleGrid => "グリッド",
        VariablesKey::Index => "インデックス:",
        VariablesKey::ChannelN => "チャンネル {}",
        VariablesKey::LevelN => "レベル {}",
        VariablesKey::LevelAuto => "自動",
        VariablesKey::LevelOf => "レベル {}/{}",
        VariablesKey::AutoLevelOf => "自動、レベル {}/{}",
    })
}
