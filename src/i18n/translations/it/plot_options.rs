use crate::i18n::keys::PlotOptionsKey;

pub fn text(key: PlotOptionsKey) -> Option<&'static str> {
    Some(match key {
        PlotOptionsKey::Lines => "Linee",
        PlotOptionsKey::LinesHover => "Mostra linee continue che collegano i punti",
        PlotOptionsKey::Scatter => "Punti",
        PlotOptionsKey::ScatterHover => "Mostra un marcatore in corrispondenza di ogni punto",
        PlotOptionsKey::PointSize => "Dimensione punti:",
        PlotOptionsKey::CustomColor => "Colore personalizzato",
        PlotOptionsKey::CustomColorHover => {
            "Usa un colore unico per linee/punti invece della mappa di colore"
        }
        PlotOptionsKey::LineColorHover => {
            "Colore di linee / punti. Clicca per scegliere il colore."
        }
        PlotOptionsKey::LineProfile => "Profilo di linea",
        PlotOptionsKey::AllLinesSeries => "Tutte le serie",
        PlotOptionsKey::ProfileIndex => "Indice del profilo",
        PlotOptionsKey::SingleProfile => "Disponibile un solo profilo.",
        PlotOptionsKey::AspectRatio => "Proporzioni",
        PlotOptionsKey::AspectRatioHover => {
            "Se attivo, i grafici 2D mantengono le proporzioni dei dati della matrice (larghezza/altezza). \
             Se disattivo, i grafici 2D si espandono per riempire tutta l'area di disegno."
        }
        PlotOptionsKey::Algorithm => "Algoritmo",
        PlotOptionsKey::AlgoDvr => "Raymarching del volume (DVR)",
        PlotOptionsKey::AlgoMip => "Intensità massima (MIP)",
        PlotOptionsKey::AlgoMinIp => "Intensità minima (MinIP)",
        PlotOptionsKey::AlgoXray => "Proiezione media (raggi X)",
        PlotOptionsKey::AlgoLabelSurface => "Superficie di etichette categoriali",
        PlotOptionsKey::AlgoAbsorption => "RGBA ad assorbimento",
        PlotOptionsKey::AlgoAdditive => "RGBA additivo",
        PlotOptionsKey::AlgoIndexed => "RGBA indicizzato",
        PlotOptionsKey::Quality => "Qualità",
        PlotOptionsKey::QualityHover => "Campioni per voxel lungo ogni raggio",
        PlotOptionsKey::Density => "Densità",
        PlotOptionsKey::Attenuation => "Attenuazione",
        PlotOptionsKey::ZScale => "Scala Z",
        PlotOptionsKey::ModeSmooth => "Liscia",
        PlotOptionsKey::ModeBumpy => "Rilievo",
        PlotOptionsKey::ModeSteps => "Gradini",
        PlotOptionsKey::ModeVoxel => "Voxel",
        PlotOptionsKey::Height => "Altezza",
        PlotOptionsKey::FlipHeights => "Inverti altezze",
        PlotOptionsKey::FlipHeightsHover => {
            "Il valore più alto resta piatto e i valori più bassi si alzano sopra di esso"
        }
        PlotOptionsKey::Size => "Dimensione",
    })
}
