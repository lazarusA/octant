//! Japanese phrasings of the verbs.
//! Matched as substrings, as words are not set apart by spaces.
//! Single characters that are parts of common words (次 in 次元,
//! "dimension") are left out.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
        (Describe, &["説明", "とは", "情報"]),
        (Search, &["検索", "探して", "探す"]),
        (List, &["一覧", "変数は"]),
        (Goto, &["移動", "ジャンプ", "設定"]),
        (
            Open,
            &["開いて", "開く", "読み込んで", "読み込む", "切り替えて"],
        ),
        (Select, &["選択", "選んで", "選ぶ"]),
        (
            Plot,
            &["表示", "プロット", "描画", "描いて", "見せて", "可視化"],
        ),
        (Play, &["再生", "開始", "スタート", "アニメーション"]),
        (Pause, &["一時停止", "停止", "止めて", "ポーズ"]),
        (Next, &["次へ", "次のステップ", "進めて"]),
        (Previous, &["前へ", "前のステップ", "戻って", "戻る"]),
        (
            Colormap,
            &["カラーマップ", "カラーパレット", "パレット", "配色"],
        ),
        (
            ResetView,
            &["ビューをリセット", "表示をリセット", "リセット"],
        ),
    ],
    months: [
        &["1月"],
        &["2月"],
        &["3月"],
        &["4月"],
        &["5月"],
        &["6月"],
        &["7月"],
        &["8月"],
        &["9月"],
        &["10月"],
        &["11月"],
        &["12月"],
    ],
    axes: [
        &["時間", "日付"],
        &["緯度"],
        &["経度"],
        &["レベル", "深さ", "高さ"],
    ],
    dataset: &["データセット", "データ"],
    catalog: &["カタログ"],
    then: &["それから", "その後", "次に"],
};
