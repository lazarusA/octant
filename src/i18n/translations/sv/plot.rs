use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Värmekarta",
        PlotKey::Line => "1D-linje",
        PlotKey::Surface => "Yta",
        PlotKey::Volume => "Volym",
        PlotKey::Sphere => "Sfär",
        PlotKey::PointCloud => "Punktmoln",
        PlotKey::CompositeTrueColor => "Äkta färg",
        PlotKey::CompositeFalseColor => "Falsk färg",
        PlotKey::CompositeRgb => "RGB-komposit",
        PlotKey::CompositeOverlay => "Kanalöverlägg",
        PlotKey::AlignGeo => "På ett annat longitud/latitud-rutnät (stöds inte än)",
        PlotKey::AlignIndexOnly => "Samma rutnätsform, men inga koordinater matchar diagrammet",
        PlotKey::AlignIncompatible => "Delar inget rutnät med diagrammet",
        PlotKey::OverlayPlotFirst => "Plotta en variabel först",
        PlotKey::OverlayHeatmapOnly => "Överlägg ritas över värmekartor",
        PlotKey::OverlayStackFull => "Högst 4 överlägg",
        PlotKey::OverlayAlreadyLayer => "Redan ett lager i diagrammet",
        PlotKey::OverlayCantAdd => "Kan inte lägga till överlägg",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Välj visualiseringsprojektion",
        PlotKey::GridHealpixDiscrete => "HEALPix / diskret",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Stöds inte för {}-rutnät",
        PlotKey::DisabledAggregation => "Avaktiverad: 2D-aggregering är på",
        PlotKey::Requires3D => "Kräver 3D-data",
        PlotKey::DisabledGpuLimit => "Avaktiverad: {} MB > GPU-gränsen {} MB",
        PlotKey::ScalarField => "Skalärfält",
        PlotKey::LabelsHeatmapOnly => "Etikettbilder ritas bara på värmekartan",
    })
}
