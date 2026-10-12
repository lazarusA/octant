use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "线条",
        PlotOptionsKey::LinesHover => "显示连接各点的连续线条",
        PlotOptionsKey::Scatter => "散点",
        PlotOptionsKey::ScatterHover => "在每个点的位置显示散点标记",
        PlotOptionsKey::PointSize => "点大小：",
        PlotOptionsKey::CustomColor => "自定义颜色",
        PlotOptionsKey::CustomColorHover => "使用单一的线条/散点颜色，而不是按色图取色",
        PlotOptionsKey::LineColorHover => "线条 / 散点颜色。点击以选择颜色。",
        PlotOptionsKey::LineProfile => "线剖面",
        PlotOptionsKey::AllLinesSeries => "全部线条系列",
        PlotOptionsKey::ProfileIndex => "剖面索引",
        PlotOptionsKey::SingleProfile => "只有一个剖面可用。",
        PlotOptionsKey::AspectRatio => "宽高比",
        PlotOptionsKey::AspectRatioHover => {
            "勾选时，2D 图保持矩阵数据的宽高比（宽/高）。未勾选时，2D 图扩展以填满整个画布。"
        }
        PlotOptionsKey::Algorithm => "算法",
        PlotOptionsKey::AlgoDvr => "体光线步进 (DVR)",
        PlotOptionsKey::AlgoMip => "最大密度投影 (MIP)",
        PlotOptionsKey::AlgoMinIp => "最小密度投影 (MinIP)",
        PlotOptionsKey::AlgoXray => "平均投影（X 光）",
        PlotOptionsKey::AlgoLabelSurface => "分类标签表面",
        PlotOptionsKey::AlgoAbsorption => "吸收 RGBA",
        PlotOptionsKey::AlgoAdditive => "叠加 RGBA",
        PlotOptionsKey::AlgoIndexed => "索引 RGBA",
        PlotOptionsKey::Quality => "质量",
        PlotOptionsKey::QualityHover => "每条光线在每个体素内的采样数",
        PlotOptionsKey::Density => "密度",
        PlotOptionsKey::Attenuation => "衰减",
        PlotOptionsKey::ZScale => "Z 缩放",
        PlotOptionsKey::ModeSmooth => "平滑",
        PlotOptionsKey::ModeBumpy => "凹凸",
        PlotOptionsKey::ModeSteps => "阶梯",
        PlotOptionsKey::ModeVoxel => "体素",
        PlotOptionsKey::Height => "高度",
        PlotOptionsKey::FlipHeights => "翻转高度",
        PlotOptionsKey::FlipHeightsHover => "最大值平放，较小的值在其上方升起",
        PlotOptionsKey::Size => "大小",
    })
}
