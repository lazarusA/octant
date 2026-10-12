use crate::i18n::keys::CommonKey;

pub fn text(key: CommonKey) -> Option<&'static str> {
    Some(match key {
        CommonKey::Dataset => "データセット",
        CommonKey::Variables => "変数",
        CommonKey::Dimensions => "次元",
        CommonKey::Colormaps => "カラーマップ",
        CommonKey::Settings => "設定",
        CommonKey::About => "情報",
        CommonKey::Cache => "キャッシュ",
        CommonKey::Language => "言語",
        CommonKey::Loading => "読み込み中...",
        CommonKey::CloseEsc => "閉じる (Esc)",
        CommonKey::Cancel => "キャンセル",
        CommonKey::Save => "保存",
        CommonKey::Reset => "リセット",
        CommonKey::Move => "移動",
        CommonKey::MoveHover => "ドラッグで移動、ダブルクリックでリセット",
        CommonKey::Light => "ライト",
        CommonKey::Dark => "ダーク",
        CommonKey::SwitchToLightHover => "ライトモードに切り替え",
        CommonKey::SwitchToDarkHover => "ダークモードに切り替え",
        CommonKey::ToggleHeroHover => "ホーム画面の表示切り替え (右クリックで情報)",
        CommonKey::AboutOctantHover => "Octantについて",
        CommonKey::ToggleVariableControlsHover => "変数コントロールパネルの表示切り替え",
        CommonKey::FetchingProgress => "取得中 {} / {} ({}%)",
        CommonKey::Abort => "中止",
        CommonKey::AbortHover => "実行中のデータ転送を中断して中止します",
    })
}
