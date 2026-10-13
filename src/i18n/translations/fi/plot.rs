use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Lämpökartta",
        PlotKey::Line => "1D-viiva",
        PlotKey::Surface => "Pinta",
        PlotKey::Volume => "Tilavuus",
        PlotKey::Sphere => "Pallo",
        PlotKey::PointCloud => "Pistepilvi",
        PlotKey::CompositeTrueColor => "Luonnolliset värit",
        PlotKey::CompositeFalseColor => "Väärät värit",
        PlotKey::CompositeRgb => "RGB-yhdistelmä",
        PlotKey::CompositeOverlay => "Kanavapäällystaso",
        PlotKey::AlignGeo => "Eri pituus-/leveysasteruudukossa (ei vielä tuettu)",
        PlotKey::AlignIndexOnly => {
            "Sama ruudukon muoto, mutta mikään koordinaatti ei vastaa kuvaajaa"
        }
        PlotKey::AlignIncompatible => "Ei yhteistä ruudukkoa kuvaajan kanssa",
        PlotKey::OverlayPlotFirst => "Piirrä ensin muuttuja",
        PlotKey::OverlayHeatmapOnly => "Päällystasot piirretään lämpökarttojen päälle",
        PlotKey::OverlayStackFull => "Enintään 4 päällystasoa",
        PlotKey::OverlayAlreadyLayer => "Jo kuvaajan taso",
        PlotKey::OverlayCantAdd => "Päällystasoa ei voi lisätä",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Valitse visualisoinnin projektio",
        PlotKey::GridHealpixDiscrete => "HEALPix / diskreetti",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Ei tuettu ruudukkotyypille {}",
        PlotKey::DisabledAggregation => "Pois käytöstä: 2D-aggregointi on päällä",
        PlotKey::Requires3D => "Vaatii 3D-datan",
        PlotKey::DisabledGpuLimit => "Pois käytöstä: {} MB > 128 MB:n GPU-raja",
        PlotKey::ScalarField => "Skalaarikenttä",
        PlotKey::LabelsHeatmapOnly => "Tunnistekuvat piirretään lämpökarttaan",
    })
}
