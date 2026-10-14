use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Varmekart",
        PlotKey::Line => "1D-linje",
        PlotKey::Surface => "Overflate",
        PlotKey::Volume => "Volum",
        PlotKey::Sphere => "Kule",
        PlotKey::PointCloud => "Punktsky",
        PlotKey::CompositeTrueColor => "Ekte farger",
        PlotKey::CompositeFalseColor => "Falske farger",
        PlotKey::CompositeRgb => "RGB-kompositt",
        PlotKey::CompositeOverlay => "Kanaloverlegg",
        PlotKey::AlignGeo => "På et annet lengde-/breddegradsrutenett (støttes ikke ennå)",
        PlotKey::AlignIndexOnly => {
            "Samme rutenettform, men ingen koordinater samsvarer med plottet"
        }
        PlotKey::AlignIncompatible => "Deler ikke rutenett med plottet",
        PlotKey::OverlayPlotFirst => "Plott en variabel først",
        PlotKey::OverlayHeatmapOnly => "Overlegg tegnes over varmekart",
        PlotKey::OverlayStackFull => "Høyst 4 overlegg",
        PlotKey::OverlayAlreadyLayer => "Allerede et lag i plottet",
        PlotKey::OverlayCantAdd => "Kan ikke legge til overlegg",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Velg visualiseringsprojeksjon",
        PlotKey::GridHealpixDiscrete => "HEALPix / diskret",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Støttes ikke for {}-rutenett",
        PlotKey::DisabledAggregation => "Slått av: 2D-aggregering er på",
        PlotKey::Requires3D => "Krever 3D-data",
        PlotKey::DisabledGpuLimit => "Slått av: {} MB > 128 MB GPU-grense",
        PlotKey::ScalarField => "Skalarfelt",
        PlotKey::LabelsHeatmapOnly => "Etikettbilder tegnes bare på varmekartet",
    })
}
