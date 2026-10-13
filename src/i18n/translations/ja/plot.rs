use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "ヒートマップ",
        PlotKey::Line => "1Dライン",
        PlotKey::Surface => "サーフェス",
        PlotKey::Volume => "ボリューム",
        PlotKey::Sphere => "球体",
        PlotKey::PointCloud => "点群",
        PlotKey::CompositeTrueColor => "トゥルーカラー",
        PlotKey::CompositeFalseColor => "フォールスカラー",
        PlotKey::CompositeRgb => "RGB合成",
        PlotKey::CompositeOverlay => "チャンネル重ね合わせ",
        PlotKey::AlignGeo => "別の経度/緯度グリッド上にあります (未対応)",
        PlotKey::AlignIndexOnly => "グリッドの形状は同じですが、プロットと一致する座標がありません",
        PlotKey::AlignIncompatible => "プロットと共通のグリッドがありません",
        PlotKey::OverlayPlotFirst => "先に変数をプロットしてください",
        PlotKey::OverlayHeatmapOnly => "オーバーレイはヒートマップ上に描画されます",
        PlotKey::OverlayStackFull => "オーバーレイは最大4つです",
        PlotKey::OverlayAlreadyLayer => "すでにプロットのレイヤーです",
        PlotKey::OverlayCantAdd => "オーバーレイを追加できません",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "可視化の投影を選択",
        PlotKey::GridHealpixDiscrete => "HEALPix / 離散",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "{} グリッドには非対応",
        PlotKey::DisabledAggregation => "無効: 2D集約がオンです",
        PlotKey::Requires3D => "3Dデータが必要です",
        PlotKey::DisabledGpuLimit => "無効: {} MB > GPU上限 128 MB",
        PlotKey::ScalarField => "スカラー場",
        PlotKey::LabelsHeatmapOnly => "ラベル画像はヒートマップに描画されます",
    })
}
