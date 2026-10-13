use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> &'static str {
    match key {
        PlotKey::Heatmap => "Heatmap",
        PlotKey::Line => "1D Line",
        PlotKey::Surface => "Surface",
        PlotKey::Volume => "Volume",
        PlotKey::Sphere => "Sphere",
        PlotKey::PointCloud => "Point Cloud",
        PlotKey::CompositeTrueColor => "True color",
        PlotKey::CompositeFalseColor => "False color",
        PlotKey::CompositeRgb => "RGB composite",
        PlotKey::CompositeOverlay => "Channel overlay",
        PlotKey::AlignGeo => "On another longitude/latitude grid (not supported yet)",
        PlotKey::AlignIndexOnly => "Same grid shape, but no coordinates match the plot",
        PlotKey::AlignIncompatible => "Shares no grid with the plot",
        PlotKey::OverlayPlotFirst => "Plot a variable first",
        PlotKey::OverlayHeatmapOnly => "Overlays draw over heatmaps",
        PlotKey::OverlayStackFull => "At most 4 overlays",
        PlotKey::OverlayAlreadyLayer => "Already a layer of the plot",
        PlotKey::OverlayCantAdd => "Can't add overlay",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Select Visualization Projection",
        PlotKey::GridHealpixDiscrete => "HEALPix / Discrete",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Unsupported for {} grid",
        PlotKey::DisabledAggregation => "Disabled: 2D Aggregation is on",
        PlotKey::Requires3D => "Requires 3D Data",
        PlotKey::DisabledGpuLimit => "Disabled: {} MB > 128 MB GPU limit",
        PlotKey::ScalarField => "Scalar Field",
        PlotKey::LabelsHeatmapOnly => "Label images draw on the heatmap",
    }
}
