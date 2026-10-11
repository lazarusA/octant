use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "線",
        PlotOptionsKey::LinesHover => "点を結ぶ連続した線を表示します",
        PlotOptionsKey::Scatter => "散布",
        PlotOptionsKey::ScatterHover => "各点の位置に散布マーカーを表示します",
        PlotOptionsKey::PointSize => "点のサイズ:",
        PlotOptionsKey::CustomColor => "カスタム色",
        PlotOptionsKey::CustomColorHover => "カラーマップの代わりに、線と散布点を単色で描画します",
        PlotOptionsKey::LineColorHover => "線 / 散布点の色。クリックして色を選択します。",
        PlotOptionsKey::LineProfile => "ラインプロファイル",
        PlotOptionsKey::AllLinesSeries => "全ライン系列",
        PlotOptionsKey::ProfileIndex => "プロファイル番号",
        PlotOptionsKey::SingleProfile => "プロファイルは1つだけです。",
        PlotOptionsKey::AspectRatio => "縦横比",
        PlotOptionsKey::AspectRatioHover => {
            "オンにすると、2Dプロットは行列データの縦横比 (幅/高さ) を保ちます。オフにすると、2Dプロットはキャンバス全体に広がります。"
        }
        PlotOptionsKey::Algorithm => "アルゴリズム",
        PlotOptionsKey::AlgoDvr => "ボリュームレイマーチング (DVR)",
        PlotOptionsKey::AlgoMip => "最大値投影 (MIP)",
        PlotOptionsKey::AlgoMinIp => "最小値投影 (MinIP)",
        PlotOptionsKey::AlgoXray => "平均値投影 (X線)",
        PlotOptionsKey::AlgoLabelSurface => "カテゴリラベル表面",
        PlotOptionsKey::AlgoAbsorption => "吸収 RGBA",
        PlotOptionsKey::AlgoAdditive => "加算 RGBA",
        PlotOptionsKey::AlgoIndexed => "インデックス RGBA",
        PlotOptionsKey::Quality => "品質",
        PlotOptionsKey::QualityHover => "各レイに沿ったボクセルあたりのサンプル数",
        PlotOptionsKey::Density => "密度",
        PlotOptionsKey::Attenuation => "減衰",
        PlotOptionsKey::ZScale => "Zスケール",
        PlotOptionsKey::ModeSmooth => "滑らか",
        PlotOptionsKey::ModeBumpy => "凹凸",
        PlotOptionsKey::ModeSteps => "階段",
        PlotOptionsKey::ModeVoxel => "ボクセル",
        PlotOptionsKey::Height => "高さ",
        PlotOptionsKey::FlipHeights => "高さを反転",
        PlotOptionsKey::FlipHeightsHover => {
            "最大値が平らに置かれ、それより小さい値がその上に盛り上がります"
        }
        PlotOptionsKey::Size => "サイズ",
    })
}
