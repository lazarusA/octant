use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> &'static str {
    match key {
        PlotOptionsKey::Lines => "Lines",
        PlotOptionsKey::LinesHover => "Show continuous lines connecting points",
        PlotOptionsKey::Scatter => "Scatter",
        PlotOptionsKey::ScatterHover => "Show scatter markers at each point location",
        PlotOptionsKey::PointSize => "Point Size:",
        PlotOptionsKey::CustomColor => "Custom Color",
        PlotOptionsKey::CustomColorHover => {
            "Use a solid line/scatter color instead of colormap evaluation"
        }
        PlotOptionsKey::LineColorHover => "Line / scatter color. Click to select color.",
        PlotOptionsKey::LineProfile => "Line Profile",
        PlotOptionsKey::AllLinesSeries => "All Lines Series",
        PlotOptionsKey::ProfileIndex => "Profile Index",
        PlotOptionsKey::SingleProfile => "Single profile available.",
        PlotOptionsKey::AspectRatio => "Aspect Ratio",
        PlotOptionsKey::AspectRatioHover => {
            "If checked, 2D plots preserve matrix data aspect ratio (width/height). \
             If unchecked, 2D plots expand to fill full canvas."
        }
        PlotOptionsKey::Algorithm => "Algorithm",
        PlotOptionsKey::AlgoDvr => "Volume Raymarching (DVR)",
        PlotOptionsKey::AlgoMip => "Maximum Intensity (MIP)",
        PlotOptionsKey::AlgoMinIp => "Minimum Intensity (MinIP)",
        PlotOptionsKey::AlgoXray => "Average Projection (X-ray)",
        PlotOptionsKey::AlgoLabelSurface => "Categorical Label Surface",
        PlotOptionsKey::AlgoAbsorption => "Absorption RGBA",
        PlotOptionsKey::AlgoAdditive => "Additive RGBA",
        PlotOptionsKey::AlgoIndexed => "Indexed RGBA",
        PlotOptionsKey::Quality => "Quality",
        PlotOptionsKey::QualityHover => "Samples per voxel along each ray",
        PlotOptionsKey::Density => "Density",
        PlotOptionsKey::Attenuation => "Attenuation",
        PlotOptionsKey::ZScale => "Z-Scale",
        PlotOptionsKey::ModeSmooth => "Smooth",
        PlotOptionsKey::ModeBumpy => "Bumpy",
        PlotOptionsKey::ModeSteps => "Steps",
        PlotOptionsKey::ModeVoxel => "Voxel",
        PlotOptionsKey::Height => "Height",
        PlotOptionsKey::FlipHeights => "Flip heights",
        PlotOptionsKey::FlipHeightsHover => {
            "The highest value lies flat and lower values rise above it"
        }
        PlotOptionsKey::Size => "Size",
    }
}
