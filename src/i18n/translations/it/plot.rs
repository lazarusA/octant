use crate::i18n::keys::PlotKey;

pub fn text(key: PlotKey) -> Option<&'static str> {
    Some(match key {
        PlotKey::Heatmap => "Mappa di calore",
        PlotKey::Line => "Linea 1D",
        PlotKey::Surface => "Superficie",
        PlotKey::Volume => "Volume",
        PlotKey::Sphere => "Sfera",
        PlotKey::PointCloud => "Nuvola di punti",
        PlotKey::CompositeTrueColor => "Colori reali",
        PlotKey::CompositeFalseColor => "Falsi colori",
        PlotKey::CompositeRgb => "Composito RGB",
        PlotKey::CompositeOverlay => "Sovrapposizione di canali",
        PlotKey::AlignGeo => {
            "Su un'altra griglia di longitudine/latitudine (non ancora supportato)"
        }
        PlotKey::AlignIndexOnly => {
            "Stessa forma della griglia, ma nessuna coordinata corrisponde al grafico"
        }
        PlotKey::AlignIncompatible => "Non condivide alcuna griglia con il grafico",
        PlotKey::OverlayPlotFirst => "Prima traccia una variabile",
        PlotKey::OverlayHeatmapOnly => "Le sovrapposizioni si disegnano sulle mappe di calore",
        PlotKey::OverlayStackFull => "Al massimo 4 sovrapposizioni",
        PlotKey::OverlayAlreadyLayer => "Già un livello del grafico",
        PlotKey::OverlayCantAdd => "Impossibile aggiungere la sovrapposizione",
        PlotKey::OverlayNamedReason => "'{}': {}",
        PlotKey::SelectProjection => "Seleziona la proiezione di visualizzazione",
        PlotKey::GridHealpixDiscrete => "HEALPix / discreta",
        PlotKey::DisabledItem => "{} ({})",
        PlotKey::UnsupportedForGrid => "Non supportato per la griglia {}",
        PlotKey::DisabledAggregation => "Disattivato: l'aggregazione 2D è attiva",
        PlotKey::Requires3D => "Richiede dati 3D",
        PlotKey::DisabledGpuLimit => "Disattivato: {} MB > limite GPU di 128 MB",
        PlotKey::ScalarField => "Campo scalare",
        PlotKey::LabelsHeatmapOnly => {
            "Le immagini di etichette si disegnano solo sulla mappa di calore"
        }
    })
}
