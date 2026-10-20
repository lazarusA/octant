use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "热力图",
        PlotKey::Line => "1D 线图",
        PlotKey::Surface => "曲面",
        PlotKey::Volume => "体渲染",
        PlotKey::Sphere => "球面",
        PlotKey::PointCloud => "点云",
        PlotKey::CompositeTrueColor => "真彩色",
        PlotKey::CompositeFalseColor => "假彩色",
        PlotKey::CompositeRgb => "RGB 合成",
        PlotKey::CompositeOverlay => "通道叠加",
        PlotKey::AlignGeo => "位于另一个经纬度网格上（暂不支持）",
        PlotKey::AlignIndexOnly => "网格形状相同，但没有与图匹配的坐标",
        PlotKey::AlignIncompatible => "与图没有共同的网格",
        PlotKey::OverlayPlotFirst => "请先绘制一个变量",
        PlotKey::OverlayHeatmapOnly => "叠加层只能绘制在热力图上",
        PlotKey::OverlayStackFull => "最多 4 个叠加层",
        PlotKey::OverlayAlreadyLayer => "已是图中的图层",
        PlotKey::OverlayCantAdd => "无法添加叠加层",
        PlotKey::OverlayNamedReason => "'{}'：{}",
        PlotKey::SelectProjection => "选择可视化投影",
        PlotKey::GridHealpixDiscrete => "HEALPix / 离散",
        PlotKey::DisabledItem => "{}（{}）",
        PlotKey::UnsupportedForGrid => "不支持 {} 网格",
        PlotKey::DisabledAggregation => "已禁用：2D 聚合已开启",
        PlotKey::DisabledGpuLimit => "已禁用：{} MB > {} MB GPU 上限",
        PlotKey::ScalarField => "标量场",
        PlotKey::LabelsHeatmapOnly => "标签图像仅在热图上绘制",
    })
}
