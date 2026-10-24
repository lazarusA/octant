//! Japanese phrasings of the verbs.
//! Matched as substrings, as words are not set apart by spaces.
//! Single characters that are parts of common words (次 in 次元,
//! "dimension") are left out.

use super::Phrases;
use super::Verb::*;

pub(super) static PHRASES: Phrases = Phrases {
    verbs: &[
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
    catalog: &["カタログ"],
    then: &["それから", "その後", "次に"],
};
