use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Heatmap",
        PlotKey::Line => "1D-Linie",
        PlotKey::Surface => "Oberfläche",
        PlotKey::Volume => "Volumen",
        PlotKey::Sphere => "Kugel",
        PlotKey::PointCloud => "Punktwolke",
        PlotKey::CompositeTrueColor => "Echtfarben",
        PlotKey::CompositeFalseColor => "Falschfarben",
        PlotKey::CompositeRgb => "RGB-Komposit",
        PlotKey::CompositeOverlay => "Kanalüberlagerung",
        PlotKey::AlignGeo => "Auf einem anderen Längen-/Breitengitter (noch nicht unterstützt)",
        PlotKey::AlignIndexOnly => "Gleiche Gitterform, aber keine Koordinaten passen zum Plot",
        PlotKey::AlignIncompatible => "Teilt kein Gitter mit dem Plot",
        PlotKey::OverlayPlotFirst => "Zuerst eine Variable plotten",
        PlotKey::OverlayHeatmapOnly => "Überlagerungen werden über Heatmaps gezeichnet",
        PlotKey::OverlayStackFull => "Höchstens 4 Überlagerungen",
        PlotKey::OverlayAlreadyLayer => "Bereits eine Ebene des Plots",
        PlotKey::OverlayCantAdd => "Überlagerung kann nicht hinzugefügt werden",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Darstellungsart wählen",
        PlotKey::GridHealpixDiscrete => "HEALPix / Diskret",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Nicht unterstützt für {}-Gitter",
        PlotKey::DisabledAggregation => "Deaktiviert: 2D-Aggregation ist aktiv",
        PlotKey::Requires3D => "Erfordert 3D-Daten",
        PlotKey::DisabledGpuLimit => "Deaktiviert: {} MB > {} MB GPU-Limit",
        PlotKey::ScalarField => "Skalarfeld",
        PlotKey::LabelsHeatmapOnly => "Label-Bilder werden nur auf der Heatmap gezeichnet",
    })
}
